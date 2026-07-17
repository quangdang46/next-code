#!/usr/bin/env python3
"""Safer env-token rebrand helpers for jcode → next-code mop-up.

Focus: JCODE_* / NEXT_CODE_* call sites in Rust. Does NOT touch allowlisted
domains, com.jcode.mobile, or third-party UAs.

Modes (combine as needed):

  --report                 Print classification (default if no --apply*)
  --apply-test-setters     In test modules / *test* paths:
                             set_var/remove_var("JCODE_X") → NEXT_CODE_X
                             and matching var/var_os("JCODE_X") in same files
                           Skips dual-read unit tests that must keep JCODE_
  --apply-product-env      Production simple reads:
                             std::env::var("JCODE_X")     → product_env("X")
                             std::env::var_os("JCODE_X")  → product_env_os("X")
                           Only when the key is a pure JCODE_<SUFFIX> suffix.
                           Leaves a report line when conversion is skipped.
  --dry-run                With apply modes: print paths that would change
  --paths P [P ...]        Limit to relative paths

Never rewrites:
  - com.jcode.mobile
  - *.jcode.sh / jcode.sh
  - crates/next-code-core/src/env.rs product_env implementation body
    (except dual-read tests keep JCODE_ under --apply-test-setters skip)
  - Lines already containing dual-read / legacy_jcode / product_env fallbacks
    that intentionally name JCODE_
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

# --- patterns ----------------------------------------------------------------

VAR_CALL = re.compile(
    r"""(?P<full>(?P<std>std::)?env::var(?P<os>_os)?\(\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))"\s*\))"""
)
SET_CALL = re.compile(
    r"""(?P<full>(?P<qual>(?:crate::env::|next_code_core::env::|super::|self::)?)(?P<fn>set_var|remove_var)\(\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))")"""
)

# Broader set_var that may use std::env::set_var
SET_CALL_STD = re.compile(
    r"""(?P<full>std::env::(?P<fn>set_var|remove_var)\(\s*"(?P<key>JCODE_(?P<suffix>[A-Z0-9_]+))")"""
)

TEST_PATH_HINT = re.compile(
    r"(?:^|/)tests?/|/test_|_tests\.rs$|_test\.rs$|/benches/",
    re.IGNORECASE,
)

# Files that must keep explicit JCODE_ for dual-read assertions
KEEP_JCODE_SETTER_PATHS = {
    "crates/next-code-core/src/env.rs",
    "crates/next-code-storage/src/lib.rs",
    "crates/next-code-storage/src/active_pids.rs",
}

# Do not rewrite product_env implementation reads of JCODE_
SKIP_PRODUCT_ENV_PATHS = {
    "crates/next-code-core/src/env.rs",
}

LINE_SKIP = re.compile(
    r"(?i)("
    r"dual-?read|"
    r"legacy_jcode|jcode_compat|old_jcode_|"
    r"com\.jcode\.mobile|"
    r"claude-cli|codex_cli_rs|"
    r"[\w.-]*jcode\.sh\b|"
    r"falls?\s+back\s+to\s+[`']?JCODE_|"
    r"fallback\s+to\s+[`']?JCODE_|"
    r"product_env|"  # already using helper — leave nearby JCODE_ alone if any
    r"formerly jcode|renamed from jcode"
    r")"
)

CFG_TEST = re.compile(r"#\[cfg\(test\)\]")
MOD_TESTS = re.compile(r"\bmod\s+tests\b")


def rel_of(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def is_testish_path(rel: str) -> bool:
    return bool(TEST_PATH_HINT.search(rel))


def line_in_test_module(lines: list[str], lineno: int) -> bool:
    """Heuristic: any #[cfg(test)] or `mod tests` in the 200 lines above."""
    start = max(0, lineno - 200)
    window = "".join(lines[start:lineno])
    return bool(CFG_TEST.search(window) or MOD_TESTS.search(window))


def should_skip_line(line: str) -> bool:
    if "com.jcode.mobile" in line:
        return True
    if "jcode.sh" in line:
        return True
    if LINE_SKIP.search(line):
        return True
    return False


def iter_rs_files(root: Path, paths: list[str] | None) -> list[Path]:
    if paths:
        out = []
        for p in paths:
            fp = (root / p).resolve()
            if fp.is_file() and fp.suffix == ".rs":
                out.append(fp)
            elif fp.is_dir():
                out.extend(sorted(fp.rglob("*.rs")))
        return out
    return sorted(
        p
        for p in root.rglob("*.rs")
        if p.is_file()
        and "target" not in p.parts
        and ".git" not in p.parts
    )


# --- report ------------------------------------------------------------------

def report(root: Path, files: list[Path]) -> int:
    buckets = {
        "A_prod_read": [],  # (rel, line, key, snippet)
        "A_test_read": [],
        "B_test_set": [],
        "B_prod_set": [],
        "skipped": [],
    }
    suffix_prod = Counter()
    for path in files:
        rel = rel_of(path, root)
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        lines = text.splitlines(keepends=True)
        for i, line in enumerate(lines, 1):
            if should_skip_line(line) and "JCODE_" in line:
                # still count allowlist-ish
                pass
            for m in VAR_CALL.finditer(line):
                key = m.group("key")
                rec = (rel, i, key, line.strip()[:100])
                testish = is_testish_path(rel) or line_in_test_module(lines, i)
                if testish:
                    buckets["A_test_read"].append(rec)
                else:
                    buckets["A_prod_read"].append(rec)
                    suffix_prod[m.group("suffix")] += 1
            for rx in (SET_CALL, SET_CALL_STD):
                for m in rx.finditer(line):
                    key = m.group("key")
                    rec = (rel, i, m.group("fn"), key, line.strip()[:100])
                    testish = is_testish_path(rel) or line_in_test_module(lines, i)
                    if testish:
                        buckets["B_test_set"].append(rec)
                    else:
                        buckets["B_prod_set"].append(rec)

    def file_counts(recs, key_idx=0):
        c = Counter(r[key_idx] for r in recs)
        return c

    print("=== rewrite_env_tokens report ===")
    print(f"root: {root}")
    print(f"A production env reads: {len(buckets['A_prod_read'])} "
          f"in {len(file_counts(buckets['A_prod_read']))} files")
    print(f"A test env reads:       {len(buckets['A_test_read'])} "
          f"in {len(file_counts(buckets['A_test_read']))} files")
    print(f"B test set/remove:      {len(buckets['B_test_set'])} "
          f"in {len(file_counts(buckets['B_test_set']))} files")
    print(f"B prod set/remove:      {len(buckets['B_prod_set'])} "
          f"in {len(file_counts(buckets['B_prod_set']))} files")
    print()
    print("--- top A production files ---")
    for f, n in file_counts(buckets["A_prod_read"]).most_common(20):
        print(f"  {n:4d}  {f}")
    print()
    print("--- top production suffixes ---")
    for s, n in suffix_prod.most_common(25):
        print(f"  {n:4d}  JCODE_{s}  → product_env(\"{s}\")")
    print()
    print("--- top B test setter files ---")
    for f, n in file_counts(buckets["B_test_set"]).most_common(15):
        print(f"  {n:4d}  {f}")
    print()
    print("--- B production setter files (need dual-set review) ---")
    for f, n in sorted(file_counts(buckets["B_prod_set"]).items(), key=lambda x: -x[1]):
        print(f"  {n:4d}  {f}")
    return 0


# --- apply test setters ------------------------------------------------------

def apply_test_setters(root: Path, files: list[Path], dry_run: bool) -> int:
    """Rewrite JCODE_ → NEXT_CODE_ in set_var/remove_var/var in test contexts."""
    changed_files = 0
    changed_sites = 0

    for path in files:
        rel = rel_of(path, root)
        if rel in KEEP_JCODE_SETTER_PATHS:
            continue
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
            testish = is_testish_path(rel) or line_in_test_module(lines, i)
            if not testish:
                out.append(line)
                continue
            new = line
            # Only touch JCODE_ tokens inside set_var/remove_var/var strings
            def repl_set(m: re.Match[str]) -> str:
                nonlocal file_hits
                file_hits += 1
                return m.group(0).replace(
                    f'"JCODE_{m.group("suffix")}"',
                    f'"NEXT_CODE_{m.group("suffix")}"',
                    1,
                )

            new = SET_CALL.sub(repl_set, new)
            new = SET_CALL_STD.sub(repl_set, new)

            def repl_var(m: re.Match[str]) -> str:
                nonlocal file_hits
                file_hits += 1
                return m.group(0).replace(
                    f'"JCODE_{m.group("suffix")}"',
                    f'"NEXT_CODE_{m.group("suffix")}"',
                    1,
                )

            new = VAR_CALL.sub(repl_var, new)
            out.append(new)

        if file_hits:
            changed_files += 1
            changed_sites += file_hits
            new_text = "".join(out)
            print(f"{'DRY ' if dry_run else 'WRITE'} {rel}  ({file_hits} sites)")
            if not dry_run and new_text != text:
                path.write_text(new_text, encoding="utf-8")

    print(
        f"apply-test-setters: files={changed_files} sites={changed_sites} "
        f"dry_run={dry_run}"
    )
    return 0


# --- apply product_env -------------------------------------------------------

# Match a whole call used as expression; keep surrounding code.
# We only convert when the call is a simple env::var("JCODE_X") form.
SIMPLE_VAR = re.compile(
    r"""(?P<full>(?P<prefix>std::)?env::var(?P<os>_os)?\(\s*"JCODE_(?P<suffix>[A-Z0-9_]+)"\s*\))"""
)

# Patterns we refuse to auto-convert (report only)
SKIP_CONVERT_HINT = re.compile(
    r"or_else|unwrap_or|product_env|product_var|JCODE_.*NEXT_CODE|NEXT_CODE_.*JCODE"
)


def apply_product_env(root: Path, files: list[Path], dry_run: bool) -> int:
    """Convert simple production env::var("JCODE_X") to product_env("X")."""
    changed_files = 0
    converted = 0
    skipped: list[tuple[str, int, str]] = []
    needs_import: dict[str, int] = defaultdict(int)

    for path in files:
        rel = rel_of(path, root)
        if rel in SKIP_PRODUCT_ENV_PATHS:
            continue
        if is_testish_path(rel):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        lines = text.splitlines(keepends=True)
        out: list[str] = []
        file_conv = 0
        for i, line in enumerate(lines, 1):
            if should_skip_line(line):
                out.append(line)
                continue
            if line_in_test_module(lines, i):
                out.append(line)
                continue
            if SKIP_CONVERT_HINT.search(line):
                if "JCODE_" in line and "env::var" in line:
                    skipped.append((rel, i, "complex_or_dual " + line.strip()[:80]))
                out.append(line)
                continue

            def repl(m: re.Match[str]) -> str:
                nonlocal file_conv
                suffix = m.group("suffix")
                os_suf = m.group("os") or ""
                file_conv += 1
                if os_suf:
                    return f'product_env_os("{suffix}")'
                # var returns Result — product_env same
                return f'product_env("{suffix}")'

            new = SIMPLE_VAR.sub(repl, line)
            if new != line:
                # product_env_os returns Option, var_os returns Option — OK
                # product_env returns Result — same as var — OK
                # But: product_env_os(...) vs env::var_os — call sites using
                # .is_some() / .is_none() / .as_deref() still work on Option.
                out.append(new)
            else:
                out.append(line)

        if file_conv:
            changed_files += 1
            converted += file_conv
            needs_import[rel] += file_conv
            new_text = "".join(out)
            print(f"{'DRY ' if dry_run else 'WRITE'} {rel}  ({file_conv} conversions)")
            if not dry_run and new_text != text:
                path.write_text(new_text, encoding="utf-8")

    print(
        f"apply-product-env: files={changed_files} conversions={converted} "
        f"dry_run={dry_run}"
    )
    if needs_import:
        print()
        print(
            "NOTE: converted files need `use next_code_core::env::{product_env, product_env_os};` "
            "(or crate-local re-export). Add imports manually or via a follow-up pass."
        )
        for rel, n in sorted(needs_import.items(), key=lambda x: -x[1])[:30]:
            print(f"  import? {n:3d}  {rel}")
    if skipped:
        print()
        print(f"--- skipped complex reads ({len(skipped)}) sample ---")
        for rel, i, why in skipped[:40]:
            print(f"  {rel}:{i}: {why}")
    return 0


# --- main --------------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
    )
    ap.add_argument("--report", action="store_true", help="Print classification")
    ap.add_argument(
        "--apply-test-setters",
        action="store_true",
        help="Rewrite JCODE_→NEXT_CODE_ in test set_var/remove_var/var",
    )
    ap.add_argument(
        "--apply-product-env",
        action="store_true",
        help="Convert simple production env::var(JCODE_) to product_env",
    )
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--paths", nargs="*", help="Limit to relative paths")
    args = ap.parse_args()
    root: Path = args.root.resolve()
    files = iter_rs_files(root, args.paths)

    if not args.apply_test_setters and not args.apply_product_env:
        args.report = True

    rc = 0
    if args.report:
        rc = report(root, files) or rc
    if args.apply_test_setters:
        rc = apply_test_setters(root, files, args.dry_run) or rc
    if args.apply_product_env:
        rc = apply_product_env(root, files, args.dry_run) or rc
    return rc


if __name__ == "__main__":
    sys.exit(main())
