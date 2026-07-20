pub mod types;

pub use types::*;

pub const DEFAULT_ASK_USER_QUESTION_TIMEOUT_ENABLED: bool = true;
pub const RESPONSE_TIMEOUT_ENV: &str = "GROK_ASK_USER_QUESTION_TIMEOUT_SECS";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub question: String,
    pub options: Vec<QuestionOption>,
    #[serde(default, alias = "multi_select")]
    pub multi_select: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
