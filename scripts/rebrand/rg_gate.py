#!/usr/bin/env python3
"""Fail if unexpected residual jcode brand hits remain.

Scans the tree case-insensitively for 'jcode', excludes allowlisted paths
and line patterns from docs/REBRAND_ALLOWLIST.md (hardcoded mirrors below).

Exit codes:
  0 — clean (only allowlisted residuals)
  1 — unexpected hits remain
  2 — usage / IO error
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

NEEDLE = re.compile(r"jcode", re.IGNORECASE)

SKIP_DIR_NAMES = {
    ".git",
    "target",
    "node_modules",
    "changelog",
    "plans",
}

SKIP_FILE_NAMES = {
    "Cargo.lock",
    "REBRAND_ALLOWLIST.md",
    "REBRAND_CONTRACT.md",
    "REBRAND_IMPLEMENTATION_PLAN.md",
    "REBRAND_JCODE_TO_NEXT_CODE_AUDIT.md",
    "REBRAND_STATUS.md",
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
    ".o",
    ".a",
    ".rlib",
    ".so",
    ".dylib",
}

PLAN_NAME = re.compile(r".*_PLAN\.md$", re.IGNORECASE)

# Line allowlist (from REBRAND_ALLOWLIST.md §9)
LINE_ALLOW = re.compile(
    r"(?i)("
    r"dual-?read|"
    r"compat(?:ibility)?[:\s].*jcode|"
    r"legacy_jcode|jcode_compat|old_jcode_|"
    r"com\.jcode\.mobile|"
    r"claude-cli|"
    r"codex_cli_rs|"
    r"DO_NOT_TRACK|"
    r"(?:^|[\s\"'`])\.claude/|"
    r"(?:^|[\s\"'`])\.codex/|"
    r"(?:^|[\s\"'`])\.cursor/|"
    r"1jehuang/jcode|"
    r"(?:^|[^\w])jbench(?:[^\w]|$)|"
    r"[\w.-]*jcode\.sh\b|"
    r"\bjcode\.sh\b|"
    r"formerly jcode|"
    r"renamed from jcode|"
    r"upstream jcode|"
    r"origin-sync.*jcode|"
    r"REBRAND_|"
    # intentional dual-read / migration shims (compat window)
    r"migrated-from-jcode|"
    r"\bjcode_dir\b|"
    r"jcode\s*→\s*next-code|"
    r"jcode\s*->\s*next-code|"
    r"legacy\s+[`']?JCODE_|"
    r"legacy\s+[`']?\.?jcode|"
    r"fallback\s+to\s+[`']?JCODE_|"
    r"falls?\s+back\s+to\s+[`']?JCODE_|"
    r"falls?\s+back\s+to\s+[`']?\.?jcode|"
    r"PRODUCT_DIR_CANDIDATES.*\.jcode|"
    r"PROJECT_DIR_CANDIDATES"
    r")"
)

# Path allowlist: rebrand tooling may still mention jcode as the *old* name
PATH_ALLOW_SUBSTRINGS = (
    "/scripts/rebrand/",
    "/docs/REBRAND_",
)


def should_skip_path(path: Path, root: Path) -> bool:
    try:
        rel = path.relative_to(root)
    except ValueError:
        return True
    parts = rel.parts
    if any(p in SKIP_DIR_NAMES for p in parts):
        return True
    if path.name in SKIP_FILE_NAMES:
        return True
    if PLAN_NAME.match(path.name):
        return True
    if path.suffix.lower() in SKIP_SUFFIXES:
        return True
    # docs/*_PLAN already; docs/plans via SKIP_DIR_NAMES
    rel_s = rel.as_posix()
    if rel_s.startswith("changelog/") or "/changelog/" in rel_s:
        return True
    if rel_s.startswith("docs/plans/") or "/docs/plans/" in rel_s:
        return True
    return False


def path_allowlisted(rel_posix: str) -> bool:
    for frag in PATH_ALLOW_SUBSTRINGS:
        if frag.strip("/") in rel_posix or rel_posix.startswith(frag.lstrip("/")):
            return True
        if frag in f"/{rel_posix}":
            return True
    # Explicit: scripts/rebrand/*
    if rel_posix.startswith("scripts/rebrand/"):
        return True
    if rel_posix.startswith("docs/REBRAND_"):
        return True
    return False


def is_binary_snippet(data: bytes) -> bool:
    return b"\x00" in data[:8192]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
    )
    ap.add_argument(
        "--max-print",
        type=int,
        default=50,
        help="Max unexpected hit lines to print",
    )
    ap.add_argument(
        "--strict-tooling",
        action="store_true",
        help="Do not allowlist scripts/rebrand or REBRAND_* docs",
    )
    args = ap.parse_args()
    root: Path = args.root.resolve()

    unexpected: list[tuple[str, int, str]] = []
    allow_count = 0
    skip_files = 0
    scanned_files = 0
    hit_files: Counter[str] = Counter()

    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if should_skip_path(path, root):
            skip_files += 1
            continue
        rel = path.relative_to(root).as_posix()
        try:
            raw = path.read_bytes()
        except OSError:
            continue
        if is_binary_snippet(raw):
            skip_files += 1
            continue
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            try:
                text = raw.decode("utf-8", errors="replace")
            except Exception:
                skip_files += 1
                continue

        scanned_files += 1
        path_ok = (not args.strict_tooling) and path_allowlisted(rel)

        for lineno, line in enumerate(text.splitlines(), 1):
            if not NEEDLE.search(line):
                continue
            if path_ok or LINE_ALLOW.search(line):
                allow_count += 1
                continue
            unexpected.append((rel, lineno, line.rstrip()[:240]))
            hit_files[rel] += 1

    print("rg_gate summary")
    print(f"  root:            {root}")
    print(f"  files_scanned:   {scanned_files}")
    print(f"  files_skipped:   {skip_files}")
    print(f"  allowlisted_hits:{allow_count}")
    print(f"  unexpected_hits: {len(unexpected)}")
    print(f"  unexpected_files:{len(hit_files)}")

    if unexpected:
        print("\nUnexpected hits (first %d):" % min(args.max_print, len(unexpected)))
        for rel, lineno, line in unexpected[: args.max_print]:
            print(f"  {rel}:{lineno}: {line}")
        if len(unexpected) > args.max_print:
            print(f"  ... {len(unexpected) - args.max_print} more")
        print("\nTop files:")
        for rel, n in hit_files.most_common(20):
            print(f"  {n:5d}  {rel}")
        return 1

    print("\nOK — no unexpected jcode residuals")
    return 0


if __name__ == "__main__":
    sys.exit(main())
