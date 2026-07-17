#!/usr/bin/env python3
"""Fix corrupted auth match arms and strip remaining NextCode subscription refs."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(r"c:\Users\ADMIN\Documents\Projects\next-code")


def write(rel: str, text: str) -> None:
    path = ROOT / rel
    path.write_text(text, encoding="utf-8", newline="\n")
    print(f"updated {rel}")


def fix_auth_mod() -> None:
    rel = "crates/next-code-base/src/auth/mod.rs"
    t = (ROOT / rel).read_text(encoding="utf-8")

    # Fix three corrupted match arms that lost their LoginProviderTarget::NextCode headers
    # Pattern: `crate::provider_catalog::                if ...` through closing `}`
    corrupted = re.compile(
        r"[ \t]*crate::provider_catalog::\s+if[\s\S]*?\n[ \t]*\}\n",
        re.MULTILINE,
    )
    # Only remove ones that mention subscription_catalog
    def repl_corrupted(m: re.Match) -> str:
        block = m.group(0)
        if "subscription_catalog" in block:
            return ""
        return block

    t = corrupted.sub(repl_corrupted, t)

    # Remove probe_next_code_status function and its call
    t = re.sub(
        r"[ \t]*record_auth_probe_step\(&mut timings, \"next-code\", \|\| probe_next_code_status\(&mut status\)\);\n",
        "",
        t,
    )
    t = re.sub(
        r"\nfn probe_next_code_status\(status: &mut AuthStatus\) \{\n(?:.*\n)*?\}\n",
        "\n",
        t,
        count=1,
    )

    # Remove next_code field usages that would break - replace status.next_code assignments
    # and references in tuples. Keep AuthStatus field for now if defined elsewhere;
    # we'll handle struct fields separately.
    t = re.sub(
        r'[ \t]*\("next-code", auth_state_label\(status\.next_code\)\),\n',
        "",
        t,
    )
    t = re.sub(
        r"[ \t]*\|\| self\.next_code == AuthState::Available\n",
        "",
        t,
    )
    t = re.sub(
        r'[ \t]*\("next-code", self\.next_code\.label\(\)\.to_string\(\)\),\n',
        "",
        t,
    )

    # Remove AuthCredentialSource::NextCodeManagedFile arms if they break - keep for now
    # if the enum variant still exists for other credential sources.

    write(rel, t)


def fix_provider_init() -> None:
    rel = "src/cli/provider_init.rs"
    t = (ROOT / rel).read_text(encoding="utf-8")
    t = t.replace("    NextCode,\n", "")
    t = re.sub(r'[ \t]*"next-code" => Self::NextCode,\n', "", t)
    t = re.sub(
        r"[ \t]*\(\s*ProviderChoice::NextCode,\s*crate::provider_catalog::NEXT_CODE_LOGIN_PROVIDER,\s*\),\n",
        "",
        t,
    )
    # Remove match arms for ProviderChoice::NextCode
    t = re.sub(r"[ \t]*ProviderChoice::NextCode =>[^\n]*\n", "", t)
    t = re.sub(
        r"[ \t]*Self::NextCode =>[^\n]*\n",
        "",
        t,
    )
    # Display / as_str
    t = re.sub(r'[ \t]*Self::NextCode => "next-code",\n', "", t)
    write(rel, t)


def fix_provider_e2e() -> None:
    rel = "crates/next-code-provider-doctor/src/provider_e2e.rs"
    path = ROOT / rel
    if not path.exists():
        return
    t = path.read_text(encoding="utf-8")
    # Remove NextCode from enum if present
    t = re.sub(r"[ \t]*NextCode,\n", "", t)
    t = re.sub(r'[ \t]*"next-code" => Some\(Self::NextCode\),\n', "", t)
    # Remove Self::NextCode match arms (single-line and blocks)
    t = re.sub(
        r"[ \t]*Self::NextCode =>[^\n]*\n",
        "",
        t,
    )
    t = re.sub(
        r"[ \t]*Self::NextCode => \{[\s\S]*?\n[ \t]*\}\n",
        "",
        t,
    )
    t = re.sub(
        r"[ \t]*Self::NextCode => std::sync::Arc::new\(next_code_base::provider::jcode::NextCodeProvider::new\(\)\),\n",
        "",
        t,
    )
    write(rel, t)


def strip_tui_auth() -> None:
    """Aggressively remove NextCode subscription UI from TUI auth.rs."""
    rel = "crates/next-code-tui/src/tui/app/auth.rs"
    t = (ROOT / rel).read_text(encoding="utf-8")

    # Remove start_jcode_login entire function
    t = re.sub(
        r"\n    fn start_jcode_login\(&mut self\) \{[\s\S]*?\n    \}\n",
        "\n",
        t,
        count=1,
    )

    # Remove match arm calling start_jcode_login
    t = re.sub(
        r"[ \t]*crate::provider_catalog::LoginProviderTarget::NextCode => self\.start_jcode_login\(\),\n",
        "",
        t,
    )
    t = re.sub(
        r"[ \t]*LoginProviderTarget::NextCode => unreachable!\(\"handled above\"\),\n",
        "",
        t,
    )
    t = re.sub(
        r"[ \t]*if matches!\(provider\.target, LoginProviderTarget::NextCode\) \{[\s\S]*?\n[ \t]*\}\n",
        "",
        t,
        count=1,
    )

    # Lines referencing subscription_* — comment out by deleting whole functions that are subscription-only
    # Find function that builds next-code account panel (around line 76)
    # Heuristic: remove functions whose body is majority subscription refs

    # Delete lines that directly reference subscription_catalog / subscription_api
    # but that can break syntax. Better: find and remove whole fn blocks that contain them.

    # Remove known account manage/logout helpers that use subscription
    for fn_name in [
        "open_next_code_account",
        "logout_next_code_account",
        "clear_next_code_credentials",
        "refresh_next_code_account",
        "show_next_code_account_status",
        "start_next_code",
    ]:
        t = re.sub(
            rf"\n    (?:async )?fn {fn_name}\b[\s\S]*?\n    \}}\n",
            "\n",
            t,
        )

    write(rel, t)
    # Report remaining
    remaining = [
        (i, line)
        for i, line in enumerate(t.splitlines(), 1)
        if "subscription_catalog" in line
        or "subscription_api" in line
        or "start_jcode" in line
        or "LoginProviderTarget::NextCode" in line
    ]
    print(f"  remaining subscription refs in auth.rs: {len(remaining)}")
    for i, line in remaining[:40]:
        print(f"    {i}: {line.strip()[:100]}")


def delete_subscription_tests() -> None:
    for rel in [
        "crates/next-code-base/src/provider/tests/catalog_subscription.rs",
    ]:
        p = ROOT / rel
        if p.exists():
            p.unlink()
            print(f"deleted {rel}")


def main() -> None:
    fix_auth_mod()
    fix_provider_init()
    fix_provider_e2e()
    strip_tui_auth()
    delete_subscription_tests()
    print("done")


if __name__ == "__main__":
    main()
