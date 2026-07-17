#!/usr/bin/env python3
"""Fix invalid Rust idents introduced by bare jcode→next-code rewrites."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(".")
SKIP = {".git", "target", "node_modules"}

# Field / path access: foo.next-code → foo.next_code
FIELD = re.compile(
    r"(?<![\w\"'`/-])(?P<pre>(?:self|status|full|fast|s|auth|state|cfg|opts|this|app)\.)next-code\b"
)
# Struct field init: next-code: → next_code:
INIT = re.compile(r"(?P<pre>(?:^|[{\s,]) )next-code\s*:")
# Module path: provider::next-code:: → provider::jcode:: (module file still jcode.rs)
MODPATH = re.compile(r"provider::next-code::")
# Compare both sides: fast.next-code
BOTH = re.compile(r"\.next-code\b(?!\s*[/\"]|\.toml|\.json|/)")

changed = []
for path in ROOT.rglob("*.rs"):
    if any(p in SKIP for p in path.parts):
        continue
    text = path.read_text(encoding="utf-8")
    new = text
    new = FIELD.sub(r"\g<pre>next_code", new)
    new = re.sub(r"(\.|::)next-code(\s*=)", r"\1next_code\2", new)
    new = re.sub(r"(\{|,|\n)\s*next-code\s*:", lambda m: m.group(0).replace("next-code", "next_code", 1), new)
    new = MODPATH.sub("provider::jcode::", new)
    # Remaining .next-code used as field (not path string)
    lines = []
    for line in new.splitlines(keepends=True):
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("//!") or stripped.startswith("*"):
            lines.append(line)
            continue
        if '"' in line or "'" in line:
            # only rewrite outside strings: conservative — field patterns already handled
            lines.append(line)
            continue
        line2 = re.sub(r"\.next-code\b", ".next_code", line)
        line2 = re.sub(r"\bnext-code\s*:", "next_code:", line2)
        lines.append(line2)
    new = "".join(lines)
    if new != text:
        path.write_text(new, encoding="utf-8")
        changed.append(path.as_posix())

print(f"changed={len(changed)}")
for c in changed:
    print(c)
