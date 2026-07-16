#!/usr/bin/env python3
"""Rewrite Cargo.toml brand tokens: jcode → next-code / next_code.

Walks all Cargo.toml under the repo root (skips target/). Longer tokens
first. Mechanical — intended for Phase 1 after rename_crates.sh.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

# (old, new) — longest / most specific first
REPLACEMENTS: list[tuple[str, str]] = [
    # profile package stanzas
    ('package."jcode-tui-anim"', 'package."next-code-tui-anim"'),
    ("package.'jcode-tui-anim'", "package.'next-code-tui-anim'"),
    # feature dep: strings
    ("dep:jcode-", "dep:next-code-"),
    # path deps (quoted forms)
    ('path = "crates/jcode-', 'path = "crates/next-code-'),
    ("path = 'crates/jcode-", "path = 'crates/next-code-"),
    ('path = "evals/jcode-', 'path = "evals/next-code-'),
    ("path = 'evals/jcode-", "path = 'evals/next-code-"),
    # workspace member path fragments
    ('"crates/jcode-', '"crates/next-code-'),
    ("'crates/jcode-", "'crates/next-code-"),
    ('"evals/jcode-', '"evals/next-code-'),
    ("'evals/jcode-", "'evals/next-code-"),
    # feature table keys / paths like jcode-tui/
    ("jcode-tui/", "next-code-tui/"),
    # binary names (longer first)
    ('name = "jcode-edit-bench"', 'name = "next-code-edit-bench"'),
    ("name = 'jcode-edit-bench'", "name = 'next-code-edit-bench'"),
    ('name = "jcode-harness"', 'name = "next-code-harness"'),
    ("name = 'jcode-harness'", "name = 'next-code-harness'"),
    # lib names (underscore)
    ('name = "jcode_', 'name = "next_code_'),
    ("name = 'jcode_", "name = 'next_code_"),
    # package / bin name = "jcode-foo"
    ('name = "jcode-', 'name = "next-code-'),
    ("name = 'jcode-", "name = 'next-code-"),
    # root package / lib / bin exact "jcode"
    ('name = "jcode"', 'name = "next-code"'),
    ("name = 'jcode'", "name = 'next-code'"),
    # [lib] name = "jcode" already covered; underscore root lib:
    # if someone used name = "jcode" for lib, next-code is wrong for rustc —
    # Cargo accepts hyphen package + underscore lib. Explicit lib override:
    # After hyphen package renames, fix lib name back to next_code.
]

# Second pass: [lib] and any remaining exact lib idents that must be underscores.
LIB_FIXUPS: list[tuple[str, str]] = [
    # If we turned lib name into next-code, fix to next_code
    ('[lib]\nname = "next-code"', '[lib]\nname = "next_code"'),
    ("[lib]\nname = 'next-code'", "[lib]\nname = 'next_code'"),
    # standalone lib name lines that might not sit right after [lib] —
    # rewrite hyphenated next-code_* is wrong; only exact next-code as lib:
]

# Package-key dependency table keys: jcode-foo = { path = ... }
# Also bare keys in [dependencies]
KEY_REPLACEMENTS: list[tuple[str, str]] = [
    ("jcode-edit-bench", "next-code-edit-bench"),
    ("jcode-harness", "next-code-harness"),
    ("jcode-", "next-code-"),
]


def should_skip(path: Path, root: Path) -> bool:
    rel = path.relative_to(root)
    parts = rel.parts
    if "target" in parts or ".git" in parts:
        return True
    return False


def rewrite_text(text: str) -> str:
    out = text
    for old, new in REPLACEMENTS:
        out = out.replace(old, new)

    # Dependency / package keys: jcode-foo → next-code-foo (remaining)
    # Avoid touching comments about third parties if easy — mechanical is OK.
    for old, new in KEY_REPLACEMENTS:
        out = out.replace(old, new)

    # [lib] name must be a valid Rust ident: next-code → next_code when it is a lib name.
    # Heuristic: lines that look like name = "next-code" immediately under [lib],
    # or name = "next-code_foo" shouldn't happen. Fix root lib only.
    lines = out.splitlines(keepends=True)
    fixed: list[str] = []
    in_lib = False
    in_bin = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_lib = stripped == "[lib]" or stripped.startswith("[lib.")
            in_bin = stripped.startswith("[[bin]]") or stripped.startswith("[bin")
        if in_lib and ('name = "next-code"' in line or "name = 'next-code'" in line):
            line = line.replace('name = "next-code"', 'name = "next_code"')
            line = line.replace("name = 'next-code'", "name = 'next_code'")
        # lib names with underscores already handled via jcode_ → next_code_
        # Ensure name = "next-code_foo" never appears — convert hyphens after next-code_ wrong.
        if in_lib:
            # name = "jcode_foo" already became next_code_foo via KEY? KEY uses jcode-
            # Explicit: name = "jcode_ → next_code_ already in REPLACEMENTS
            pass
        # bins keep hyphens
        if in_bin:
            pass
        fixed.append(line)
    out = "".join(fixed)

    # Remaining jcode_ lib idents in name = "..."
    out = out.replace('name = "jcode_', 'name = "next_code_')
    out = out.replace("name = 'jcode_", "name = 'next_code_")

    # Catch-all: any leftover path fragment crates/jcode
    out = out.replace("crates/jcode-", "crates/next-code-")
    out = out.replace("evals/jcode-", "evals/next-code-")

    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
        help="Repo root (default: inferred)",
    )
    ap.add_argument("--dry-run", action="store_true", help="Report only")
    ap.add_argument("--verbose", "-v", action="store_true")
    args = ap.parse_args()
    root: Path = args.root.resolve()

    tomls = sorted(root.rglob("Cargo.toml"))
    changed = 0
    scanned = 0
    for path in tomls:
        if should_skip(path, root):
            continue
        scanned += 1
        original = path.read_text(encoding="utf-8")
        updated = rewrite_text(original)
        if updated != original:
            changed += 1
            if args.verbose or args.dry_run:
                print(f"{'DRY ' if args.dry_run else 'WRITE'} {path.relative_to(root)}")
            if not args.dry_run:
                path.write_text(updated, encoding="utf-8")

    print(f"rewrite_cargo: scanned={scanned} changed={changed} dry_run={args.dry_run}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
