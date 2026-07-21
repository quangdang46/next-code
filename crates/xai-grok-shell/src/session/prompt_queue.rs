//! Stub of upstream `xai-grok-shell::session::prompt_queue`.
//! Aligns with `xai-prompt-queue` wire types the pager tests construct.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QueueEntryWire {
    pub id: String,
    #[serde(default)]
    pub version: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_editor: Option<String>,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub position: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QueueChanged {
    pub session_id: String,
    #[serde(default)]
    pub entries: Vec<QueueEntryWire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub running_prompt_id: Option<String>,
}
