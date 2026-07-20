//! Stub of upstream `xai-grok-shell::session::info` — per-session path key
//! (`id` + `cwd`) used by `persistence::session_dir` and first-prompt helpers.

use agent_client_protocol::SessionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub id: SessionId,
    pub cwd: String,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            id: SessionId::new(String::new()),
            cwd: String::new(),
        }
    }
}
