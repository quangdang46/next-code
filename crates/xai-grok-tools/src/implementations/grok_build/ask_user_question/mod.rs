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

/// Max characters shown on a Face/Claude-style question chip tab.
pub const QUESTION_HEADER_CHIP_WIDTH: usize = 12;

/// Soft Claude-parity limits (schema documents these; execute may truncate).
pub const MAX_QUESTIONS: usize = 4;
pub const MIN_OPTIONS_PER_QUESTION: usize = 2;
pub const MAX_OPTIONS_PER_QUESTION: usize = 4;

/// A single question with its options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub question: String,
    pub options: Vec<QuestionOption>,
    /// Allow multiple answers. Wire name is `multiSelect` (camelCase); also
    /// accept snake_case `multi_select` for older payloads.
    #[serde(default, alias = "multi_select")]
    pub multi_select: Option<bool>,
    /// Short chip/tab label (≤ [`QUESTION_HEADER_CHIP_WIDTH`] chars). Optional
    /// for backward compatibility; Face falls back to `Q{n}` when missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl Question {
    /// Chip label for multi-question nav: truncated `header`, or `Q{n}` (1-based).
    pub fn chip_label(&self, index: usize) -> String {
        let raw = self
            .header
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Q{}", index + 1));
        truncate_chip_label(&raw, QUESTION_HEADER_CHIP_WIDTH)
    }
}

/// Truncate a chip label to `max_chars` on a UTF-8 char boundary.
pub fn truncate_chip_label(label: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let mut out = String::new();
    for (i, ch) in label.chars().enumerate() {
        if i >= max_chars {
            break;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod question_wire_tests {
    use super::*;

    #[test]
    fn deserializes_claude_shaped_multiselect_and_header() {
        let json = r#"{
            "question": "Which features?",
            "header": "Features",
            "options": [
                {"label": "Auth", "description": "Login"},
                {"label": "Logging", "description": "Logs"}
            ],
            "multiSelect": true
        }"#;
        let q: Question = serde_json::from_str(json).unwrap();
        assert_eq!(q.header.as_deref(), Some("Features"));
        assert_eq!(q.multi_select, Some(true));
        assert_eq!(q.chip_label(0), "Features");
    }

    #[test]
    fn deserializes_snake_case_multi_select_alias() {
        let json = r#"{
            "question": "Pick one?",
            "options": [
                {"label": "A", "description": "a"},
                {"label": "B", "description": "b"}
            ],
            "multi_select": true
        }"#;
        let q: Question = serde_json::from_str(json).unwrap();
        assert_eq!(q.multi_select, Some(true));
        assert!(q.header.is_none());
        assert_eq!(q.chip_label(2), "Q3");
    }

    #[test]
    fn chip_label_truncates_to_twelve() {
        let q = Question {
            question: "Long?".into(),
            options: vec![],
            multi_select: None,
            header: Some("AuthenticationMethod".into()),
            id: None,
        };
        assert_eq!(q.chip_label(0), "Authentication");
        assert_eq!(q.chip_label(0).chars().count(), QUESTION_HEADER_CHIP_WIDTH);
    }
}
