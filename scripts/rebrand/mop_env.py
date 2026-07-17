#!/usr/bin/env python3
"""Mechanical env-token mop: JCODE_* → NEXT_CODE_* / product_env dual-read.

Safe bulk replacements for the rebrand. Does NOT touch:
  - product_env implementation body (JCODE_ fallback strings stay)
  - LEGACY_SERVICE constants jcode-provider-service / jcode-secrets
  - com.jcode.mobile
  - *.jcode.sh
  - changelog / *_PLAN.md historical docs

Modes:
  --report
  --apply-test-setters     test set_var/remove_var/var/EnvVarGuard JCODE_→NEXT_CODE_
  --apply-product-env      production env::var("JCODE_X") → product_env("X")
  --apply-string-keys      const/static/array "JCODE_X" → "NEXT_CODE_X" (non-skip paths)
  --apply-prod-setters     production set_var/remove_var("JCODE_X") → NEXT_CODE_X
  --apply-scripts          .sh/.ps1/.py dual-read / prefer NEXT_CODE_
  --apply-ci               .yml/.yaml NEXT_CODE_* prefer (keep dual where intentional)
  --apply-comments         ~/.jcode → ~/.next-code, jcode_dir (comments) → next_code_dir
  --apply-all              all apply modes
  --dry-run
  --paths P [P ...]
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT_DEFAULT = Path(__file__).resolve().parents[2]

# --- skip paths / line guards -------------------------------------------------

KEEP_JCODE_SETTER_PATHS = {
    "crates/next-code-core/src/env.rs",  # product_env impl + dual-read unit tests
    "crates/next-code-storage/src/lib.rs",  # keep 1-2 explicit JCODE_HOME fallback tests
    "crates/next-code-storage/src/active_pids.rs",
}

SKIP_PRODUCT_ENV_PATHS = {
    "crates/next-code-core/src/env.rs",
}

# build-time cargo:rustc-env keys: convert carefully in string-keys pass
# but never rewrite product_env fallback format!("JCODE_{suffix}")
SKIP_FILE_GLOBS_COMMENT = (
    "changelog",
    "CHANGELOG",
    "_PLAN.md",
    "scripts/rebrand/",
)

LINE_SKIP = re.compile(
    r"(?i)("
    r"dual-?read|"
    r"legacy_jcode|jcode_compat|old_jcode_|"
    r"com\.jcode\.mobile|"
    r"claude-cli|codex_cli_rs|"
    r"[\w.-]*jcode\.sh\b|"
    r"falls?\s+back\s+to\s+[`']?JCODE_|"
    r"fallback\s+to\s+[`']?JCODE_|"
    r"legacy\s+[`']?JCODE_|"
    r"product_env|"  # leave nearby intentional JCODE_ alone
    r"formerly jcode|renamed from jcode|"
    r"jcode-provider-service|jcode-secrets|"
    r"format!\(\"JCODE_\{|"  # product_env body
    r"LEGACY_SERVICE"
    r")"
)

TEST_PATH_HINT = re.compile(
    r"(?:^|/)tests?/|/test_|_tests\.rs$|_test\.rs$|/benches/|client_session_tests/",
    re.IGNORECASE,
)
CFG_TEST = re.compile(r"#\[cfg\(test\)\]")
MOD_TESTS = re.compile(r"\bmod\s+tests\b")

VAR_CALL = re.compile(
    r"""(?P<full>(?P<std>std::)?env::var(?P<os>_os)?\(\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))"\s*\))"""
)
SET_CALL = re.compile(
    r"""(?P<full>(?P<qual>(?:crate::env::|next_code_core::env::|next_code::env::|super::|self::)?)(?P<fn>set_var|remove_var)\(\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))")"""
)
SET_CALL_STD = re.compile(
    r"""(?P<full>std::env::(?P<fn>set_var|remove_var)\(\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))")"""
)
ENV_GUARD = re.compile(
    r"""(?P<full>EnvVarGuard::(?P<fn>set|set_path|remove)\(\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))")"""
)
# Command::env / .env("JCODE_X", ...)
CMD_ENV = re.compile(
    r"""(?P<full>\.env\(\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))")"""
)
# const/static string keys
CONST_KEY = re.compile(
    r"""(?P<full>(?P<kw>const|static)\s+(?P<name>[A-Z0-9_]+)\s*:\s*&str\s*=\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))")"""
)
# bare "JCODE_SUFFIX" as a full string literal on a line (arrays, match arms, etc.)
BARE_KEY = re.compile(r'"JCODE_(?P<suffix>[A-Z0-9_]+)"')
# cargo:rustc-env / rerun-if-env-changed
CARGO_ENV = re.compile(
    r"""(?P<full>(?:cargo:rustc-env=|cargo:rerun-if-env-changed=|env!\()(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+)))"""
)

SIMPLE_VAR = re.compile(
    r"""(?P<full>(?P<prefix>std::)?env::var(?P<os>_os)?\(\s*"JCODE_(?P<suffix>[A-Z0-9_]+)"\s*\))"""
)
SKIP_CONVERT_HINT = re.compile(
    r"or_else|unwrap_or|product_env|product_var|JCODE_.*NEXT_CODE|NEXT_CODE_.*JCODE"
)

# format!("JCODE_PROVIDER_{}_API_KEY", ...) → NEXT_CODE_
FORMAT_PROVIDER = re.compile(
    r"""format!\(\s*"JCODE_(?P<body>[A-Z0-9_{}]+)"\s*"""
)


def rel_of(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def is_testish_path(rel: str) -> bool:
    return bool(TEST_PATH_HINT.search(rel))


def line_in_test_module(lines: list[str], lineno: int) -> bool:
    start = max(0, lineno - 250)
    window = "".join(lines[start:lineno])
    return bool(CFG_TEST.search(window) or MOD_TESTS.search(window))


def precompute_test_lines(lines: list[str], rel: str) -> set[int] | None:
    """Return set of 1-based line numbers in test context, or None if whole file is testish."""
    if is_testish_path(rel):
        return None  # whole file
    # Find #[cfg(test)] / mod tests markers; treat everything after the last
    # top-level test module start as testish is too coarse. Instead mark a
    # sliding window: any line within 0..EOF after a cfg(test)/mod tests is
    # candidate — approximate by: line is testish if any marker exists in the
    # previous 250 lines (same heuristic, computed once via prefix scan).
    markers: list[int] = []  # 0-based indices of marker lines
    for i, line in enumerate(lines):
        if CFG_TEST.search(line) or MOD_TESTS.search(line):
            markers.append(i)
    if not markers:
        return set()
    test_lines: set[int] = set()
    # For each line, check if any marker is in [line-250, line)
    # Efficient: two-pointer over markers
    j = 0
    for i in range(len(lines)):
        # advance j so markers[j] >= i-250
        while j < len(markers) and markers[j] < i - 250:
            j += 1
        # any marker in [i-250, i)?
        k = j
        while k < len(markers) and markers[k] < i:
            test_lines.add(i + 1)  # 1-based
            break
            k += 1  # pragma: no cover
    return test_lines


def is_test_line(test_set: set[int] | None, lineno: int) -> bool:
    if test_set is None:
        return True
    return lineno in test_set


def should_skip_line(line: str) -> bool:
    if "com.jcode.mobile" in line:
        return True
    if re.search(r"[\w.-]*jcode\.sh\b", line):
        return True
    if "jcode-provider-service" in line or "jcode-secrets" in line:
        return True
    if LINE_SKIP.search(line):
        return True
    return False


def skip_historical(rel: str) -> bool:
    if "CHANGELOG" in rel or rel.endswith("changelog.md"):
        return True
    if rel.endswith("_PLAN.md") or "/PLAN_" in rel or rel.endswith("PLAN.md"):
        # allow non-historical plans? rule says *_PLAN.md
        if rel.endswith("_PLAN.md") or rel.endswith("PLAN.md"):
            return True
    if "scripts/rebrand/" in rel:
        return True
    return False


SKIP_DIR_NAMES = {
    "target",
    ".git",
    "node_modules",
    ".next",
    "dist",
    "build",
    ".venv",
    "venv",
    "__pycache__",
}


def _walk(root: Path, suffixes: tuple[str, ...]) -> list[Path]:
    """Walk tree skipping heavy/vendor dirs early (do not descend into target/)."""
    out: list[Path] = []
    suffix_set = set(suffixes)
    keep_dot = {".github"}
    for dirpath, dirnames, filenames in os.walk(root):
        pruned = []
        for d in dirnames:
            if d in SKIP_DIR_NAMES:
                continue
            if d.startswith(".") and d not in keep_dot:
                continue
            pruned.append(d)
        dirnames[:] = pruned
        for name in filenames:
            p = Path(dirpath) / name
            if p.suffix in suffix_set:
                out.append(p)
    return sorted(out)


def iter_files(root: Path, paths: list[str] | None, suffixes: tuple[str, ...]) -> list[Path]:
    if paths:
        out: list[Path] = []
        for p in paths:
            fp = (root / p).resolve()
            if fp.is_file() and fp.suffix in suffixes:
                out.append(fp)
            elif fp.is_dir():
                out.extend(_walk(fp, suffixes))
        return out
    return _walk(root, suffixes)


def jcode_to_next(key: str) -> str:
    assert key.startswith("JCODE_")
    return "NEXT_CODE_" + key[len("JCODE_") :]


# --- report ------------------------------------------------------------------


def report(root: Path) -> int:
    files = iter_files(root, None, (".rs",))
    buckets: dict[str, list] = defaultdict(list)
    for path in files:
        rel = rel_of(path, root)
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        lines = text.splitlines(keepends=True)
        for i, line in enumerate(lines, 1):
            testish = is_testish_path(rel) or line_in_test_module(lines, i)
            for m in VAR_CALL.finditer(line):
                buckets["test_read" if testish else "prod_read"].append((rel, i, m.group("key")))
            for rx in (SET_CALL, SET_CALL_STD):
                for m in rx.finditer(line):
                    buckets["test_set" if testish else "prod_set"].append(
                        (rel, i, m.group("fn"), m.group("key"))
                    )
            for m in ENV_GUARD.finditer(line):
                buckets["env_guard"].append((rel, i, m.group("key")))
            for m in CONST_KEY.finditer(line):
                buckets["const"].append((rel, i, m.group("key")))
    print("=== mop_env report ===")
    for k, v in buckets.items():
        print(f"  {k:12s}: {len(v)}")
    # remaining JCODE_ line count
    total = 0
    for path in iter_files(
        root, None, (".rs", ".sh", ".ps1", ".py", ".yml", ".yaml", ".md", ".toml")
    ):
        rel = rel_of(path, root)
        if skip_historical(rel):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        total += text.count("JCODE_")
    print(f"  remaining JCODE_ token occurrences (non-historical): {total}")
    return 0


# --- apply test setters ------------------------------------------------------


def apply_test_setters(root: Path, files: list[Path], dry_run: bool) -> tuple[int, int]:
    changed_files = 0
    sites = 0
    for path in files:
        rel = rel_of(path, root)
        if rel in KEEP_JCODE_SETTER_PATHS:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        lines = text.splitlines(keepends=True)
        test_set = precompute_test_lines(lines, rel)
        if test_set is not None and len(test_set) == 0 and not is_testish_path(rel):
            continue
        out: list[str] = []
        file_hits = 0

        def bump_repl(m: re.Match[str]) -> str:
            nonlocal file_hits
            file_hits += 1
            return m.group(0).replace(
                f'"JCODE_{m.group("suffix")}"',
                f'"NEXT_CODE_{m.group("suffix")}"',
                1,
            )

        for i, line in enumerate(lines, 1):
            if should_skip_line(line):
                out.append(line)
                continue
            if not is_test_line(test_set, i):
                out.append(line)
                continue
            new = line
            new = SET_CALL.sub(bump_repl, new)
            new = SET_CALL_STD.sub(bump_repl, new)
            new = VAR_CALL.sub(bump_repl, new)
            new = ENV_GUARD.sub(bump_repl, new)
            new = CMD_ENV.sub(bump_repl, new)
            out.append(new)
        if file_hits:
            changed_files += 1
            sites += file_hits
            print(f"{'DRY ' if dry_run else 'WRITE'} test-set {rel} ({file_hits})")
            if not dry_run:
                path.write_text("".join(out), encoding="utf-8")
    print(f"apply-test-setters: files={changed_files} sites={sites}")
    return changed_files, sites


# --- apply product_env -------------------------------------------------------


def ensure_product_env_import(text: str, rel: str) -> str:
    """Best-effort: add product_env import if missing and conversions used it."""
    if "product_env" not in text:
        return text
    if re.search(r"\buse\b.*\bproduct_env\b", text) or "product_env_os" in text and re.search(
        r"\buse\b.*product_env", text
    ):
        # already has some use of product_env in import form?
        if re.search(r"use\s+[^\n]*product_env", text):
            return text
    # crate-local re-export via env::*
    if "next_code_core::env::" in text or "crate::env::" in text:
        # prefer qualified calls — leave unqualified product_env needing import
        pass
    # If product_env( appears unqualified, add import
    if not re.search(r"(?<![:\w])product_env(_os)?\s*\(", text):
        return text
    if re.search(r"use\s+[^\n]*\bproduct_env\b", text):
        return text

    # Choose import path by crate location
    if rel.startswith("crates/next-code-base/") or rel.startswith("crates/next-code-app-core/"):
        # re-export via crate::env
        import_line = "use crate::env::{product_env, product_env_os};\n"
        if "use crate::env::" in text:
            # already imports from crate::env — try to extend
            def ext(m: re.Match[str]) -> str:
                body = m.group(0)
                if "product_env" in body:
                    return body
                if body.rstrip().endswith(";"):
                    # use crate::env::foo;
                    if "{" in body:
                        return body.replace("}", ", product_env, product_env_os}")
                    return body  # leave alone
                return body

            return text  # don't be clever; hand-fix
        # insert after module docs / first use
    elif rel.startswith("src/") or rel.startswith("tests/"):
        import_line = "use next_code::env::{product_env, product_env_os};\n"
        # root package uses next_code
        if "crate::env::" in text or "use next_code::env" in text:
            return text
    else:
        import_line = "use next_code_core::env::{product_env, product_env_os};\n"

    # Insert after last inner-attribute / before first non-attr use or after mod docs
    lines = text.splitlines(keepends=True)
    insert_at = 0
    for i, line in enumerate(lines):
        if line.startswith("//!") or line.startswith("#![") or line.startswith("#[") and i < 30:
            insert_at = i + 1
            continue
        if line.startswith("use ") or line.startswith("pub use "):
            insert_at = i
            break
        if line.strip() == "":
            insert_at = i + 1
            continue
        break
    # Avoid duplicate
    if any("product_env" in ln and ln.strip().startswith("use ") for ln in lines):
        return text
    lines.insert(insert_at, import_line)
    return "".join(lines)


def apply_product_env(root: Path, files: list[Path], dry_run: bool) -> tuple[int, int]:
    changed_files = 0
    converted = 0
    needs_import: dict[str, int] = defaultdict(int)
    for path in files:
        rel = rel_of(path, root)
        if rel in SKIP_PRODUCT_ENV_PATHS:
            continue
        # also convert in non-test production; skip pure test files for this mode
        # (test files use NEXT_CODE_ setters and can read NEXT_CODE_ directly)
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        lines = text.splitlines(keepends=True)
        test_set = precompute_test_lines(lines, rel)
        out: list[str] = []
        file_conv = 0
        for i, line in enumerate(lines, 1):
            if should_skip_line(line):
                out.append(line)
                continue
            # In test modules, prefer NEXT_CODE_ string rewrite (done by test-setters),
            # not product_env (tests set exact keys).
            if is_test_line(test_set, i):
                out.append(line)
                continue
            if SKIP_CONVERT_HINT.search(line):
                out.append(line)
                continue

            def repl(m: re.Match[str]) -> str:
                nonlocal file_conv
                suffix = m.group("suffix")
                os_suf = m.group("os") or ""
                file_conv += 1
                if os_suf:
                    return f'product_env_os("{suffix}")'
                return f'product_env("{suffix}")'

            new = SIMPLE_VAR.sub(repl, line)
            out.append(new)

        if file_conv:
            new_text = "".join(out)
            new_text = ensure_product_env_import(new_text, rel)
            changed_files += 1
            converted += file_conv
            needs_import[rel] += file_conv
            print(f"{'DRY ' if dry_run else 'WRITE'} product-env {rel} ({file_conv})")
            if not dry_run:
                path.write_text(new_text, encoding="utf-8")
    print(f"apply-product-env: files={changed_files} conversions={converted}")
    if needs_import:
        print("  (imports auto-added when missing; verify compile)")
    return changed_files, converted


# --- apply prod setters ------------------------------------------------------


def apply_prod_setters(root: Path, files: list[Path], dry_run: bool) -> tuple[int, int]:
    """set_var/remove_var/Command.env in production → NEXT_CODE_."""
    changed_files = 0
    sites = 0
    for path in files:
        rel = rel_of(path, root)
        if rel in KEEP_JCODE_SETTER_PATHS or rel in SKIP_PRODUCT_ENV_PATHS:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        lines = text.splitlines(keepends=True)
        test_set = precompute_test_lines(lines, rel)
        out: list[str] = []
        file_hits = 0

        def bump(m: re.Match[str]) -> str:
            nonlocal file_hits
            file_hits += 1
            return m.group(0).replace(
                f'"JCODE_{m.group("suffix")}"',
                f'"NEXT_CODE_{m.group("suffix")}"',
                1,
            )

        for i, line in enumerate(lines, 1):
            if should_skip_line(line):
                out.append(line)
                continue
            # skip test modules — handled by test-setters
            if is_test_line(test_set, i):
                out.append(line)
                continue
            new = line
            new = SET_CALL.sub(bump, new)
            new = SET_CALL_STD.sub(bump, new)
            new = CMD_ENV.sub(bump, new)

            def fmt_repl(m: re.Match[str]) -> str:
                nonlocal file_hits
                body = m.group("body")
                file_hits += 1
                return m.group(0).replace(f"JCODE_{body}", f"NEXT_CODE_{body}", 1)

            new = FORMAT_PROVIDER.sub(fmt_repl, new)
            if 'format!("JCODE_' in new and not should_skip_line(new):
                newer = re.sub(
                    r'format!\(\s*"JCODE_',
                    'format!("NEXT_CODE_',
                    new,
                )
                if newer != new:
                    file_hits += 1
                    new = newer
            out.append(new)
        if file_hits:
            changed_files += 1
            sites += file_hits
            print(f"{'DRY ' if dry_run else 'WRITE'} prod-set {rel} ({file_hits})")
            if not dry_run:
                path.write_text("".join(out), encoding="utf-8")
    print(f"apply-prod-setters: files={changed_files} sites={sites}")
    return changed_files, sites


# --- apply string keys (const/static/env!/cargo) -----------------------------


def apply_string_keys(root: Path, files: list[Path], dry_run: bool) -> tuple[int, int]:
    changed_files = 0
    sites = 0
    for path in files:
        rel = rel_of(path, root)
        if rel in SKIP_PRODUCT_ENV_PATHS:
            continue
        if rel in KEEP_JCODE_SETTER_PATHS:
            # still allow const rewrites? no — leave storage dual-read tests
            pass
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        lines = text.splitlines(keepends=True)
        out: list[str] = []
        file_hits = 0
        for i, line in enumerate(lines, 1):
            if should_skip_line(line):
                out.append(line)
                continue
            # Keep explicit dual-read test assertions in env.rs / storage
            if rel in KEEP_JCODE_SETTER_PATHS:
                out.append(line)
                continue
            new = line

            def bump_const(m: re.Match[str]) -> str:
                nonlocal file_hits
                file_hits += 1
                return m.group(0).replace(
                    f'"JCODE_{m.group("suffix")}"',
                    f'"NEXT_CODE_{m.group("suffix")}"',
                    1,
                )

            new = CONST_KEY.sub(bump_const, new)

            def bump_cargo(m: re.Match[str]) -> str:
                nonlocal file_hits
                file_hits += 1
                return m.group(0).replace(
                    f"JCODE_{m.group('suffix')}",
                    f"NEXT_CODE_{m.group('suffix')}",
                    1,
                )

            new = CARGO_ENV.sub(bump_cargo, new)

            # DISABLE_JCODE_HOOKS → dual: leave for product_var_full hand-fix,
            # but rewrite plain DISABLE_JCODE_HOOKS string to mention both? skip.
            if "DISABLE_JCODE_" in new:
                out.append(new)
                continue

            # Bare string keys in non-comment production (arrays of env names)
            # Only rewrite if the whole string is JCODE_* and not a path/service name
            if '"JCODE_' in new and "jcode-provider" not in new:

                def bare(m: re.Match[str]) -> str:
                    nonlocal file_hits
                    suf = m.group("suffix")
                    # skip if this looks like a service/domain (no all-caps only — already)
                    file_hits += 1
                    return f'"NEXT_CODE_{suf}"'

                # avoid rewriting inside comments that already say legacy
                if not new.strip().startswith("//") or "legacy" not in new.lower():
                    # For comment lines, still rewrite env key names to NEXT_CODE_
                    newer = BARE_KEY.sub(bare, new)
                    new = newer
            out.append(new)
        if file_hits:
            changed_files += 1
            sites += file_hits
            print(f"{'DRY ' if dry_run else 'WRITE'} str-keys {rel} ({file_hits})")
            if not dry_run:
                path.write_text("".join(out), encoding="utf-8")
    print(f"apply-string-keys: files={changed_files} sites={sites}")
    return changed_files, sites


# --- scripts -----------------------------------------------------------------


SHELL_ASSIGN = re.compile(
    r"""(?P<full>(?P<export>export\s+)?(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))=)"""
)
SHELL_USE = re.compile(r"""\$\{?JCODE_(?P<suffix>[A-Z0-9_]+)\}?""")
SHELL_DEFAULT = re.compile(
    r"""\$\{JCODE_(?P<suffix>[A-Z0-9_]+):-(?P<default>[^}]*)\}"""
)

# Already dual-read patterns we should not double-wrap
ALREADY_DUAL = re.compile(
    r"NEXT_CODE_[A-Z0-9_]+.*JCODE_|JCODE_[A-Z0-9_]+.*NEXT_CODE_"
)


def mop_shell_line(line: str) -> tuple[str, int]:
    """Prefer NEXT_CODE_ with JCODE_ fallback for shell."""
    if should_skip_line(line) and "JCODE_" in line:
        # still allow simple default rewrites unless hard skip
        if "com.jcode" in line or "jcode.sh" in line:
            return line, 0
    if "JCODE_" not in line:
        return line, 0
    hits = 0
    new = line

    # export JCODE_X= → export NEXT_CODE_X= (and keep dual export if setting)
    # For export KEY=value used to *set* sandbox env, dual-export both.
    def export_repl(m: re.Match[str]) -> str:
        nonlocal hits
        suf = m.group("suffix")
        hits += 1
        exp = m.group("export") or ""
        # dual export: NEXT_CODE first; also set JCODE for legacy child tools
        # Keep simple single NEXT_CODE_ assignment — readers dual-read.
        return f"{exp}NEXT_CODE_{suf}="

    if re.match(r"^\s*(export\s+)?JCODE_[A-Z0-9_]+=", new):
        new2 = SHELL_ASSIGN.sub(export_repl, new, count=1)
        # If the line is `export JCODE_HOME=...` convert to NEXT_CODE_HOME
        new = new2

    # ${JCODE_X:-default} → ${NEXT_CODE_X:-${JCODE_X:-default}}
    def def_repl(m: re.Match[str]) -> str:
        nonlocal hits
        if "NEXT_CODE_" in m.group(0):
            return m.group(0)
        suf = m.group("suffix")
        default = m.group("default")
        # if default already has JCODE_ or NEXT_CODE, be careful
        hits += 1
        return f"${{NEXT_CODE_{suf}:-${{JCODE_{suf}:-{default}}}}}"

    if not ALREADY_DUAL.search(new):
        new = SHELL_DEFAULT.sub(def_repl, new)

    # bare $JCODE_X or ${JCODE_X} (no default) → ${NEXT_CODE_X:-${JCODE_X:-}}
    def use_repl(m: re.Match[str]) -> str:
        nonlocal hits
        suf = m.group("suffix")
        # skip if already inside a dual-read we just wrote
        hits += 1
        return "${NEXT_CODE_" + suf + ":-${JCODE_" + suf + ":-}}"

    # Only rewrite standalone uses not already dual
    if not ALREADY_DUAL.search(new) and "NEXT_CODE_" not in new:
        # avoid rewriting comments that document legacy names once
        if new.lstrip().startswith("#") and "legacy" in new.lower():
            return new, hits
        new2 = SHELL_USE.sub(use_repl, new)
        new = new2

    return new, hits


def apply_scripts(root: Path, dry_run: bool) -> tuple[int, int]:
    files = iter_files(root, None, (".sh", ".ps1", ".py"))
    changed_files = 0
    sites = 0
    for path in files:
        rel = rel_of(path, root)
        if skip_historical(rel) or "scripts/rebrand/" in rel:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        if "JCODE_" not in text:
            continue
        if path.suffix == ".sh":
            lines = text.splitlines(keepends=True)
            out = []
            file_hits = 0
            for line in lines:
                nl, h = mop_shell_line(line)
                file_hits += h
                out.append(nl)
            new_text = "".join(out)
        elif path.suffix == ".ps1":
            new_text = text
            file_hits = 0
            # $env:JCODE_X → prefer NEXT_CODE with fallback
            def ps1_env(m: re.Match[str]) -> str:
                nonlocal file_hits
                file_hits += 1
                suf = m.group(1)
                return f"$env:NEXT_CODE_{suf}"

            new_text2 = re.sub(r"\$env:JCODE_([A-Z0-9_]+)", ps1_env, new_text)
            new_text = new_text2
            # env:JCODE_ in other forms
            def ps1_env2(m: re.Match[str]) -> str:
                nonlocal file_hits
                file_hits += 1
                return f"env:NEXT_CODE_{m.group(1)}"

            new_text = re.sub(r"(?<![$\w])env:JCODE_([A-Z0-9_]+)", ps1_env2, new_text)
        else:  # .py
            file_hits = 0
            text0 = text

            def py_quoted(m: re.Match[str]) -> str:
                nonlocal file_hits
                file_hits += 1
                return f"{m.group('q')}NEXT_CODE_{m.group('suf')}{m.group('q')}"

            # Quoted JCODE_* env tokens → NEXT_CODE_*
            new_text = re.sub(
                r"""(?P<q>['"])JCODE_(?P<suf>[A-Z0-9_]+)(?P=q)""",
                py_quoted,
                text0,
            )
            # Dual-read for common get() value fetches (home/bin/etc.)
            for suf in ("HOME", "BIN", "RUNTIME_DIR", "SOCKET", "INSTALL_DIR"):
                for q in ('"', "'"):
                    pat = f"os.environ.get({q}NEXT_CODE_{suf}{q})"
                    dual = (
                        f"(os.environ.get({q}NEXT_CODE_{suf}{q}) "
                        f"or os.environ.get({q}JCODE_{suf}{q}))"
                    )
                    if pat in new_text and dual not in new_text:
                        new_text = new_text.replace(pat, dual)

        if new_text != text and file_hits:
            changed_files += 1
            sites += file_hits
            print(f"{'DRY ' if dry_run else 'WRITE'} script {rel} (~{file_hits})")
            if not dry_run:
                path.write_text(new_text, encoding="utf-8")
        elif new_text != text:
            changed_files += 1
            sites += 1
            print(f"{'DRY ' if dry_run else 'WRITE'} script {rel}")
            if not dry_run:
                path.write_text(new_text, encoding="utf-8")
    print(f"apply-scripts: files={changed_files} sites={sites}")
    return changed_files, sites


# --- CI yml ------------------------------------------------------------------


def apply_ci(root: Path, dry_run: bool) -> tuple[int, int]:
    files = list(iter_files(root, None, (".yml", ".yaml")))
    # also .github/scripts
    changed_files = 0
    sites = 0
    for path in files:
        rel = rel_of(path, root)
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        if "JCODE_" not in text:
            continue
        file_hits = 0
        lines = text.splitlines(keepends=True)
        out = []
        for line in lines:
            if "JCODE_" not in line:
                out.append(line)
                continue
            new = line
            # Prefer NEXT_CODE_ for env keys; if line only has JCODE_, rename.
            # If dual already present, leave JCODE_ as legacy companion.
            if "NEXT_CODE_" in new and "JCODE_" in new:
                # already dual — leave
                out.append(new)
                continue

            def bump(m: re.Match[str]) -> str:
                nonlocal file_hits
                file_hits += 1
                return "NEXT_CODE_" + m.group(1)

            new = re.sub(r"\bJCODE_([A-Z0-9_]+)", bump, new)
            out.append(new)
        new_text = "".join(out)
        if file_hits:
            changed_files += 1
            sites += file_hits
            print(f"{'DRY ' if dry_run else 'WRITE'} ci {rel} ({file_hits})")
            if not dry_run:
                path.write_text(new_text, encoding="utf-8")
    # ps1 under .github
    for path in (root / ".github").rglob("*.ps1") if (root / ".github").exists() else []:
        rel = rel_of(path, root)
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        if "JCODE_" not in text:
            continue
        file_hits = 0

        def bump(m: re.Match[str]) -> str:
            nonlocal file_hits
            file_hits += 1
            return "NEXT_CODE_" + m.group(1)

        new_text = re.sub(r"\bJCODE_([A-Z0-9_]+)", bump, text)
        if file_hits:
            changed_files += 1
            sites += file_hits
            print(f"{'DRY ' if dry_run else 'WRITE'} ci {rel} ({file_hits})")
            if not dry_run:
                path.write_text(new_text, encoding="utf-8")
    print(f"apply-ci: files={changed_files} sites={sites}")
    return changed_files, sites


# --- comments ----------------------------------------------------------------


def apply_comments(root: Path, dry_run: bool) -> tuple[int, int]:
    files = iter_files(
        root, None, (".rs", ".sh", ".ps1", ".py", ".md", ".toml", ".yml", ".yaml")
    )
    changed_files = 0
    sites = 0
    for path in files:
        rel = rel_of(path, root)
        if skip_historical(rel):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        if "jcode_dir" not in text and "~/.jcode" not in text and "$HOME/.jcode" not in text:
            continue
        new = text
        file_hits = 0

        # comments / docs only for jcode_dir — avoid renaming code identifiers
        # that are still real aliases. Rule: jcode_dir in comments → next_code_dir
        def comment_jcode_dir(m: re.Match[str]) -> str:
            nonlocal file_hits
            file_hits += 1
            return m.group(0).replace("jcode_dir", "next_code_dir")

        # line comments
        new2_lines = []
        for line in new.splitlines(keepends=True):
            if "jcode_dir" in line and (
                line.lstrip().startswith("//")
                or line.lstrip().startswith("#")
                or line.lstrip().startswith("*")
                or "///" in line[:10]
                or "//!" in line[:10]
            ):
                if "fn jcode_dir" in line or "jcode_dir(" in line:
                    new2_lines.append(line)
                    continue
                cnt = line.count("jcode_dir")
                file_hits += cnt
                new2_lines.append(line.replace("jcode_dir", "next_code_dir"))
            else:
                new2_lines.append(line)
        new = "".join(new2_lines)

        # ~/.jcode → ~/.next-code (mention legacy once is fine if dual-read docs)
        def home_repl(m: re.Match[str]) -> str:
            nonlocal file_hits
            file_hits += 1
            return m.group(0).replace(".jcode", ".next-code")

        # Don't rewrite in product_env / dual-read documentation lines that
        # intentionally mention ~/.jcode as legacy
        lines_out = []
        for line in new.splitlines(keepends=True):
            if "~/.jcode" in line or "$HOME/.jcode" in line or "%USERPROFILE%\\.jcode" in line:
                if re.search(r"(?i)legacy|dual-?read|formerly|compat", line):
                    lines_out.append(line)
                    continue
                # default path docs
                nl = line.replace("~/.jcode", "~/.next-code")
                nl = nl.replace("$HOME/.jcode", "$HOME/.next-code")
                nl = nl.replace("%USERPROFILE%\\.jcode", "%USERPROFILE%\\.next-code")
                if nl != line:
                    file_hits += 1
                lines_out.append(nl)
            else:
                lines_out.append(line)
        new = "".join(lines_out)

        if new != text:
            changed_files += 1
            sites += max(file_hits, 1)
            print(f"{'DRY ' if dry_run else 'WRITE'} comments {rel} (~{file_hits})")
            if not dry_run:
                path.write_text(new, encoding="utf-8")
    print(f"apply-comments: files={changed_files} sites={sites}")
    return changed_files, sites


# --- main --------------------------------------------------------------------


def count_jcode(root: Path) -> int:
    total = 0
    for path in iter_files(
        root, None, (".rs", ".sh", ".ps1", ".py", ".yml", ".yaml", ".md", ".toml")
    ):
        rel = rel_of(path, root)
        if skip_historical(rel) or "scripts/rebrand/" in rel:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        total += len(re.findall(r"\bJCODE_[A-Z0-9_]+", text))
    return total


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=ROOT_DEFAULT)
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--apply-test-setters", action="store_true")
    ap.add_argument("--apply-product-env", action="store_true")
    ap.add_argument("--apply-prod-setters", action="store_true")
    ap.add_argument("--apply-string-keys", action="store_true")
    ap.add_argument("--apply-scripts", action="store_true")
    ap.add_argument("--apply-ci", action="store_true")
    ap.add_argument("--apply-comments", action="store_true")
    ap.add_argument("--apply-all", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--paths", nargs="*")
    args = ap.parse_args()
    root: Path = args.root.resolve()

    if args.apply_all:
        args.apply_test_setters = True
        args.apply_product_env = True
        args.apply_prod_setters = True
        args.apply_string_keys = True
        args.apply_scripts = True
        args.apply_ci = True
        args.apply_comments = True

    any_apply = any(
        [
            args.apply_test_setters,
            args.apply_product_env,
            args.apply_prod_setters,
            args.apply_string_keys,
            args.apply_scripts,
            args.apply_ci,
            args.apply_comments,
        ]
    )
    if not any_apply:
        args.report = True

    before = count_jcode(root)
    print(f"JCODE_* tokens before: {before}")

    totals = Counter()
    rs_files = iter_files(root, args.paths, (".rs",))

    if args.report and not any_apply:
        report(root)
        return 0

    if args.apply_test_setters:
        f, s = apply_test_setters(root, rs_files, args.dry_run)
        totals["test_setter_files"] = f
        totals["test_setter_sites"] = s
    if args.apply_product_env:
        # re-read file list after prior writes
        rs_files = iter_files(root, args.paths, (".rs",))
        f, s = apply_product_env(root, rs_files, args.dry_run)
        totals["product_env_files"] = f
        totals["product_env_sites"] = s
    if args.apply_prod_setters:
        rs_files = iter_files(root, args.paths, (".rs",))
        f, s = apply_prod_setters(root, rs_files, args.dry_run)
        totals["prod_setter_files"] = f
        totals["prod_setter_sites"] = s
    if args.apply_string_keys:
        rs_files = iter_files(root, args.paths, (".rs",))
        f, s = apply_string_keys(root, rs_files, args.dry_run)
        totals["string_key_files"] = f
        totals["string_key_sites"] = s
    if args.apply_scripts:
        f, s = apply_scripts(root, args.dry_run)
        totals["script_files"] = f
        totals["script_sites"] = s
    if args.apply_ci:
        f, s = apply_ci(root, args.dry_run)
        totals["ci_files"] = f
        totals["ci_sites"] = s
    if args.apply_comments:
        f, s = apply_comments(root, args.dry_run)
        totals["comment_files"] = f
        totals["comment_sites"] = s

    after = count_jcode(root)
    print()
    print("=== mop_env summary ===")
    for k, v in totals.items():
        print(f"  {k}: {v}")
    print(f"  JCODE_* tokens before: {before}")
    print(f"  JCODE_* tokens after:  {after}")
    print(f"  JCODE_* tokens removed: {before - after}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
