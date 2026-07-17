#!/usr/bin/env python3
"""Phase-1 mechanical cleanup helpers for zero-jcode (subscription removal).

Run from repo root. Does safe, line-oriented edits only.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8", newline="\n")
    print(f"updated {path.relative_to(ROOT)}")


def strip_match_arm(text: str, pattern: str) -> str:
    """Remove a single match arm like `Foo => bar,` or multi-line arms ending in `,`."""
    return re.sub(pattern, "", text, flags=re.MULTILINE)


def main() -> None:
    # --- login.rs ---
    login = ROOT / "src/cli/login.rs"
    t = read(login)
    t = t.replace("mod next_code_device;\n", "")
    t = re.sub(
        r"\s*LoginProviderTarget::NextCode => login_jcode_flow\(options\.no_browser\)\s*\.await\s*\.map\(\|_\| LoginFlowOutcome::Completed\),",
        "",
        t,
    )
    t = re.sub(
        r"\nasync fn login_jcode_flow\(no_browser: bool\) -> Result<\(\)> \{.*?\n\}\n\npub\(crate\) async fn run_next_code_account_login\(no_browser: bool\) -> Result<\(\)> \{.*?\n\}\n",
        "\n",
        t,
        flags=re.DOTALL,
    )
    write(login, t)

    # --- provider/mod.rs already done ---

    # --- provider/models.rs: make subscription filters pass-through ---
    models = ROOT / "crates/next-code-base/src/provider/models.rs"
    t = read(models)
    # Replace filtered_display_models / ensure_model_allowed_for_subscription bodies if present
    t = re.sub(
        r"pub\(crate\) fn filtered_display_models\(models: impl IntoIterator<Item = String>\) -> Vec<String> \{.*?^\}$",
        "pub(crate) fn filtered_display_models(models: impl IntoIterator<Item = String>) -> Vec<String> {\n"
        "    models.into_iter().collect()\n"
        "}",
        t,
        flags=re.MULTILINE | re.DOTALL,
        count=1,
    )
    t = re.sub(
        r"pub\(crate\) fn ensure_model_allowed_for_subscription\(model: &str\) -> Result<\(\)> \{.*?^\}$",
        "pub(crate) fn ensure_model_allowed_for_subscription(_model: &str) -> Result<()> {\n"
        "    Ok(())\n"
        "}",
        t,
        flags=re.MULTILINE | re.DOTALL,
        count=1,
    )
    # Also filtered_model_routes if it references subscription
    if "subscription_catalog" in t:
        print("WARNING: models.rs still references subscription_catalog — manual fix needed")
        for i, line in enumerate(t.splitlines(), 1):
            if "subscription_catalog" in line:
                print(f"  {i}: {line}")
    write(models, t)

    # --- activation.rs: remove next_code_subscription helper if present ---
    act = ROOT / "crates/next-code-base/src/provider/activation.rs"
    t = read(act)
    t = t.replace("    NextCode,\n", "")
    # Remove next_code_subscription method
    t = re.sub(
        r"\n    pub fn next_code_subscription\(model: impl Into<String>\) -> Self \{.*?\n    \}\n",
        "\n",
        t,
        flags=re.DOTALL,
    )
    # Remove NextCode match arms in Display/FromStr etc.
    t = re.sub(r"\s*Self::NextCode =>[^\n]*,?\n", "\n", t)
    t = re.sub(r'\s*"next-code"\s*=>\s*Ok\(Self::NextCode\),?\n', "\n", t)
    t = re.sub(r"\s*RuntimeProviderId::NextCode =>[^\n]*\n", "\n", t)
    write(act, t)

    print("phase1 helper done")


if __name__ == "__main__":
    main()
