//! Façade stub of upstream `xai-grok-shell::extensions::mcp`.
//!
//! Status enum matches upstream `session::mcp_dispatcher::McpServerStatus`
//! (4 variants) — pager match arms are exhaustive against that set.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerStatus {
    #[default]
    Ready,
    Initializing,
    Unavailable,
    NeedsAuth,
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
    pub session_id: String,
    pub status: McpServerStatus,
    #[serde(default)]
    pub tools: Option<serde_json::Value>,
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
