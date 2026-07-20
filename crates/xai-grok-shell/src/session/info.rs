//! Stub of upstream `xai-grok-shell::session::info` — minimal per-session
//! metadata DTO for the future pager's session-info display.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Info {
    pub session_id: String,
    pub cwd: String,
}
