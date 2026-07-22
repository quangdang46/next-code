//! MCP Protocol types (JSON-RPC 2.0)

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC request
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC response
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

/// MCP Initialize params
#[derive(Debug, Clone, Serialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ClientCapabilities {}

#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP Initialize result
#[derive(Debug, Clone, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServerCapabilities {
    #[serde(default)]
    pub tools: Option<ToolsCapability>,
    #[serde(default)]
    pub resources: Option<ResourcesCapability>,
    #[serde(default)]
    pub prompts: Option<PromptsCapability>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ResourcesCapability {
    #[serde(default)]
    pub subscribe: bool,
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PromptsCapability {
    #[serde(rename = "listChanged", default)]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

/// MCP Tool definition from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// tools/list result
#[derive(Debug, Clone, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<McpToolDef>,
}

/// tools/call params
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Value,
}

/// tools/call result
#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ContentBlock>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// Content block in tool result
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}

/// MCP server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpServerConfig {
    /// Command for stdio servers. Empty for HTTP/SSE / streamable-HTTP servers.
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    /// Whether this server can be shared across sessions (default: true).
    /// Stateless API wrappers (Todoist, Canvas) should be shared.
    /// Stateful servers (Playwright browser) should not be shared.
    #[serde(default = "default_shared")]
    pub shared: bool,
    /// Transport type from Claude Code / Cursor configs ("stdio", "http", "sse",
    /// "streamable-http"). Defaults to stdio when a command is present.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
    /// URL for HTTP/SSE / streamable-HTTP servers (Claude Code / Cursor compat).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Optional static HTTP headers (Authorization, API keys, etc.).
    /// Mirrors grok `HttpConfig.headers` / mcp.json `headers` map — no OAuth
    /// browser flow here (see grok `xai-grok-mcp::oauth`).
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub headers: std::collections::HashMap<String, String>,
    /// Whether this server is enabled (default: true). Disabled servers stay
    /// registered in config but are not spawned or connected at load time
    /// until re-enabled (issue #436). opencode-style `"enabled": false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Claude Code style alias: `"disabled": true`. Wins over `enabled` when
    /// both are present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

impl McpServerConfig {
    /// Stdio (command-based) MCP servers: non-empty command and not an HTTP
    /// transport type.
    pub fn is_stdio(&self) -> bool {
        if self.is_http() {
            return false;
        }
        !self.command.trim().is_empty()
    }

    /// HTTP / SSE / streamable-HTTP MCP: explicit transport type, or empty
    /// command with a URL (Cursor/Claude Code style).
    pub fn is_http(&self) -> bool {
        if let Some(t) = &self.transport {
            let t = t.to_ascii_lowercase();
            if t == "http" || t == "sse" || t == "streamable-http" {
                return true;
            }
        }
        self.command.trim().is_empty()
            && self
                .url
                .as_ref()
                .is_some_and(|u| !u.trim().is_empty())
    }

    /// Whether next-code can connect this entry (stdio or HTTP).
    pub fn is_connectable(&self) -> bool {
        self.is_stdio() || self.is_http()
    }

    /// Whether this server should be spawned/connected automatically.
    /// Defaults to true. `"disabled": true` (Claude Code style) wins over
    /// `"enabled"` (opencode style) when both are present. Disabled servers
    /// stay in config and can still be connected on demand by name.
    pub fn is_enabled(&self) -> bool {
        if let Some(disabled) = self.disabled {
            return !disabled;
        }
        self.enabled.unwrap_or(true)
    }
}

fn default_shared() -> bool {
    true
}

/// Full MCP configuration file
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct McpConfig {
    /// Server map. Accepts the canonical Claude Code key `mcpServers` as well as
    /// next-code's historical `servers` key.
    #[serde(default, alias = "mcpServers")]
    pub servers: std::collections::HashMap<String, McpServerConfig>,
}

impl McpConfig {
    /// Load config from file
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Save config to a JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Import MCP servers from Claude Code and Codex CLI on first run.
    /// Only runs if ~/.next-code/mcp.json doesn't exist yet.
    #[expect(
        clippy::collapsible_if,
        reason = "Import logic keeps source-specific MCP config handling explicit"
    )]
    fn import_from_external() {
        let next_code_mcp = match crate::storage::next_code_dir() {
            Ok(dir) => dir.join("mcp.json"),
            Err(_) => return,
        };

        if next_code_mcp.exists() {
            return; // Not first run
        }

        let mut imported = Self::default();
        let mut sources = Vec::new();

        // Import from Claude Code. The canonical user config is `~/.claude.json`
        // (top-level `mcpServers` + per-project entries); fall back to the older
        // `~/.claude/mcp.json` layout for users who still have it.
        if let Ok(claude_json) = crate::storage::user_home_path(".claude.json") {
            if claude_json.exists() {
                let config = Self::load_claude_json(&claude_json, None);
                let count = config.servers.len();
                if count > 0 {
                    sources.push(format!("{} from Claude Code", count));
                    imported.servers.extend(config.servers);
                }
            }
        }
        if let Ok(claude_mcp) = crate::storage::user_home_path(".claude/mcp.json") {
            if claude_mcp.exists() {
                if let Ok(config) = Self::load_from_file(&claude_mcp) {
                    let count = config.servers.len();
                    if count > 0 {
                        sources.push(format!("{} from Claude Code (legacy)", count));
                        imported.servers.extend(config.servers);
                    }
                }
            }
        }

        // Import from Codex CLI (~/.codex/config.toml)
        if let Ok(codex_config) = crate::storage::user_home_path(".codex/config.toml") {
            if codex_config.exists() {
                if let Ok(config) = Self::load_from_codex_toml(&codex_config) {
                    let count = config.servers.len();
                    if count > 0 {
                        sources.push(format!("{} from Codex CLI", count));
                        // Codex overrides Claude for same-named servers
                        imported.servers.extend(config.servers);
                    }
                }
            }
        }

        if !imported.servers.is_empty() {
            if let Err(e) = imported.save_to_file(&next_code_mcp) {
                crate::logging::error(&format!("Failed to save imported MCP config: {}", e));
                return;
            }
            let names: Vec<&str> = imported.servers.keys().map(|s| s.as_str()).collect();
            crate::logging::info(&format!(
                "MCP: Imported {} servers ({}) from {}",
                imported.servers.len(),
                names.join(", "),
                sources.join(" + "),
            ));
        }
    }

    /// Parse MCP servers from Codex CLI's config.toml ([mcp_servers.*] sections)
    fn load_from_codex_toml(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let table: toml::Table = content.parse()?;

        let mut config = Self::default();
        if let Some(toml::Value::Table(mcp_servers)) = table.get("mcp_servers") {
            for (name, value) in mcp_servers {
                if let toml::Value::Table(server) = value {
                    let command = server
                        .get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if command.is_empty() {
                        continue;
                    }
                    let args = server
                        .get("args")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    let env = server
                        .get("env")
                        .and_then(|v| v.as_table())
                        .map(|t| {
                            t.iter()
                                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                .collect()
                        })
                        .unwrap_or_default();
                    let shared = server
                        .get("shared")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    config.servers.insert(
                        name.clone(),
                        McpServerConfig {
                            command,
                            args,
                            env,
                            shared,
                            transport: None,
                            url: None,
                            headers: Default::default(),
                            enabled: None,
                            disabled: None,
                        },
                    );
                }
            }
        }
        Ok(config)
    }

    /// Parse MCP servers from Claude Code's `~/.claude.json`.
    ///
    /// Claude Code stores a global set under the top-level `mcpServers` key, and
    /// per-project sets under `projects.<abs_path>.mcpServers`. We merge the
    /// global set first, then overlay the entry for `cwd` (if any) so a
    /// project-specific server wins for the active directory.
    fn load_claude_json(path: &std::path::Path, cwd: Option<&std::path::Path>) -> Self {
        let mut config = Self::default();
        let Ok(content) = std::fs::read_to_string(path) else {
            return config;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            return config;
        };

        // Global servers under top-level `mcpServers`.
        if let Some(map) = value.get("mcpServers")
            && let Ok(servers) = serde_json::from_value::<
                std::collections::HashMap<String, McpServerConfig>,
            >(map.clone())
        {
            config.servers.extend(servers);
        }

        // Per-project servers under `projects.<abs_path>.mcpServers`.
        if let (Some(cwd), Some(projects)) =
            (cwd, value.get("projects").and_then(|p| p.as_object()))
        {
            let cwd_str = cwd.to_string_lossy();
            if let Some(project) = projects.get(cwd_str.as_ref())
                && let Some(map) = project.get("mcpServers")
                && let Ok(servers) = serde_json::from_value::<
                    std::collections::HashMap<String, McpServerConfig>,
                >(map.clone())
            {
                config.servers.extend(servers);
            }
        }

        config
    }

    /// Load project-local MCP config files from `project_root`, in override
    /// order: `.next-code/mcp.json`, then `.mcp.json` (Claude Code project
    /// config), then `.claude/mcp.json` (Claude Code compatibility). Later
    /// files override same-named servers from earlier ones.
    fn load_project_locals(project_root: &std::path::Path) -> Self {
        let mut merged = Self::default();
        for relative in [
            ".next-code/mcp.json",
            ".mcp.json",
            ".claude/mcp.json",
        ] {
            let path = project_root.join(relative);
            if path.exists()
                && let Ok(config) = Self::load_from_file(&path)
            {
                merged.servers.extend(config.servers);
            }
        }
        merged
    }

    /// Load from default locations (merges next-code global + local, local overrides),
    /// resolving project-local config against the process working directory.
    pub fn load() -> Self {
        Self::load_for_dir(None)
    }

    /// Load from default locations, resolving project-local config
    /// (`.next-code/mcp.json`, `.mcp.json`, `.claude/mcp.json`, and the per-project
    /// entries in `~/.claude.json`) against `project_dir` instead of the
    /// process working directory when provided.
    ///
    /// Remote/client sessions run inside a long-lived server whose cwd is
    /// unrelated to the session's project, so the session working directory
    /// must be threaded through explicitly (issue #420).
    ///
    /// Runtime connect set: stdio + HTTP/SSE. Unknown shapes (empty command,
    /// no URL) are dropped with a log line. Prefer
    /// [`Self::load_catalog_for_dir`] when you need every raw catalog row.
    #[expect(
        clippy::collapsible_if,
        reason = "Import logic keeps source-specific MCP config merge order explicit"
    )]
    pub fn load_for_dir(project_dir: Option<&std::path::Path>) -> Self {
        let mut merged = Self::load_catalog_for_dir(project_dir);

        merged.servers.retain(|name, cfg| {
            let keep = cfg.is_connectable();
            if !keep {
                crate::logging::info(&format!(
                    "MCP: Skipping unconnectable server '{}' (need command or http url; type={})",
                    name,
                    cfg.transport.as_deref().unwrap_or("-")
                ));
            }
            keep
        });

        merged
    }

    /// Load MCP config for Face / UI catalog display (keeps all parsed rows,
    /// including HTTP). Runtime spawn uses [`Self::load_for_dir`].
    #[expect(
        clippy::collapsible_if,
        reason = "Import logic keeps source-specific MCP config merge order explicit"
    )]
    pub fn load_catalog_for_dir(project_dir: Option<&std::path::Path>) -> Self {
        Self::import_from_external();

        let mut merged = Self::default();

        if let Ok(next_code_dir) = crate::storage::next_code_dir() {
            let next_code_mcp = next_code_dir.join("mcp.json");
            if next_code_mcp.exists() {
                if let Ok(config) = Self::load_from_file(&next_code_mcp) {
                    merged.servers.extend(config.servers);
                }
            }
        }

        if let Ok(claude_json) = crate::storage::user_home_path(".claude.json") {
            if claude_json.exists() {
                let cwd = project_dir.map(std::path::Path::to_path_buf);
                let config = Self::load_claude_json(&claude_json, cwd.as_deref());
                merged.servers.extend(config.servers);
            }
        }

        if let Some(project_root) = project_dir {
            merged
                .servers
                .extend(Self::load_project_locals(project_root).servers);
        }

        merged
    }
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod protocol_tests;
