#!/usr/bin/env python3
from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(r"c:\Users\ADMIN\Documents\Projects\next-code")
out = ROOT / "crates/next-code-plugin-runtime/src/tui_system.rs"

src = subprocess.check_output(
    ["git", "show", "HEAD:crates/next-code-plugin-runtime/src/tui_system.rs"],
    cwd=ROOT,
).decode("utf-8")

# Remove legacy prefix vars
src = re.sub(
    r"\n\s*let _?legacy_kb_prefix = format!\([\s\S]*?\);\n",
    "\n",
    src,
    count=1,
)
src = re.sub(
    r"\n\s*let _?legacy_evt_prefix = format!\([\s\S]*?\);\n",
    "\n",
    src,
    count=1,
)

# Remove register_kb_legacy / register_evt_legacy + set(__jcode_...)
src = re.sub(
    r"\n\s*let register_kb_legacy = Function::new\([\s\S]*?\.set\(\s*\"__jcode_register_keybinding\"\s*,\s*register_kb_legacy\s*\)[\s\S]*?;\n",
    "\n",
    src,
    count=1,
)
src = re.sub(
    r"\n\s*let register_evt_legacy = Function::new\([\s\S]*?\.set\(\s*\"__jcode_register_tui_event\"\s*,\s*register_evt_legacy\s*\)[\s\S]*?;\n",
    "\n",
    src,
    count=1,
)

# Remove or_else get __jcode_tui_pi
src = re.sub(
    r"\n\s*\.or_else\(\|_ \| globals\.get::<_, Object<'js>>\(\"__jcode_tui_pi\"\)\)[^\n]*\n",
    "\n",
    src,
)

# Remove fn_name_legacy definitions
src = re.sub(
    r"\n\s*let fn_name_legacy = format!\(\"__jcode_[^\"]+\".*?\);\n",
    "\n",
    src,
)

# Simplify func lookup in invoke_keybinding / invoke_event to primary-only
src = re.sub(
    r"let func = match globals\.get::<_, rquickjs::Function<'_>>\(fn_name_primary\.as_str\(\)\) \{\s*"
    r"Ok\(f\) => f,\s*"
    r"Err\(_\) => match globals\.get::<_, rquickjs::Function<'_>>\(fn_name_legacy\.as_str\(\)\) \{\s*"
    r"Ok\(f\) => f,\s*"
    r"Err\(_\) => return Ok\(false\),[^\n]*\s*"
    r"\},\s*"
    r"\};",
    "let func = match globals.get::<_, rquickjs::Function<'_>>(fn_name_primary.as_str()) {\n"
    "                Ok(f) => f,\n"
    "                Err(_) => return Ok(false), // No handler registered\n"
    "            };",
    src,
)

# Same pattern maybe with different return for events
src = re.sub(
    r"let func = match globals\.get::<_, rquickjs::Function<'_>>\(fn_name_primary\.as_str\(\)\) \{\s*"
    r"Ok\(f\) => f,\s*"
    r"Err\(_\) => match globals\.get::<_, rquickjs::Function<'_>>\(fn_name_legacy\.as_str\(\)\) \{\s*"
    r"Ok\(f\) => f,\s*"
    r"Err\(_\) => return Ok\(None\),[^\n]*\s*"
    r"\},\s*"
    r"\};",
    "let func = match globals.get::<_, rquickjs::Function<'_>>(fn_name_primary.as_str()) {\n"
    "                Ok(f) => f,\n"
    "                Err(_) => return Ok(None),\n"
    "            };",
    src,
)

# Remove legacy result_obj blocks that set __jcode_result
src = re.sub(
    r"\n\s*let result_obj_legacy = rquickjs::Object::new\(ctx\.clone\(\)\)[\s\S]*?"
    r"\.set\(\s*\"__jcode_result\"\s*,\s*result_obj_legacy\s*\)[\s\S]*?;\n",
    "\n",
    src,
)

# Simplify handled dual-read
src = re.sub(
    r"\.or_else\(\|\| \{\s*globals\s*\.get::<_, rquickjs::Object<'_>>\(\"__jcode_result\"\)[^\n]*\s*"
    r"\.ok\(\)\s*\.and_then\(\|o\| o\.get::<_, bool>\(\"handled\"\)\.ok\(\)\)\s*\}\)",
    "",
    src,
)
src = re.sub(
    r"\n\s*\.or_else\(\|_ \| globals\.get::<_, rquickjs::Object<'_>>\(\"__jcode_result\"\)\)[^\n]*",
    "",
    src,
)

# Drop leftover jcode comment lines / dual-read comments mentioning jcode
cleaned = []
for line in src.splitlines(keepends=True):
    low = line.lower()
    if "__jcode_" in line and line.strip().startswith("//"):
        continue
    if "dual-read" in low and "jcode" in low:
        continue
    cleaned.append(line)
src = "".join(cleaned)

# Also remove set("__jcode_tui_pi"...) if present
src = re.sub(
    r"\n\s*ctx\.globals\(\)\.set\(\"__jcode_tui_pi\",[^\n]*\n",
    "\n",
    src,
)
src = re.sub(
    r"\n\s*globals\.set\(\"__jcode_tui_pi\",[^\n]*\n",
    "\n",
    src,
)

out.write_text(src, encoding="utf-8", newline="\n")
remaining = [(i, l.strip()) for i, l in enumerate(src.splitlines(), 1) if "jcode" in l.lower()]
print(f"wrote {out}, remaining jcode lines: {len(remaining)}")
for i, l in remaining[:30]:
    print(f"  {i}: {l[:120]}")
