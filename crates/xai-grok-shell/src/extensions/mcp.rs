//! Façade stub of upstream `xai-grok-shell::extensions::mcp` — only the
//! catalog/status DTOs the future pager reads (`McpServerStatus`,
//! `McpServerStatusPayload`, `McpToolEntry`, `McpToolsChanged`). The
//! actual `mcp/list` / `mcp/call` ext-method handlers (upstream connects to
//! real MCP servers) are out of scope for this compile-stub layer.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatus {
    Connecting,
    Connected,
    Failed,
    Disabled,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerStatusPayload {
    pub server_name: String,
    pub status: McpServerStatus,
    #[serde(default)]
    pub tools: Vec<McpToolEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct McpToolsChanged {
    pub session_id: String,
    #[serde(default)]
    pub server_name: String,
}
