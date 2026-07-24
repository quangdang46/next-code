//! AskUserQuestion wire types + format helpers (Face ACP + next-code brain).
//!
//! Stock grok-build also ships a `xai_tool_runtime::Tool` impl here; next-code
//! registers an app-core tool instead and reuses these types/formatters.

pub mod format;
pub mod types;

pub use format::{
    CANCEL_TEXT, format_accepted_tool_result, format_chat_about_this,
    format_id_keyed_accepted_tool_result, format_skip_interview,
};
pub use types::{
    AskUserQuestionExtRequest, AskUserQuestionExtResponse, AskUserQuestionMode, QuestionAnnotation,
    UserQuestionError, UserQuestionRequest, UserQuestionResponse, UserQuestionResult,
    UserQuestionSender,
};

/// Default max time to wait for answers (30 minutes).
pub const RESPONSE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30 * 60);

/// Default for `timeout_enabled`: timer armed unless disarmed.
pub const DEFAULT_ASK_USER_QUESTION_TIMEOUT_ENABLED: bool = true;

/// Env var: override [`RESPONSE_TIMEOUT`] with a duration in **seconds**.
pub const RESPONSE_TIMEOUT_ENV: &str = "GROK_ASK_USER_QUESTION_TIMEOUT_SECS";

/// Parse [`RESPONSE_TIMEOUT_ENV`] (positive integer seconds).
pub fn response_timeout_env_secs() -> Option<u64> {
    let raw = std::env::var(RESPONSE_TIMEOUT_ENV).ok()?;
    match raw.trim().parse::<u64>() {
        Ok(secs) if secs > 0 => Some(secs),
        _ => None,
    }
}

/// Effective wait budget for one questionnaire (env override or default).
pub fn response_timeout() -> std::time::Duration {
    response_timeout_env_secs()
        .map(std::time::Duration::from_secs)
        .unwrap_or(RESPONSE_TIMEOUT)
}

/// Runtime-configurable timeout policy for the ask_user_question tool.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AskUserQuestionParams {
    #[serde(default)]
    pub timeout_enabled: Option<bool>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

impl AskUserQuestionParams {
    /// `Some(duration)` = bounded wait; `None` = wait forever.
    pub fn wait_budget(&self) -> Option<std::time::Duration> {
        if !self
            .timeout_enabled
            .unwrap_or(DEFAULT_ASK_USER_QUESTION_TIMEOUT_ENABLED)
        {
            return None;
        }
        match self.timeout_secs {
            Some(secs) if secs > 0 => Some(std::time::Duration::from_secs(secs)),
            Some(_) => Some(response_timeout()),
            None => Some(response_timeout()),
        }
    }
}

/// A single option within a question.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A single question with its options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub question: String,
    pub options: Vec<QuestionOption>,
    /// Model-facing schema name is snake_case (`multi_select`); also accept ACP `multiSelect`.
    #[serde(default, alias = "multi_select")]
    pub multi_select: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
