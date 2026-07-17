#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(r"c:\Users\ADMIN\Documents\Projects\next-code")


def main() -> None:
    # 1. Fix include
    p = ROOT / "crates/next-code-base/src/provider/tests.rs"
    t = p.read_text(encoding="utf-8")
    t2 = t.replace('include!("tests/catalog_subscription.rs");\n', "")
    p.write_text(t2, encoding="utf-8", newline="\n")
    print("fixed include", t != t2)

    # 2. Remove next_code from AuthStatus
    p = ROOT / "crates/next-code-base/src/auth/status_types.rs"
    t = p.read_text(encoding="utf-8")
    t2 = re.sub(
        r"\n    /// Next Code subscription router credentials\n    pub next_code: AuthState,\n",
        "\n",
        t,
    )
    p.write_text(t2, encoding="utf-8", newline="\n")
    print("status_types", t != t2)

    # 3. Strip TUI auth subscription functions
    p = ROOT / "crates/next-code-tui/src/tui/app/auth.rs"
    t = p.read_text(encoding="utf-8")
    for name in [
        "show_next_code_subscription_status",
        "open_next_code_account_management",
        "start_next_code_account_logout",
        "clear_next_code_local_credentials",
    ]:
        t = re.sub(
            rf"\n    #\[allow\(dead_code\)\]\n    pub\(super\) fn {name}\b[\s\S]*?(?=\n    (?:#\[|pub|async fn|fn ))",
            "\n",
            t,
            count=1,
        )
        t = re.sub(
            rf"\n    (?:pub\(super\) |pub\(crate\) )?fn {name}\b[\s\S]*?(?=\n    (?:#\[|pub|async fn|fn ))",
            "\n",
            t,
            count=1,
        )

    # Drop lines with subscription refs (may leave syntax holes — cargo will tell us)
    lines_out = []
    for i, line in enumerate(t.splitlines(keepends=True), 1):
        if "subscription_catalog" in line or "subscription_api" in line:
            print(f"  DROP auth.rs:{i}: {line.strip()[:90]}")
            continue
        lines_out.append(line)
    t = "".join(lines_out)
    p.write_text(t, encoding="utf-8", newline="\n")
    print(
        "auth.rs remaining:",
        sum(1 for l in t.splitlines() if "subscription_" in l),
    )

    # 4. auth_account_commands
    p = ROOT / "crates/next-code-tui/src/tui/app/auth_account_commands.rs"
    t = p.read_text(encoding="utf-8")
    for needle in (
        "AccountCommand::NextCodeStatus",
        "AccountCommand::NextCodeManage",
        "AccountCommand::NextCodeLogout",
    ):
        t = re.sub(rf"[ \t]*{re.escape(needle)}[^\n]*\n", "", t)
    p.write_text(t, encoding="utf-8", newline="\n")
    print("auth_account_commands updated")

    # 5. AccountCommand enum variants
    for path in ROOT.rglob("*.rs"):
        if "target" in path.parts:
            continue
        txt = path.read_text(encoding="utf-8", errors="ignore")
        if "enum AccountCommand" in txt and "NextCodeStatus" in txt:
            print("AccountCommand enum in", path.relative_to(ROOT))
            txt2 = re.sub(r"[ \t]*NextCodeStatus,?\n", "", txt)
            txt2 = re.sub(r"[ \t]*NextCodeManage,?\n", "", txt2)
            txt2 = re.sub(r"[ \t]*NextCodeLogout,?\n", "", txt2)
            if txt2 != txt:
                path.write_text(txt2, encoding="utf-8", newline="\n")
                print("  cleaned")

    # 6. OpenRouter NextCodeSubscription transport — remove variant usage
    for rel in (
        "crates/next-code-base/src/provider/openrouter.rs",
        "crates/next-code-provider-openrouter-runtime/src/lib.rs",
    ):
        p = ROOT / rel
        if not p.exists():
            continue
        t = p.read_text(encoding="utf-8")
        t2 = t.replace("NextCodeSubscription", "OpenRouterApiKey")  # remap if needed
        # Better: remove the specific arms
        t2 = re.sub(
            r"[ \t]*return Self::NextCodeSubscription;\n",
            "",
            t,
        )
        t2 = re.sub(
            r'[ \t]*\| "next-code-subscription" => Some\(Self::NextCodeSubscription\),\n',
            "",
            t2,
        )
        t2 = re.sub(r"[ \t]*NextCodeSubscription,?\n", "", t2)
        t2 = re.sub(r"[ \t]*Self::NextCodeSubscription =>[^\n]*\n", "", t2)
        p.write_text(t2, encoding="utf-8", newline="\n")
        print("openrouter", rel, t != t2)

    print("done")


if __name__ == "__main__":
    main()
