//! Façade stub of upstream `xai-grok-shell::session::merge` — merged
//! local+remote session-list DTO for the pager's session picker.

use std::time::Duration;

use serde::Serialize;

use crate::agent::session_registry_client::SessionRegistryClient;

pub const REMOTE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedSession {
    pub session_id: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_prompt: Option<String>,
    pub updated_at: String,
    pub created_at: String,
    pub cwd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    /// `"local"` | `"remote"` | `"conversation"` etc.
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_label: Option<String>,
}

pub async fn fetch_merged(
    _client: Option<&SessionRegistryClient>,
    _cwd: Option<&str>,
    _query: Option<&str>,
    _limit: usize,
) -> Vec<MergedSession> {
    Vec::new()
}
