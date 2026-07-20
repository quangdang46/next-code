//! Façade stub of upstream `xai-grok-shell::extensions::mcp`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatus {
    #[default]
    Ready,
    Initializing,
    Unavailable,
    NeedsAuth,
    Connecting,
    Connected,
    Failed,
    Disabled,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct McpToolEntry {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct McpServerStatusPayload {
    #[serde(default, alias = "server_name")]
    pub name: String,
    #[serde(default)]
    pub session_id: Option<String>,
    pub status: McpServerStatus,
    #[serde(default)]
    pub tools: Option<Vec<McpToolEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct McpToolsChanged {
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub server_name: String,
    #[serde(default)]
    pub servers: Vec<McpServerStatusPayload>,
}
