#!/usr/bin/env python3
"""Multi-pass string rebrand for non-Cargo sources (jcode → next-code).

Longest / most specific tokens first. Skips paths listed in the contract
allowlist (changelog, plans, binaries, Cargo.lock, target, .git).

Cargo.toml is intentionally left to rewrite_cargo.py (this script skips it).
Rust idents are left to rewrite_rust_idents.py; this still rewrites string
literals and comments inside .rs files for product strings.

Swift/iOS: rewrite display strings; KEEP com.jcode.mobile bundle identifiers.
URL schemes are NOT auto-rewritten to force dual registration in a later PR.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Ordered passes — longest / most specific first.
# Each entry: (old, new) exact string replace, OR (compiled_re, repl, True)
PASSES: list[tuple] = [
    ("jcode-provider-service", "next-code-provider-service"),
    ("jcode-secrets", "next-code-secrets"),
    ("jcode-updater", "next-code-updater"),
    ("jcode-embedding/", "next-code-embedding/"),
    ("jcode-edit-bench", "next-code-edit-bench"),
    ("jcode-harness", "next-code-harness"),
    ("jcode-hotkey", "next-code-hotkey"),
    ("jcode-daemon.lock", "next-code-daemon.lock"),
    ("jcode.sock", "next-code.sock"),
    ("libjcode_base", "libnext_code_base"),
    ("homebrew-jcode", "homebrew-next-code"),
    ("jcode-bin", "next-code-bin"),
    ("JCODE_", "NEXT_CODE_"),
    ("jcode_dir", "next_code_dir"),
    ("is_jcode_repo", "is_next_code_repo"),
    ("~/.jcode", "~/.next-code"),
    ("/.jcode/", "/.next-code/"),
    # Windows LOCALAPPDATA forms
    (r"%LOCALAPPDATA%\\jcode", r"%LOCALAPPDATA%\\next-code"),
    (r"%LOCALAPPDATA%\jcode", r"%LOCALAPPDATA%\next-code"),
    ("LOCALAPPDATA\\jcode", "LOCALAPPDATA\\next-code"),
    ("LOCALAPPDATA/jcode", "LOCALAPPDATA/next-code"),
    ("/usr/lib/jcode", "/usr/lib/next-code"),
    # User-Agent prefix
    ('"jcode/', '"next-code/'),
    ("'jcode/", "'next-code/"),
    ("jcode/{", "next-code/{"),  # format strings jcode/{ver}
    # URL scheme (product strings). Dual-read registration is a separate code change.
    ("jcode://", "nextcode://"),
    # Display / clap
    ("J-Code", "Next Code"),
    ("J Code", "Next Code"),
    # Live GitHub product URLs (not changelog — those paths are skipped)
    ("1jehuang/jcode", "quangdang46/next-code"),
    # Type-like names (Swift / docs). Do NOT touch com.jcode.mobile.
    ("JCodeKit", "NextCodeKit"),
    ("JcodeKit", "NextCodeKit"),
    ("JCode", "NextCode"),
    ("Jcode", "NextCode"),
]

# Word-boundary bare `jcode` → `next-code` (last pass on text-ish files).
BARE_JCODE = re.compile(r"\bjcode\b")

# Project-dir form `.jcode/` or `.jcode` at end / before quote — careful.
DOT_JCODE_DIR = re.compile(r"(?<![\w.-])\.jcode(?=/|\"|'|\s|$|\\)")

SKIP_DIR_NAMES = {
    ".git",
    "target",
    "node_modules",
    "changelog",
    "plans",
}

SKIP_FILE_NAMES = {
    "Cargo.lock",
    "Cargo.toml",  # rewrite_cargo.py owns these
    "REBRAND_ALLOWLIST.md",
    "REBRAND_CONTRACT.md",
    "REBRAND_IMPLEMENTATION_PLAN.md",
    "REBRAND_JCODE_TO_NEXT_CODE_AUDIT.md",
}

SKIP_SUFFIXES = {
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".ico",
    ".icns",
    ".pdf",
    ".woff",
    ".woff2",
    ".ttf",
    ".otf",
    ".bin",
    ".wasm",
    ".sqlite",
    ".db",
    ".lock",
    ".mp4",
    ".zip",
    ".gz",
    ".tgz",
    ".xz",
}

# Path substrings that mean "leave this file alone"
SKIP_PATH_PARTS = (
    "/changelog/",
    "/docs/plans/",
    "/.git/",
    "/target/",
)

# Filename patterns: docs/*_PLAN.md
PLAN_NAME = re.compile(r".*_PLAN\.md$", re.IGNORECASE)

# Lines / files where bare jcode must stay (third-party, allowlist)
LINE_SKIP_BARE = re.compile(
    r"claude-cli|codex_cli_rs|DO_NOT_TRACK|com\.jcode\.mobile|"
    r"(?:^|[\s\"'`])\.claude/|(?:^|[\s\"'`])\.codex/|"
    r"dual-?read|jcode_compat|legacy_jcode|formerly jcode|"
    r"renamed from jcode|upstream jcode|jcode\.sh|\.jcode\.sh",
    re.IGNORECASE,
)

# Never rewrite com.jcode.mobile (restore if a pass touched it — defensive)
BUNDLE_ID = "com.jcode.mobile"


def should_skip_path(path: Path, root: Path) -> bool:
    try:
        rel = path.relative_to(root)
    except ValueError:
        return True
    parts = set(rel.parts)
    if parts & SKIP_DIR_NAMES:
        return True
    rel_s = "/" + rel.as_posix()
    for frag in SKIP_PATH_PARTS:
        if frag in rel_s or rel_s.startswith(frag.lstrip("/")):
            return True
    if path.name in SKIP_FILE_NAMES:
        return True
    if PLAN_NAME.match(path.name):
        return True
    if path.suffix.lower() in SKIP_SUFFIXES:
        return True
    # Skip this rebrand tooling directory's docs references? Allow rewriting
    # scripts/rebrand sources themselves only for consistency — they shouldn't
    # contain product jcode paths except comments. Process them.
    return False


def apply_passes(text: str) -> str:
    # Protect bundle id from any accidental rewrite
    sentinel = "@@COM_JCODE_MOBILE@@"
    text = text.replace(BUNDLE_ID, sentinel)

    for item in PASSES:
        old, new = item[0], item[1]
        text = text.replace(old, new)

    # .jcode/ project dir (after /.jcode/ pass)
    text = DOT_JCODE_DIR.sub(".next-code", text)

    text = text.replace(sentinel, BUNDLE_ID)
    return text


def apply_bare_jcode(text: str, path: Path) -> str:
    """Word-boundary jcode → next-code, skipping allowlisted lines."""
    # Skip process_title special handling is not needed if bare replace works.
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    for line in lines:
        if LINE_SKIP_BARE.search(line):
            out.append(line)
            continue
        # Protect bundle id again
        if BUNDLE_ID in line or "com.jcode." in line:
            out.append(line)
            continue
        out.append(BARE_JCODE.sub("next-code", line))
    return "".join(out)


def is_probably_text(path: Path) -> bool:
    # Extension heuristic
    text_ext = {
        ".rs",
        ".toml",
        ".md",
        ".txt",
        ".sh",
        ".bash",
        ".zsh",
        ".py",
        ".ps1",
        ".swift",
        ".m",
        ".h",
        ".plist",
        ".json",
        ".yml",
        ".yaml",
        ".toml",
        ".rb",
        ".js",
        ".ts",
        ".tsx",
        ".jsx",
        ".css",
        ".html",
        ".svg",
        ".service",
        ".desktop",
        ".in",
        ".cmake",
        ".mk",
        ".dockerfile",
        ".gitignore",
        ".gitattributes",
        ".editorconfig",
        ".env",
        ".example",
        ".template",
        ".fish",
        ".nu",
    }
    if path.suffix.lower() in text_ext:
        return True
    if path.name in {
        "Dockerfile",
        "Makefile",
        "Justfile",
        "LICENSE",
        "AGENTS.md",
        "README",
        "README.md",
        "install.sh",
        "install.ps1",
    }:
        return True
    # extensionless scripts
    if path.suffix == "" and path.is_file():
        return True
    return False


def rewrite_file(path: Path, root: Path, bare: bool) -> str | None:
    try:
        original = path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, OSError):
        return None
    updated = apply_passes(original)
    if bare and is_probably_text(path):
        updated = apply_bare_jcode(updated, path)
    if updated != original:
        return updated
    return None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
    )
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--verbose", "-v", action="store_true")
    ap.add_argument(
        "--no-bare",
        action="store_true",
        help="Skip final word-boundary \\bjcode\\b pass",
    )
    ap.add_argument(
        "--paths",
        nargs="*",
        help="Optional relative paths to limit scope",
    )
    args = ap.parse_args()
    root: Path = args.root.resolve()
    bare = not args.no_bare

    if args.paths:
        candidates = [root / p for p in args.paths]
    else:
        candidates = [p for p in root.rglob("*") if p.is_file()]

    scanned = 0
    changed = 0
    for path in sorted(candidates):
        if not path.is_file():
            continue
        if should_skip_path(path, root):
            continue
        if not is_probably_text(path) and path.suffix.lower() not in {
            ".rs",
            ".md",
            ".sh",
            ".py",
            ".swift",
            ".plist",
            ".json",
            ".yml",
            ".yaml",
            ".toml",
            ".ps1",
            ".service",
            ".rb",
        }:
            continue
        scanned += 1
        new_text = rewrite_file(path, root, bare=bare)
        if new_text is not None:
            changed += 1
            rel = path.relative_to(root)
            if args.verbose or args.dry_run:
                print(f"{'DRY ' if args.dry_run else 'WRITE'} {rel}")
            if not args.dry_run:
                path.write_text(new_text, encoding="utf-8")

    print(
        f"rewrite_strings: scanned={scanned} changed={changed} "
        f"bare={bare} dry_run={args.dry_run}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
