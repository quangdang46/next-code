//! Stub of upstream `xai-grok-agent::discovery` — the pager's
//! `agents_modal.rs` calls `discover(cwd)` to list agent definitions found
//! under `.grok/agents/`, `~/.grok/agents/`, and the bundled cache. This
//! stub always returns empty (no real filesystem walk); real discovery is
//! runtime-side (PR8), not part of this Face compile-stub layer.

use std::path::Path;

use crate::config::AgentDefinition;

pub fn discover(_cwd: &Path) -> Vec<AgentDefinition> {
    Vec::new()
}
