#!/usr/bin/env python3
"""Phases 2-4: remove telemetry phone-home, sponsored discovery, and support@jcode.sh."""
from __future__ import annotations

import re
import shutil
from pathlib import Path

ROOT = Path(r"c:\Users\ADMIN\Documents\Projects\next-code")


def write(rel: str, text: str) -> None:
    (ROOT / rel).write_text(text, encoding="utf-8", newline="\n")
    print(f"updated {rel}")


def main() -> None:
    # --- Phase 2: delete telemetry-core crate ---
    telem = ROOT / "crates/next-code-telemetry-core"
    if telem.exists():
        shutil.rmtree(telem)
        print("deleted crates/next-code-telemetry-core")

    # Workspace Cargo.toml
    p = ROOT / "Cargo.toml"
    t = p.read_text(encoding="utf-8")
    t = t.replace('    "crates/next-code-telemetry-core",\n', "")
    t = re.sub(r"^next-code-telemetry-core = \{[^\n]+\n", "", t, flags=re.M)
    write("Cargo.toml", t)

    p = ROOT / "crates/next-code-base/Cargo.toml"
    t = p.read_text(encoding="utf-8")
    t = re.sub(r"^next-code-telemetry-core = \{[^\n]+\n", "", t, flags=re.M)
    write("crates/next-code-base/Cargo.toml", t)

    # Replace pub mod telemetry / re-exports in base lib.rs with a local no-op stub
    # Better: remove the mod and create a tiny stub that keeps call sites compiling
    # while doing nothing. Plan preferred deleting call sites; stub is safer for
    # wide surface. We'll create stub then optionally strip later.

    stub = '''//! Telemetry phone-home was removed for the open-source next-code fork.
//! These no-op stubs keep call sites compiling without contacting any server.

#![allow(unused_variables, dead_code)]

pub fn init() {}
pub fn record_install() {}
pub fn record_session_start(_provider: &str, _model: &str) {}
pub fn record_session_end(_reason: &str) {}
pub fn record_turn_end() {}
pub fn record_tool_execution(_name: &str, _input: &serde_json::Value, _ok: bool, _latency_ms: u64) {}
pub fn record_auth_started(_provider: &str, _method: &str) {}
pub fn record_auth_completed(_provider: &str, _method: &str) {}
pub fn record_auth_failed(_provider: &str, _method: &str, _reason: &str) {}
pub fn record_auth_surface_blocked(_provider: &str, _method: &str) {}
pub fn record_auth_cancelled(_provider: &str, _method: &str) {}
pub fn record_feedback(_text: &str) {}
pub fn record_onboarding_step(_step: &str) {}
pub fn record_upgrade(_from: &str, _to: &str) {}
pub fn record_error(_category: &str) {}
pub fn record_discovery(_event: DiscoveryTelemetry<'_>) {}
pub fn flush() {}
pub fn is_enabled() -> bool { false }

#[derive(Debug, Clone)]
pub struct DiscoveryTelemetry<'a> {
    pub request_id: &'a str,
    pub phase: &'a str,
    pub category: Option<&'a str>,
    pub selected_tool: Option<&'a str>,
    pub outcome: &'a str,
    pub failure_reason: Option<&'a str>,
    pub http_status: Option<u16>,
    pub latency_ms: u64,
    pub response_bytes: Option<u64>,
    pub result_count: Option<u32>,
    pub query_present: bool,
    pub reason_present: bool,
    pub benchmark_run: bool,
    pub endpoint: &'a str,
}

// Re-export commonly used types from usage-types if present.
pub use next_code_usage_types::{ErrorCategory, SessionEndReason};
'''
    # Check how telemetry was exported from base
    lib = (ROOT / "crates/next-code-base/src/lib.rs").read_text(encoding="utf-8")
    if "pub mod telemetry" in lib or "pub use next_code_telemetry" in lib or "telemetry" in lib:
        print("lib.rs telemetry refs:")
        for i, line in enumerate(lib.splitlines(), 1):
            if "telemetry" in line.lower():
                print(f"  {i}: {line}")

    write("crates/next-code-base/src/telemetry_stub.rs", stub)

    # --- Phase 3: delete discover + sponsors ---
    for rel in [
        "crates/next-code-app-core/src/tool/discover.rs",
        "crates/next-code-base/src/sponsors.rs",
        "crates/next-code-tui/src/tui/app/sponsor_disclosure.rs",
    ]:
        p = ROOT / rel
        if p.exists():
            p.unlink()
            print(f"deleted {rel}")
    prov = ROOT / "crates/next-code-base/src/sponsors"
    if prov.exists():
        shutil.rmtree(prov)
        print("deleted sponsors/")

    # Strip mod discover / registration from tool/mod.rs
    p = ROOT / "crates/next-code-app-core/src/tool/mod.rs"
    t = p.read_text(encoding="utf-8")
    t = t.replace("mod discover;\n", "")
    t = re.sub(
        r"[ \t]*// Sponsored discovery[\s\S]*?if crate::config::config\(\)\.sponsors\.enabled \{[\s\S]*?\n[ \t]*\}\n",
        "",
        t,
        count=1,
    )
    write("crates/next-code-app-core/src/tool/mod.rs", t)

    # Strip mod sponsors from base lib.rs
    p = ROOT / "crates/next-code-base/src/lib.rs"
    t = p.read_text(encoding="utf-8")
    t = re.sub(r"^pub mod sponsors;\n", "", t, flags=re.M)
    t = re.sub(r"^mod sponsors;\n", "", t, flags=re.M)
    # Replace telemetry re-export with stub
    t = re.sub(
        r"^pub (?:mod|use)[^\n]*telemetry[^\n]*\n",
        "pub mod telemetry { pub use crate::telemetry_stub::*; }\nmod telemetry_stub;\n",
        t,
        flags=re.M,
    )
    if "telemetry_stub" not in t:
        # Insert stub mod near top after other mods
        t = "mod telemetry_stub;\npub mod telemetry {\n    pub use crate::telemetry_stub::*;\n}\n" + t
    write("crates/next-code-base/src/lib.rs", t)

    # --- Phase 4: support email ---
    p = ROOT / "crates/next-code-tui/src/tui/app/support.rs"
    t = p.read_text(encoding="utf-8")
    t = t.replace(
        'pub(super) const SUPPORT_EMAIL: &str = "support@jcode.sh";',
        'pub(super) const SUPPORT_EMAIL: &str = "support@next-code.dev";',
    )
    # Prefer GitHub issues — change mailto builder to note GitHub if present
    t = t.replace("support@jcode.sh", "support@next-code.dev")
    write("crates/next-code-tui/src/tui/app/support.rs", t)

    print("phases 2-4 structural done")


if __name__ == "__main__":
    main()
