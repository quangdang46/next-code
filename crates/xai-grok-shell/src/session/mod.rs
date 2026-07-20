//! Façade stub of upstream `xai-grok-shell::session` — only the DTOs/
//! functions the future pager imports directly for session lifecycle
//! (`ContextInfo`, `PromptOrigin`, `ClientType`, `ExtMethodResult`) plus
//! thin sub-modules for persistence/worktree/storage/merge/restore/
//! repo_changes/prompt_queue/info (per plan doc frequency ranking).
//! Upstream's real session engine (acp_session/, goal_orchestrator, mcp
//! dispatch, compaction, two_pass, …) is NOT vendored here.

pub mod info;
pub mod merge;
pub mod persistence;
pub mod prompt_queue;
pub mod repo_changes;
pub mod restore;
pub mod result;
pub mod storage;
pub mod worktree;

pub use persistence::{resolve_local_session, resolve_local_session_any_cwd};
pub use result::{Empty, ExtMethodResult};

use serde::{Deserialize, Serialize};

/// Simplified stand-in for upstream's `prod_mc_cli_chat_proxy_types`
/// re-export — the feedback client-type discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ClientType {
    #[default]
    Tui,
    Headless,
    Api,
}

/// Describes who originated a prompt: the user, or the shell's auto-wake
/// system reacting to a completed background task / subagent. Variant
/// list matches upstream 1:1 (small enum, cheap to keep faithful).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptOrigin {
    User,
    TaskCompleted { task_id: String },
    SubagentCompleted { subagent_id: String },
    NotificationDrain,
    GoalSummary,
    GoalClassifierNudge,
    SchedulerFired,
}

/// Simplified stand-in for upstream's much larger `ContextInfo` (token/
/// compaction accounting struct in `acp_types.rs`) — only the fields
/// needed for a `from_notification` construction path are kept.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextInfo {
    pub used: u64,
    pub total: u64,
    pub turn_count: u64,
    pub tool_call_count: u64,
    pub usage_pct: u8,
}

impl ContextInfo {
    /// Upstream builds this from an ACP session-notification payload; this
    /// stub always returns the zeroed default (no real accounting).
    pub fn from_notification(_payload: &serde_json::Value) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionInfoResponse {
    pub session_id: String,
    pub cwd: String,
}
