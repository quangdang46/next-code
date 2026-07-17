#!/usr/bin/env python3
"""Remove NextCode / subscription residual match arms and imports."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(r"c:\Users\ADMIN\Documents\Projects\next-code")


def edit(rel: str, fn) -> None:
    path = ROOT / rel
    old = path.read_text(encoding="utf-8")
    new = fn(old)
    if new != old:
        path.write_text(new, encoding="utf-8", newline="\n")
        print(f"updated {rel}")
    else:
        print(f"unchanged {rel}")


def drop_lines_with(text: str, needles: list[str]) -> str:
    out = []
    for line in text.splitlines(keepends=True):
        if any(n in line for n in needles):
            continue
        out.append(line)
    return "".join(out)


def drop_nextcode_match_arms(text: str) -> str:
    # Single-line arms
    text = re.sub(
        r"[ \t]*\|?\s*LoginProviderTarget::NextCode[^\n]*\n",
        "",
        text,
    )
    text = re.sub(
        r"[ \t]*LoginProviderAuthStateKey::NextCode[^\n]*\n",
        "",
        text,
    )
    text = re.sub(
        r"[ \t]*\|?\s*RuntimeProviderId::NextCode[^\n]*\n",
        "",
        text,
    )
    text = re.sub(
        r"[ \t]*Self::NextCode =>[^\n]*\n",
        "",
        text,
    )
    # Or-pattern fragments like `| LoginProviderTarget::NextCode`
    text = re.sub(r"\s*\|\s*LoginProviderTarget::NextCode\b", "", text)
    text = re.sub(r"LoginProviderTarget::NextCode\s*\|\s*", "", text)
    text = re.sub(r"\s*\|\s*RuntimeProviderId::NextCode\b", "", text)
    text = re.sub(r"RuntimeProviderId::NextCode\s*\|\s*", "", text)
    return text


def main() -> None:
    # provider_init.rs — remove NextCode arms that construct NextCodeProvider
    def fix_provider_init(t: str) -> str:
        t = re.sub(
            r"[ \t]*LoginProviderTarget::NextCode => Arc::new\(provider::jcode::NextCodeProvider::new\(\)\),\n",
            "",
            t,
        )
        t = re.sub(
            r"[ \t]*Arc::new\(provider::jcode::NextCodeProvider::new\(\)\)\n?",
            "",
            t,
        )
        t = drop_nextcode_match_arms(t)
        # Remove blocks that only exist for next-code subscription
        t = re.sub(
            r"[ \t]*if .*subscription_catalog.*\{.*?\n[ \t]*\}\n",
            "",
            t,
            flags=re.DOTALL,
        )
        return t

    edit("src/cli/provider_init.rs", fix_provider_init)

    # auth/integration.rs
    edit(
        "crates/next-code-base/src/auth/integration.rs",
        lambda t: drop_nextcode_match_arms(t),
    )

    # auth/mod.rs — drop NextCode arms; leave fields that may need manual cleanup
    def fix_auth_mod(t: str) -> str:
        t = drop_nextcode_match_arms(t)
        # Remove next_code field from AuthState if present
        t = re.sub(r"[ \t]*next_code:[^\n]*,\n", "", t)
        t = re.sub(r"[ \t]*pub next_code:[^\n]*\n", "", t)
        t = re.sub(
            r"[ \t]*crate::provider_catalog::LoginProviderTarget::NextCode => \{.*?\n[ \t]*\}\n",
            "",
            t,
            flags=re.DOTALL,
        )
        return t

    edit("crates/next-code-base/src/auth/mod.rs", fix_auth_mod)

    # auth/lifecycle.rs
    def fix_lifecycle(t: str) -> str:
        t = drop_nextcode_match_arms(t)
        t = re.sub(
            r"[ \t]*crate::provider_catalog::LoginProviderTarget::NextCode => \{.*?\n[ \t]*\}\n",
            "",
            t,
            flags=re.DOTALL,
        )
        t = drop_lines_with(
            t,
            [
                "subscription_catalog",
                "subscription_api",
            ],
        )
        return t

    edit("crates/next-code-base/src/auth/lifecycle.rs", fix_lifecycle)

    # provider/selection.rs
    edit(
        "crates/next-code-base/src/provider/selection.rs",
        lambda t: drop_nextcode_match_arms(t),
    )

    # auth_test/probes.rs
    def fix_probes(t: str) -> str:
        t = re.sub(
            r"[ \t]*crate::provider_catalog::LoginProviderTarget::NextCode => \{.*?\n[ \t]*\}\n",
            "",
            t,
            flags=re.DOTALL,
        )
        return drop_nextcode_match_arms(t)

    edit("src/cli/auth_test/probes.rs", fix_probes)

    # proctitle
    edit(
        "src/cli/proctitle.rs",
        lambda t: re.sub(
            r'[ \t]*Some\(Command::Account \{ \.\. \}\) => "next-code account"\.to_string\(\),\n',
            "",
            t,
        ),
    )

    # provider_init: also strip subscription_catalog lines
    edit(
        "src/cli/provider_init.rs",
        lambda t: drop_lines_with(t, ["subscription_catalog", "subscription_api", "provider::jcode"]),
    )

    print("done")


if __name__ == "__main__":
    main()
