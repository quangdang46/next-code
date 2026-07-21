//! Stub of upstream `xai-grok-shell::agent::roster`.

use serde::{Deserialize, Serialize};

use crate::sampling::types::ReasoningEffort;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RosterActivity {
    Working,
    #[default]
    Idle,
    NeedsInput,
    Dormant,
    Completed,
    Dead,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RosterOrigin {
    #[default]
    Local,
    Remote { host: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RosterEntry {
    pub session_id: String,
    #[serde(default)]
    pub title: Option<String>,
    pub cwd: String,
    pub is_worktree: bool,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    pub yolo: bool,
    pub activity: RosterActivity,
    pub resident: bool,
    pub last_change_unix_ms: i64,
    pub origin: RosterOrigin,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RosterListResponse {
    pub sessions: Vec<RosterEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RosterChanged {
    #[serde(default)]
    pub upserted: Vec<RosterEntry>,
    #[serde(default)]
    pub removed: Vec<String>,
}

pub const SESSIONS_LIST_METHOD: &str = "x.ai/sessions/list";
pub const SESSIONS_CHANGED_METHOD: &str = "x.ai/sessions/changed";
