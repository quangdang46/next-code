//! Stub of upstream `xai-grok-shell::agent::roster` — lists sub-agents
//! available for a session. Always empty in this compile-stub layer.

#[derive(Debug, Clone, Default)]
pub struct RosterEntry {
    pub name: String,
    pub description: String,
}
