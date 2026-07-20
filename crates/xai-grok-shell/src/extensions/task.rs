//! Façade stub of upstream `xai-grok-shell::extensions::task` — DTOs for
//! the `x.ai/task/kill` and subagent-cancel ext methods. `KillOutcome` is
//! re-used from `xai-grok-tools` (PR4) rather than redefined.

use serde::{Deserialize, Serialize};
pub use xai_grok_tools::types::computer::KillOutcome;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KillTaskRequest {
    pub session_id: String,
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillTaskResponse {
    pub task_id: String,
    pub outcome: KillOutcome,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelSubagentRequest {
    pub subagent_id: String,
}
