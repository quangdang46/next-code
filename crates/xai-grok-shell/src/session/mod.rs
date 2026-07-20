//! Façade stub of upstream `xai-grok-shell::session` — only the DTOs/
//! functions the future pager imports directly for session lifecycle
//! (`ContextInfo`, `PromptOrigin`, `ClientType`, `ExtMethodResult`) plus
//! thin sub-modules for persistence/worktree/storage/merge/restore/
//! repo_changes/prompt_queue/info (per plan doc frequency ranking).
//! Upstream's real session engine is NOT vendored here.

pub mod acp_types;
pub mod info;
pub mod memory;
pub mod merge;
pub mod persistence;
pub mod prompt_queue;
pub mod repo_changes;
pub mod restore;
pub mod result;
pub mod share;
pub mod storage;
pub mod worktree;

pub use persistence::{resolve_local_session, resolve_local_session_any_cwd};
pub use result::{Empty, ExtMethodResult};
pub use share::{ShareSessionRequest, ShareSessionResponse};

use serde::{Deserialize, Serialize};

use crate::session::acp_types::SessionInfoData;

/// Simplified stand-in for upstream's `prod_mc_cli_chat_proxy_types`
/// re-export — the feedback client-type discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientType {
    #[default]
    Tui,
    Headless,
    Api,
}

/// Describes who originated a prompt: the user, or the shell's auto-wake
/// system reacting to a completed background task / subagent.
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

impl PromptOrigin {
    pub fn from_prompt_id(prompt_id: &str) -> Self {
        if let Some(rest) = prompt_id.strip_prefix("task-completed-") {
            return Self::TaskCompleted {
                task_id: rest.to_string(),
            };
        }
        if let Some(rest) = prompt_id.strip_prefix("subagent-completed-") {
            return Self::SubagentCompleted {
                subagent_id: rest.to_string(),
            };
        }
        if prompt_id.starts_with("notifications-") || prompt_id.starts_with("notification-") {
            return Self::NotificationDrain;
        }
        if prompt_id.starts_with("scheduler-fired-") {
            return Self::SchedulerFired;
        }
        if prompt_id.starts_with("goal-summary-") {
            return Self::GoalSummary;
        }
        if prompt_id.starts_with("goal-classifier-") {
            return Self::GoalClassifierNudge;
        }
        Self::User
    }

    pub fn is_synthetic(&self) -> bool {
        !matches!(self, Self::User)
    }

    /// Synthetic auto-wake / scheduler prompts should not echo into scrollback.
    pub fn hide_user_echo_from_scrollback(&self) -> bool {
        matches!(
            self,
            Self::TaskCompleted { .. }
                | Self::SubagentCompleted { .. }
                | Self::NotificationDrain
                | Self::SchedulerFired
                | Self::GoalSummary
                | Self::GoalClassifierNudge
        )
    }
}

/// Token-usage category row shown under the `/context` legend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsageCategory {
    pub label: String,
    pub tokens: u64,
    #[serde(default)]
    pub detail: Option<String>,
}

impl TokenUsageCategory {
    pub fn skills_listing(text: &str, count: usize) -> Self {
        Self {
            label: "Skills".to_string(),
            tokens: text.len() as u64,
            detail: Some(count_detail(count, "skill")),
        }
    }

    pub fn mcp_servers(text: &str, count: usize) -> Self {
        Self {
            label: "MCP servers".to_string(),
            tokens: text.len() as u64,
            detail: Some(count_detail(count, "server")),
        }
    }
}

/// Format `12 tools` / `1 tool` style detail strings.
pub fn count_detail(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

/// Context-window accounting snapshot for `/context` and session info.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextInfo {
    pub used: u64,
    pub total: u64,
    pub free_tokens: u64,
    pub message_tokens: u64,
    pub system_prompt_tokens: u64,
    pub turn_count: u64,
    pub tool_call_count: u64,
    pub compaction_count: u64,
    pub usage_pct: u8,
    #[serde(default)]
    pub usage_categories: Vec<TokenUsageCategory>,
    #[serde(default)]
    pub tool_definitions_tokens: u64,
    #[serde(default)]
    pub tool_definitions_count: usize,
    #[serde(default)]
    pub message_count: u64,
    #[serde(default)]
    pub auto_compact_threshold_percent: u8,
}

impl ContextInfo {
    /// Build a minimal snapshot from a token-usage notification.
    pub fn from_notification(used: u64, total: u64) -> Self {
        let total = if total == 0 { used } else { total };
        let usage_pct = if total == 0 {
            0
        } else {
            ((used as f64 / total as f64) * 100.0).round() as u8
        };
        Self {
            used,
            total,
            free_tokens: total.saturating_sub(used),
            usage_pct,
            auto_compact_threshold_percent: 85,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfoResponse {
    pub session_id: String,
    pub cwd: String,
    #[serde(default)]
    pub data: SessionInfoData,
}

/// Resolve a human-facing model label, optionally appending the resolved id.
pub fn model_display_name(
    display_name: Option<&str>,
    model: &str,
    resolved_model_id: Option<&str>,
    show_resolved: bool,
) -> String {
    let base = display_name.unwrap_or(model);
    if show_resolved {
        if let Some(resolved) = resolved_model_id {
            if resolved != base && resolved != model {
                return format!("{base} ({resolved})");
            }
        }
    }
    base.to_string()
}

/// Whether the session-info UI should show a model fingerprint/hash line.
pub fn should_show_model_fingerprint(catalog_flag: bool, model: &str) -> bool {
    catalog_flag || model.contains("grok-build") || model.contains("coding")
}
