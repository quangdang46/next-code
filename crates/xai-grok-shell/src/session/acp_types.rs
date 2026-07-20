//! Stub of upstream `xai-grok-shell::session::acp_types`.

use serde::{Deserialize, Serialize};

use crate::session::ClientType;

pub use crate::session::{
    ContextInfo, TokenUsageCategory, count_detail, model_display_name,
    should_show_model_fingerprint,
};

/// Nested payload inside [`crate::session::SessionInfoResponse`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfoData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_display_name: Option<String>,
    #[serde(default)]
    pub resolved_model_id: Option<String>,
    #[serde(default)]
    pub model_fingerprint: Option<String>,
    #[serde(default)]
    pub show_model_fingerprint: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub turns: u64,
    #[serde(default)]
    pub turn_index: u64,
    #[serde(default)]
    pub context: ContextInfo,
}

/// ACP `x.ai/feedback` request body.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientFeedbackInput {
    pub session_id: String,
    pub client_type: ClientType,
    #[serde(default)]
    pub rating_type: Option<String>,
    #[serde(default)]
    pub rating_value: Option<i32>,
    #[serde(default)]
    pub feedback_text: Option<String>,
    #[serde(default)]
    pub feedback_categories: Vec<String>,
    #[serde(default)]
    pub context_type: Option<String>,
    #[serde(default)]
    pub turn_number: Option<u64>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub client_version: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub terminal_info: Option<xai_grok_shared::session::FeedbackTerminalInfo>,
}

/// Whether this model slug supports showing checkpoint identity.
pub fn is_coding_model_slug(model: &str) -> bool {
    matches!(model, "grok-build" | "grok-4.5") || model.contains("coding")
}
