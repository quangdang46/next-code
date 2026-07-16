#!/usr/bin/env python3
"""Token-aware Rust ident rebrand: jcode → next_code.

Only .rs and build.rs files. Patterns:
  jcode::          → next_code::
  jcode_IDENT      → next_code_IDENT
  use jcode        → use next_code
  extern crate jcode → extern crate next_code

Does not attempt full string-literal awareness; path/use patterns rarely
appear inside strings as idents. Accepts some string collateral for
crate-name strings like "jcode_foo".
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Order matters: apply more specific patterns first where needed.
PATTERNS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"\bjcode::"), "next_code::"),
    (re.compile(r"\bextern\s+crate\s+jcode\b"), "extern crate next_code"),
    (re.compile(r"\buse\s+jcode\b"), "use next_code"),
    # jcode_foo, jcode_base, etc. — not jcode- (hyphen is not a Rust ident)
    (re.compile(r"\bjcode_([A-Za-z0-9_]+)"), r"next_code_\1"),
    # bare crate root in paths already handled via jcode::
    # bare `jcode` as a sole use-tree root without :: is covered by use jcode
]


def should_skip(path: Path, root: Path) -> bool:
    rel = path.relative_to(root)
    parts = rel.parts
    if "target" in parts or ".git" in parts:
        return True
    return False


def rewrite_text(text: str) -> str:
    out = text
    for pat, repl in PATTERNS:
        out = pat.sub(repl, out)
    return out


def iter_rust_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if should_skip(path, root):
            continue
        name = path.name
        if name.endswith(".rs") or name == "build.rs":
            files.append(path)
    return sorted(files)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
    )
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--verbose", "-v", action="store_true")
    args = ap.parse_args()
    root: Path = args.root.resolve()

    changed = 0
    scanned = 0
    for path in iter_rust_files(root):
        scanned += 1
        try:
            original = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        updated = rewrite_text(original)
        if updated != original:
            changed += 1
            if args.verbose or args.dry_run:
                print(f"{'DRY ' if args.dry_run else 'WRITE'} {path.relative_to(root)}")
            if not args.dry_run:
                path.write_text(updated, encoding="utf-8")

    print(f"rewrite_rust_idents: scanned={scanned} changed={changed} dry_run={args.dry_run}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
