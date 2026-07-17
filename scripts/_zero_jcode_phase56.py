#!/usr/bin/env python3
"""Phase 5+6: mechanical jcode renames and delete rebrand docs/tooling."""
from __future__ import annotations

import re
import shutil
from pathlib import Path

ROOT = Path(r"c:\Users\ADMIN\Documents\Projects\next-code")

SKIP_DIRS = {
    ".git",
    "target",
    "node_modules",
    ".venv",
    "dist",
    "__pycache__",
}


def iter_text_files():
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        if path.suffix.lower() not in {
            ".rs",
            ".toml",
            ".md",
            ".py",
            ".json",
            ".jsonl",
            ".yml",
            ".yaml",
            ".txt",
            ".sh",
            ".service",
            ".html",
            ".js",
            ".ts",
            ".tsx",
        } and path.name not in {"Cargo.toml", "README", "AGENTS.md"}:
            # still allow extensionless? skip
            if path.suffix:
                continue
        try:
            yield path
        except Exception:
            pass


def main() -> None:
    # --- Keyword renames in product code ---
    keyword_files = list((ROOT / "crates").rglob("*.rs")) + list((ROOT / "src").rglob("*.rs"))
    for path in keyword_files:
        t = path.read_text(encoding="utf-8", errors="ignore")
        orig = t
        t = t.replace("canceljcode", "cancelnext")
        t = t.replace("stopjcode", "stopnext")
        t = t.replace("real_jcode_tool_smoke", "real_next_code_tool_smoke")
        t = t.replace("cleanup_removes_only_old_jcode_logs", "cleanup_removes_only_old_next_code_logs")
        if t != orig:
            path.write_text(t, encoding="utf-8", newline="\n")
            print(f"keywords: {path.relative_to(ROOT)}")

    # --- Drop plugin dual-read __jcode_* / engines.jcode / package.json jcode ---
    plugin_files = [
        ROOT / "crates/next-code-plugin-runtime/src/api.rs",
        ROOT / "crates/next-code-plugin-runtime/src/tui_system.rs",
        ROOT / "crates/next-code-plugin-runtime/src/tui_api.rs",
        ROOT / "crates/next-code-plugin-runtime/src/bridge.rs",
        ROOT / "crates/next-code-plugin-runtime/src/sandbox.rs",
        ROOT / "crates/next-code-plugin-core/src/manifest.rs",
        ROOT / "crates/next-code-plugin-core/src/tests.rs",
    ]
    for path in plugin_files:
        if not path.exists():
            continue
        t = path.read_text(encoding="utf-8")
        orig = t
        # Remove lines marked dual-read: legacy for jcode
        lines = []
        for line in t.splitlines(keepends=True):
            if any(
                s in line
                for s in (
                    "__jcode_",
                    '"jcode"',
                    "'jcode'",
                    "engines.jcode",
                    "dual-read: legacy",
                    "dual-read: also `__jcode",
                    "dual-read: `__jcode",
                    ".set(\"jcode\"",
                    ".set(\"__jcode",
                    'get("jcode")',
                    "pub jcode:",
                    ".jcode.is_none()",
                    "deserialized.jcode",
                    "jcode: Some",
                    "jcode: None",
                )
            ):
                # Keep structural lines that would break if removed carelessly —
                # for match arms / field defs we need smarter handling below
                if "pub jcode:" in line:
                    continue  # drop field
                if "deserialized.jcode" in line or ".jcode.is_none()" in line:
                    continue
                if "jcode: Some" in line or "jcode: None" in line:
                    continue
                if '.set("jcode"' in line or '.set("__jcode' in line:
                    continue
                if 'get("jcode")' in line:
                    continue
                if "__jcode_" in line and ("format!" in line or "fn_name_legacy" in line or "let " in line):
                    continue
                if "dual-read" in line and line.strip().startswith("//"):
                    continue
                if '"jcode"' in line and ("or_else" in line or "get(" in line or "missing" in line):
                    # rewrite missing message later
                    pass
            lines.append(line)
        t = "".join(lines)
        # Specific rewrites
        t = t.replace(
            '"missing \'nextcode\', \'jcode\', or \'pi\' field"',
            '"missing \'nextcode\' or \'pi\' field"',
        )
        t = re.sub(
            r"\.or_else\(\|\| value\.get\(\"jcode\"\)\)\s*//[^\n]*\n",
            "\n",
            t,
        )
        t = re.sub(
            r"\.or_else\(\|\| value\.get\(\"jcode\"\)\)",
            "",
            t,
        )
        if t != orig:
            path.write_text(t, encoding="utf-8", newline="\n")
            print(f"plugin: {path.relative_to(ROOT)}")

    # --- Rename .jcode directories to .next-code if present ---
    for jcode_dir in [
        ROOT / ".jcode",
        ROOT / "crates/next-code-app-core/.jcode",
    ]:
        if jcode_dir.exists() and jcode_dir.is_dir():
            dest = jcode_dir.with_name(".next-code")
            if dest.exists():
                # merge: copy contents then remove
                for item in jcode_dir.rglob("*"):
                    if item.is_file():
                        rel = item.relative_to(jcode_dir)
                        target = dest / rel
                        target.parent.mkdir(parents=True, exist_ok=True)
                        shutil.copy2(item, target)
                shutil.rmtree(jcode_dir)
                print(f"merged {jcode_dir} -> {dest}")
            else:
                jcode_dir.rename(dest)
                print(f"renamed {jcode_dir} -> {dest}")

    # Code that resolves ".jcode" project dir — prefer .next-code only (no dual-read)
    for path in keyword_files:
        t = path.read_text(encoding="utf-8", errors="ignore")
        orig = t
        # Common patterns
        t = t.replace('".jcode"', '".next-code"')
        t = t.replace("'.jcode'", "'.next-code'")
        t = t.replace("/.jcode/", "/.next-code/")
        t = t.replace("\\.jcode\\", "\\.next-code\\")
        t = t.replace("~/.jcode", "~/.next-code")
        if t != orig:
            path.write_text(t, encoding="utf-8", newline="\n")
            print(f"dotfolder: {path.relative_to(ROOT)}")

    # --- Rename/delete scripts/jcode_* and phone-server units ---
    for path in list((ROOT / "scripts").glob("jcode_*")):
        path.unlink()
        print(f"deleted {path.relative_to(ROOT)}")
    units = ROOT / "scripts/phone-server/units"
    if units.exists():
        for path in units.glob("jcode-*.service"):
            path.unlink()
            print(f"deleted {path.relative_to(ROOT)}")

    # --- Phase 6: delete rebrand docs + scripts/rebrand ---
    for rel in [
        "JCODE_DCP_PLAN.md",
        "docs/plans/JCODE_PROVIDER.md",
        "docs/REBRAND_JCODE_TO_NEXT_CODE_AUDIT.md",
        "docs/REBRAND_IMPLEMENTATION_PLAN.md",
        "docs/REBRAND_MOPUP_PLAN.md",
        "docs/REBRAND_STATUS.md",
        "docs/REBRAND_ALLOWLIST.md",
        "docs/REBRAND_CONTRACT.md",
        "docs/SPONSORED_DISCOVERY_SPONSOR_ONBOARDING.md",
    ]:
        p = ROOT / rel
        if p.exists():
            p.unlink()
            print(f"deleted doc {rel}")

    rebrand = ROOT / "scripts/rebrand"
    if rebrand.exists():
        shutil.rmtree(rebrand)
        print("deleted scripts/rebrand")

    # Delete helper scripts we created
    for helper in ROOT.glob("scripts/_zero_jcode_*.py"):
        helper.unlink()
        print(f"deleted {helper.name}")

    # Purge telemetry-worker jcode domain (delete worker or scrub)
    tw = ROOT / "telemetry-worker"
    if tw.exists():
        # Delete entire telemetry-worker since phone-home is gone
        shutil.rmtree(tw)
        print("deleted telemetry-worker/")

    print("phase5+6 done")


if __name__ == "__main__":
    main()
