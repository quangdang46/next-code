//! Façade stub of upstream `xai-grok-shell::session::storage` — session
//! event-log replay/search. No on-disk store in this compile-stub layer.

use std::path::Path;

use agent_client_protocol::SessionUpdate;
use serde::{Deserialize, Serialize};

pub mod search;

/// Load ACP session updates for replay. `Ok(None)` when the session is missing.
pub fn load_updates_for_replay(
    _session_id: &str,
) -> anyhow::Result<Option<Vec<SessionUpdate>>> {
    Ok(None)
}

/// Load updates for a session under an explicit grok-home root.
pub fn load_updates_for_replay_at(
    _session_id: &str,
    _grok_home: &Path,
) -> anyhow::Result<Option<Vec<SessionUpdate>>> {
    Ok(None)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchHit {
    pub session_id: String,
    pub title: String,
    pub score: f64,
    pub updated_at_unix: i64,
    #[serde(default)]
    pub snippet: Option<String>,
}
