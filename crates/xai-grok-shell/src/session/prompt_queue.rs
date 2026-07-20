//! Stub of upstream `xai-grok-shell::session::prompt_queue` — mid-turn
//! follow-up prompt queue DTOs pushed to the pager via ACP notifications.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueEntryWire {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueChanged {
    pub session_id: String,
    #[serde(default)]
    pub entries: Vec<QueueEntryWire>,
}
