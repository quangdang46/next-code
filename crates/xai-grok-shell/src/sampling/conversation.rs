//! Minimal ConversationItem stubs for pager history stats.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConversationItem {
    System(SystemItem),
    User(UserItem),
    Assistant(AssistantItem),
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemItem {
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserItem {
    #[serde(default)]
    pub content: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synthetic_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssistantItem {
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<serde_json::Value>,
}
