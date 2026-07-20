//! Stub of upstream `xai-grok-shell::agent::roster`.

#[derive(Debug, Clone, Default)]
pub struct RosterEntry {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Default)]
pub struct Roster {
    pub entries: Vec<RosterEntry>,
}
