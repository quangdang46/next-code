//! Stub of upstream `xai-grok-shell::session::share`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareSessionRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareSessionResponse {
    pub share_url: String,
}

pub async fn share_session(_session_id: &str) -> anyhow::Result<ShareSessionResponse> {
    Err(anyhow::anyhow!("share stub"))
}
