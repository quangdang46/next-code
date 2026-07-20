//! Façade stub of upstream `xai-grok-shell::extensions::mcp`.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerSource {
    Managed,
    #[default]
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatusReason {
    #[default]
    TransportClosed,
    HandshakeFailed,
    ConfigAdded,
    ConfigRemoved,
    ConfigChanged,
    Disabled,
    AuthExpired,
    Initialized,
    RestartSucceeded,
    RestartFailed,
    ManagedTokenRefreshed,
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
#[serde(rename_all = "camelCase")]
pub struct McpServerStatusPayload {
    #[serde(default)]
    pub session_id: String,
    #[serde(default, alias = "server_name")]
    pub name: String,
    #[serde(default)]
    pub source: McpServerSource,
    pub status: McpServerStatus,
    #[serde(default)]
    pub reason: McpServerStatusReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default)]
    pub tools: Option<serde_json::Value>,
}

/// Post-handshake tools push: `{ sessionId, serverName, tools }`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpToolsChanged {
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub server_name: String,
    #[serde(default)]
    pub tools: Vec<McpToolEntry>,
}
