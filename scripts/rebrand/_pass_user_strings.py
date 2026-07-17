#!/usr/bin/env python3
"""Mechanical user-string + comment rebrand pass (jcode → Next Code / next-code).

Protects:
- LoginProviderTarget::Jcode / ProviderChoice::Jcode / RuntimeProviderId::Jcode
- provider::jcode / mod jcode / pub mod jcode / JcodeProvider
- allowlisted domains jcode.sh / www.jcode.sh
- com.jcode.mobile
- dual-read / legacy / compat symlink mentions
- D1 / wrangler service names in telemetry-worker infra
- changelog / plans (out of scope)

Does NOT rename Rust type idents for the product provider.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

EXACT: list[tuple[str, str]] = [
    ("jcode-provider-service", "next-code-provider-service"),
    ("jcode-secrets", "next-code-secrets"),
    ("jcode-updater", "next-code-updater"),
    ("jcode-embedding/", "next-code-embedding/"),
    ("jcode-edit-bench", "next-code-edit-bench"),
    ("jcode-harness", "next-code-harness"),
    ("jcode-hotkey", "next-code-hotkey"),
    ("jcode-daemon.lock", "next-code-daemon.lock"),
    ("jcode-debug.sock", "next-code-debug.sock"),
    ("jcode.sock", "next-code.sock"),
    ("jcode-bin", "next-code-bin"),
    ("jcode-hooks", "next-code-hooks"),
    ("jcode-app-core", "next-code-app-core"),
    ("jcode-build-meta", "next-code-build-meta"),
    ("jcode-best-of-n", "next-code-best-of-n"),
    ("jcode-tui", "next-code-tui"),
    ("jcode-base", "next-code-base"),
    ("jcode-server", "next-code-server"),
    ("jcode-main", "next-code-main"),
    ("jcode-memlog-cleanup", "next-code-memlog-cleanup"),
    ("jcode-session-bak-prune", "next-code-session-bak-prune"),
    ("jcode-cli-restart-test-home-", "next-code-cli-restart-test-home-"),
    ("jcode-selfdev-test-home-", "next-code-selfdev-test-home-"),
    ("jcode-memory-bench", "next-code-memory-bench"),
    ("jcode-plugins", "next-code-plugins"),
    ("jcode-home", "next-code-home"),
    ("jcode-generated-image", "next-code-generated-image"),
    ("jcode-conf-test", "next-code-conf-test"),
    ("jcode-bak-claim-test", "next-code-bak-claim-test"),
    ("jcode-bak-prune-test", "next-code-bak-prune-test"),
    ("jcode-gallery-golden-", "next-code-gallery-golden-"),
    ("jcode-single-profile-", "next-code-single-profile-"),
    ("jcode-priority-notes", "next-code-priority-notes"),
    ("github.com/1jehuang/jcode", "github.com/quangdang46/next-code"),
    ("1jehuang/jcode", "quangdang46/next-code"),
    ("github.com/jcode", "github.com/quangdang46/next-code"),
    ("~/.jcode", "~/.next-code"),
    ("/.jcode/", "/.next-code/"),
    (r"%LOCALAPPDATA%\\jcode", r"%LOCALAPPDATA%\\next-code"),
    (r"%LOCALAPPDATA%\jcode", r"%LOCALAPPDATA%\next-code"),
    ("LOCALAPPDATA\\jcode", "LOCALAPPDATA\\next-code"),
    ("LOCALAPPDATA/jcode", "LOCALAPPDATA/next-code"),
    ("AppData/Local/jcode", "AppData/Local/next-code"),
    ("AppData\\Local\\jcode", "AppData\\Local\\next-code"),
    ('"jcode/', '"next-code/'),
    ("'jcode/", "'next-code/"),
    ("jcode/{", "next-code/{"),
    ("J-Code", "Next Code"),
    ("J Code", "Next Code"),
    ("JCodeKit", "NextCodeKit"),
    ("JcodeKit", "NextCodeKit"),
    ("JCodeMobile", "NextCodeMobile"),
]

# Protect tokens temporarily (order matters)
PROTECT_RES: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"\bcom\.jcode\.mobile\b"), "@@COM_JCODE_MOBILE@@"),
    (re.compile(r"\bLoginProviderTarget::Jcode\b"), "@@LPT_JCODE@@"),
    (re.compile(r"\bProviderChoice::Jcode\b"), "@@PC_JCODE@@"),
    (re.compile(r"\bRuntimeProviderId::Jcode\b"), "@@RPI_JCODE@@"),
    (re.compile(r"\bResumeTarget::JcodeSession\b"), "@@RT_JCODE_SESSION@@"),
    (re.compile(r"\bJcodeProvider\b"), "@@JCODE_PROVIDER@@"),
    (re.compile(r"\bJcodeSession\b"), "@@JCODE_SESSION_TYPE@@"),
    (re.compile(r"\bprovider::jcode\b"), "@@PROVIDER_JCODE_PATH@@"),
    (re.compile(r"\bpub mod jcode\b"), "@@PUB_MOD_JCODE@@"),
    (re.compile(r"\bmod jcode\b"), "@@MOD_JCODE@@"),
    (re.compile(r"\bSelf::Jcode\b"), "@@SELF_JCODE@@"),
    (re.compile(r"jcode://"), "@@JCODE_SCHEME@@"),
    (re.compile(r"\bresolve_resume_target_to_jcode\b"), "@@FN_RESOLVE_TO_JCODE@@"),
    (re.compile(r"\bfocused_jcode_session\b"), "@@FN_FOCUSED_JCODE@@"),
    (re.compile(r"\blogin_jcode_flow\b"), "@@FN_LOGIN_JCODE@@"),
    (re.compile(r"\bis_jcode_repo\b"), "@@FN_IS_JCODE_REPO@@"),
    (re.compile(r"\bjcode_dir\b"), "@@FN_JCODE_DIR@@"),
    (re.compile(r"\bjcode_compat\b"), "@@JCODE_COMPAT@@"),
    (re.compile(r"\blegacy_jcode\b"), "@@LEGACY_JCODE@@"),
    # D1 / wrangler names
    (re.compile(r"\bjcode-telemetry\b"), "@@JCODE_TELEMETRY@@"),
    (re.compile(r"\btelemetry\.jcode\.sh\b"), "@@TELEMETRY_JCODE_SH@@"),
    # bare domain (after URL protect)
    (re.compile(r"\bjcode\.sh\b"), "@@JCODE_SH@@"),
]

UNPROTECT: list[tuple[str, str]] = [
    ("@@COM_JCODE_MOBILE@@", "com.jcode.mobile"),
    ("@@LPT_JCODE@@", "LoginProviderTarget::Jcode"),
    ("@@PC_JCODE@@", "ProviderChoice::Jcode"),
    ("@@RPI_JCODE@@", "RuntimeProviderId::Jcode"),
    ("@@RT_JCODE_SESSION@@", "ResumeTarget::JcodeSession"),
    ("@@JCODE_PROVIDER@@", "JcodeProvider"),
    ("@@JCODE_SESSION_TYPE@@", "JcodeSession"),
    ("@@PROVIDER_JCODE_PATH@@", "provider::jcode"),
    ("@@PUB_MOD_JCODE@@", "pub mod jcode"),
    ("@@MOD_JCODE@@", "mod jcode"),
    ("@@SELF_JCODE@@", "Self::Jcode"),
    ("@@JCODE_SCHEME@@", "jcode://"),
    ("@@FN_RESOLVE_TO_JCODE@@", "resolve_resume_target_to_jcode"),
    ("@@FN_FOCUSED_JCODE@@", "focused_jcode_session"),
    ("@@FN_LOGIN_JCODE@@", "login_jcode_flow"),
    ("@@FN_IS_JCODE_REPO@@", "is_jcode_repo"),
    ("@@FN_JCODE_DIR@@", "jcode_dir"),
    ("@@JCODE_COMPAT@@", "jcode_compat"),
    ("@@LEGACY_JCODE@@", "legacy_jcode"),
    ("@@JCODE_TELEMETRY@@", "jcode-telemetry"),
    ("@@TELEMETRY_JCODE_SH@@", "telemetry.jcode.sh"),
    ("@@JCODE_SH@@", "jcode.sh"),
    ("@@PROVIDER_ID_JCODE@@", "jcode"),
    ("@@BARE_ENUM_JCODE@@", "Jcode"),
    ("@@JCODE_JSON_KEY@@", '"jcode"'),
    ("@@JCODE_TITLE_D@@", "jcode:d:"),
    ("@@JCODE_TITLE_C@@", "jcode:c:"),
    ("@@JCODE_TITLE_SLASH@@", "jcode/"),
    ("@@JCODE_TITLE_SPACE@@", "jcode "),
]

LINE_SKIP = re.compile(
    r"claude-cli|codex_cli_rs|DO_NOT_TRACK|com\.jcode\.mobile|"
    r"dual-?read|jcode_compat|legacy_jcode|formerly jcode|"
    r"renamed from jcode|upstream jcode|"
    r"compat symlink|still accepts jcode|also accepts jcode|legacy jcode|"
    r"also handles jcode|"
    r"@@",
    re.IGNORECASE,
)

BARE_JCODE = re.compile(r"\bjcode\b")
BARE_JCODE_UPPER = re.compile(r"\bJcode\b")
DOT_JCODE = re.compile(r"(?<![\w.-])\.jcode(?=/|\"|'|\s|$|\\)")
CRATE_JCODE = re.compile(r"\bjcode-([a-z0-9-]+)\b")
# Enum-ish bare Jcode (not already protected)
ENUM_JCODE = re.compile(
    r"(?P<pre>(?:^|[\s\|,\(\[=]))Jcode(?P<post>(?=[\s,\|\)\]=>;:/\"']|$))"
)

SCOPE_REL = [
    "src/cli",
    "src/main.rs",
    "src/lib.rs",
    "src/crash_log.rs",
    "src/customization.rs",
    "src/hooks",
    "src/extension_policy.rs",
    "src/model_routing.rs",
    "src/orchestration_api.rs",
    "src/prefix_cache_stable.rs",
    "src/skill_disable.rs",
    "src/skill_distillation.rs",
    "src/theme.rs",
    "src/bin",
    "crates/next-code-tui",
    "crates/next-code-app-core",
    "crates/next-code-base",
    "scripts",
    "RELEASING.md",
    "OAUTH.md",
    "PARITY.md",
    "entities.json",
    "telemetry-worker",
]

SKIP_NAME = {"Cargo.lock", "Cargo.toml", "_pass_user_strings.py", "rewrite_strings.py"}


def in_scope(path: Path) -> bool:
    try:
        rel = path.relative_to(ROOT).as_posix()
    except ValueError:
        return False
    if path.name in SKIP_NAME:
        return False
    if "/scripts/rebrand/" in f"/{rel}" or rel.startswith("scripts/rebrand/"):
        return False
    if any(x in f"/{rel}" for x in ("/changelog/", "/docs/plans/", "/.git/", "/target/", "/node_modules/")):
        return False
    for s in SCOPE_REL:
        if rel == s or rel.startswith(s.rstrip("/") + "/"):
            return True
    return False


def protect(text: str) -> tuple[str, list[str]]:
    domains: list[str] = []

    def domain_sub(m: re.Match[str]) -> str:
        domains.append(m.group(0))
        return f"@@JCODE_DOMAIN_{len(domains)-1}@@"

    # Full URLs first
    text = re.sub(r"https?://(?:www\.)?jcode\.sh[^\s\"'`)>\]]*", domain_sub, text)

    for rx, token in PROTECT_RES:
        text = rx.sub(token, text)

    # Provider CLI id dual-accept: "jcode" | "next-code"
    text = re.sub(
        r'"jcode"(\s*\|\s*"(?:next-code|next code subscription|jcode subscription)")',
        r'"@@PROVIDER_ID_JCODE@@"\1',
        text,
    )
    text = re.sub(
        r'("(?:next-code|next code subscription|jcode subscription)"\s*\|\s*)"jcode"',
        r'\1"@@PROVIDER_ID_JCODE@@"',
        text,
    )
    # as_arg_value style
    text = re.sub(
        r'(@@SELF_JCODE@@\s*=>\s*)"jcode"',
        r'\1"@@PROVIDER_ID_JCODE@@"',
        text,
    )
    text = re.sub(
        r'(@@PC_JCODE@@[^\n]*=>\s*)"jcode"',
        r'\1"@@PROVIDER_ID_JCODE@@"',
        text,
    )
    # assert_eq!(ProviderChoice::Jcode.as_arg_value(), "jcode");
    text = re.sub(
        r'(as_arg_value\(\),\s*)"jcode"',
        r'\1"@@PROVIDER_ID_JCODE@@"',
        text,
    )
    text = re.sub(
        r'(assert_eq!\([^;]*ProviderChoice::Jcode[^;]*,\s*)"jcode"',
        r'\1"@@PROVIDER_ID_JCODE@@"',
        text,
    )
    # Some("jcode") near provider
    text = re.sub(
        r'(provider_name\s*==\s*|name\(\)\s*==\s*|==\s*)Some\("jcode"\)',
        r'\1Some("@@PROVIDER_ID_JCODE@@")',
        text,
    )
    # ACP JSON key "jcode": { — protocol capability key; protect for now
    text = re.sub(
        r'"jcode"(\s*:\s*\{)',
        r"@@JCODE_JSON_KEY@@\1",
        text,
    )
    # telemetry record_auth_success("jcode-subscription"
    text = re.sub(
        r'"jcode-subscription"',
        '"@@PROVIDER_ID_JCODE@@-subscription"',
        text,
    )
    # join("jcode") path segments that are provider id dirs — protect
    text = re.sub(
        r'\.join\("jcode"\)',
        '.join("@@PROVIDER_ID_JCODE@@")',
        text,
    )
    # Provider catalog / login selection / runtime ids still "jcode"
    text = re.sub(r'\bSome\("jcode"\)', 'Some("@@PROVIDER_ID_JCODE@@")', text)
    text = re.sub(
        r'(as_arg_value\(\),\s*)"jcode"',
        r'\1"@@PROVIDER_ID_JCODE@@"',
        text,
    )
    # Dual-read process/socket name prefixes: keep literal "jcode"
    text = re.sub(
        r'name\.starts_with\("jcode"\)',
        'name.starts_with("@@PROVIDER_ID_JCODE@@")',
        text,
    )
    # Dual-read client title prefixes jcode:d: / jcode:c:
    text = re.sub(r'"jcode:d:"', '"@@JCODE_TITLE_D@@"', text)
    text = re.sub(r'"jcode:c:"', '"@@JCODE_TITLE_C@@"', text)
    # Dual-read window title split_once("jcode/") / ("jcode ")
    text = re.sub(r'split_once\("jcode/"\)', 'split_once("@@JCODE_TITLE_SLASH@@")', text)
    text = re.sub(r'split_once\("jcode "\)', 'split_once("@@JCODE_TITLE_SPACE@@")', text)
    return text, domains


def unprotect(text: str, domains: list[str]) -> str:
    for i, url in enumerate(domains):
        text = text.replace(f"@@JCODE_DOMAIN_{i}@@", url)
    for token, orig in UNPROTECT:
        text = text.replace(token, orig)
    # fix subscription restore
    text = text.replace('"jcode-subscription"', '"jcode-subscription"')
    return text


def rewrite_line(line: str, path: Path) -> str:
    if LINE_SKIP.search(line):
        s = line
        for old, new in EXACT:
            s = s.replace(old, new)
        s = DOT_JCODE.sub(".next-code", s)
        return s

    s = line
    for old, new in EXACT:
        s = s.replace(old, new)

    s = DOT_JCODE.sub(".next-code", s)
    s = CRATE_JCODE.sub(r"next-code-\1", s)

    # Comment/doc crate underscore names
    if path.suffix in {".md", ".js", ".html", ".sh", ".py", ".json"} or re.match(
        r"^\s*(//|//!|/\*|\*|#)", s
    ):
        s = re.sub(r"\bjcode_([a-z0-9_]+)\b", r"next_code_\1", s)

    if path.suffix == ".md":
        s = re.sub(r"\bJCODE_", "NEXT_CODE_", s)

    # Protect remaining enum Jcode then bare-rewrite
    s = ENUM_JCODE.sub(r"\g<pre>@@BARE_ENUM_JCODE@@\g<post>", s)

    s = BARE_JCODE.sub("next-code", s)

    # Product display Jcode → Next Code in comments/docs/strings
    if path.suffix in {".md", ".js", ".html", ".sh", ".py"} or re.match(
        r"^\s*(//|//!|#)", s
    ):
        s = BARE_JCODE_UPPER.sub("Next Code", s)
    else:
        # Rust: rewrite Jcode only inside quotes on the line
        def repl_in_strings(m: re.Match[str]) -> str:
            return m.group(0).replace("Jcode", "Next Code")

        s = re.sub(r'"[^"\n]*"', repl_in_strings, s)
        s = re.sub(r"'[^'\n]*'", repl_in_strings, s)

    return s


def rewrite_text(text: str, path: Path) -> str:
    rel = path.relative_to(ROOT).as_posix()

    # Keep D1 / wrangler infra names entirely
    if rel in {
        "telemetry-worker/wrangler.toml",
        "telemetry-worker/package.json",
        "telemetry-worker/health.sql",
    }:
        return text

    protected, domains = protect(text)
    out = "".join(rewrite_line(line, path) for line in protected.splitlines(keepends=True))
    # preserve trailing newline semantics
    if text.endswith("\n") and not out.endswith("\n"):
        out += "\n"
    result = unprotect(out, domains)

    # Telemetry dashboard: titles + localStorage (safe)
    if rel == "telemetry-worker/src/dashboard.js":
        result = result.replace("jcode · telemetry console", "next-code · telemetry console")
        result = result.replace("<title>next-code · telemetry console</title>", "<title>next-code · telemetry console</title>")
        # after bare rewrite may already be next-code
        result = result.replace("jcode telemetry", "next-code telemetry")
        result = result.replace("// jcode telemetry", "// next-code telemetry")
        result = result.replace("// next-code telemetry console", "// next-code telemetry console")
        result = result.replace("jcode is a terminal", "next-code is a terminal")
        result = result.replace("installed jcode", "installed next-code")
        result = result.replace("ran jcode", "ran next-code")
        result = result.replace("jcode_dash_token", "next_code_dash_token")
        # Design comment
        result = result.replace(
            "Design intent (frontend-design skill): next-code is a terminal coding agent",
            "Design intent (frontend-design skill): next-code is a terminal coding agent",
        )

    if rel == "PARITY.md":
        result = result.replace("# next-code Feature Registry", "# Next Code Feature Registry")
        result = result.replace("# jcode Feature Registry", "# Next Code Feature Registry")
        result = result.replace("| next-code Impl |", "| Next Code Impl |")
        result = result.replace("| jcode Impl |", "| Next Code Impl |")
        result = result.replace("next-code Impl", "Next Code Impl")
        result = result.replace("jcode Impl", "Next Code Impl")

    if rel == "RELEASING.md":
        # Keep intentional jcode → next-code compat symlink wording
        pass

    return result


def main() -> int:
    dry = "--dry-run" in sys.argv
    changed = 0
    scanned = 0
    files: list[Path] = []
    for s in SCOPE_REL:
        p = ROOT / s
        if p.is_file():
            files.append(p)
        elif p.is_dir():
            for f in p.rglob("*"):
                if f.is_file() and f.suffix.lower() in {
                    ".rs",
                    ".md",
                    ".sh",
                    ".py",
                    ".ps1",
                    ".js",
                    ".ts",
                    ".json",
                    ".toml",
                    ".html",
                    ".sql",
                    ".txt",
                    ".yml",
                    ".yaml",
                }:
                    files.append(f)

    for path in sorted(set(files)):
        if not in_scope(path):
            continue
        try:
            original = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        scanned += 1
        updated = rewrite_text(original, path)
        if updated != original:
            changed += 1
            rel = path.relative_to(ROOT)
            print(f"{'DRY ' if dry else 'WRITE'} {rel}")
            if not dry:
                path.write_text(updated, encoding="utf-8")

    print(f"pass_user_strings: scanned={scanned} changed={changed} dry={dry}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
