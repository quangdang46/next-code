//! Façade stub of upstream `xai-grok-shell::agent::config` — the second
//! highest-frequency pager import prefix (51 hits). Upstream is an 11k-line
//! file; this keeps only the shapes the future pager touches directly
//! (`Config`, `AgentDefinition`, `AgentMode`, `AgentSelectionConfig`,
//! `CliAgentOverrides`, `CLI_CHAT_PROXY_BASE_URL_DEFAULT`). Note this is a
//! self-contained simplified `AgentDefinition`, deliberately NOT the same
//! type as `xai_grok_agent::config::AgentDefinition` (that stub crate
//! covers the discovery/plugin surface only) — upstream itself re-exports
//! one crate's type into the other, but duplicating the (much larger) real
//! definition here would require vendoring far more of `xai-grok-tools`/
//! `xai-tool-types` than this façade layer is scoped for.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The mode in which the agent is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AgentMode {
    Tui,
    Headless,
    Stdio,
    Serve,
    Leader,
    #[default]
    Generic,
}

/// Default agent type when the server or user config doesn't specify one.
pub const DEFAULT_AGENT_TYPE: &str = "grok-build-plan";

pub fn default_agent_type() -> String {
    DEFAULT_AGENT_TYPE.to_owned()
}

/// Default base URL for the cli chat proxy.
pub const CLI_CHAT_PROXY_BASE_URL_DEFAULT: &str = "https://cli-chat-proxy.grok.com/v1";

/// Simplified stand-in for upstream `PermissionMode` (full variant set
/// mirrors `next-code`'s own permission modes so a future GrokHost mapping
/// stays lossless).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PermissionMode {
    #[default]
    Plan,
    AcceptEdits,
    BypassPermissions,
    Auto,
    DontAsk,
}

/// Simplified stand-in for upstream's (much larger) `AgentDefinition`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub disallowed_tools: Vec<String>,
    #[serde(default)]
    pub permission_mode: PermissionMode,
    #[serde(default)]
    pub session_tools_allowlist: Option<Vec<String>>,
    #[serde(default)]
    pub session_tools_denylist: Option<Vec<String>>,
    #[serde(default)]
    pub plugin_name: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentDefinitionError {
    #[error("agent-definition stub: not implemented")]
    NotImplemented,
}

impl AgentDefinition {
    /// Upstream parses a `.md` file with YAML frontmatter; this stub never
    /// reads disk.
    pub fn from_file(_path: &std::path::Path) -> Result<Self, AgentDefinitionError> {
        Err(AgentDefinitionError::NotImplemented)
    }

    /// Upstream parses inline JSON (e.g. ACP `_meta.agentProfile`); this
    /// stub never actually deserializes arbitrary JSON payloads.
    pub fn from_json(_json: &str) -> Result<Self, AgentDefinitionError> {
        Err(AgentDefinitionError::NotImplemented)
    }
}

/// Configuration for selecting the agent definition (`[agent]` in
/// `config.toml`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentSelectionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt_label: Option<String>,
}

/// CLI-flag overrides layered on top of an `AgentDefinition`.
#[derive(Debug, Clone, Default)]
pub struct CliAgentOverrides {
    pub tools: Option<Vec<String>>,
    pub disallowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub permission_mode: Option<PermissionMode>,
}

impl CliAgentOverrides {
    pub fn apply_to_definition(&self, def: &mut AgentDefinition) {
        if let Some(ref tools) = self.tools {
            def.tools = tools.clone();
        }
        if let Some(ref dt) = self.disallowed_tools {
            def.disallowed_tools = dt.clone();
        }
        if let Some(ref pm) = self.permission_mode {
            def.permission_mode = *pm;
        }
    }

    pub fn apply_to_subagent_definition(&self, def: &mut AgentDefinition) {
        def.session_tools_allowlist = self.tools.clone();
        def.session_tools_denylist = self.disallowed_tools.clone();
    }

    pub fn has_definition_overrides(&self) -> bool {
        self.tools.is_some() || self.disallowed_tools.is_some() || self.permission_mode.is_some()
    }
}

/// Nested endpoint settings the pager reads (trace upload, voice, …).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EndpointsConfig {
    pub api_base: Option<String>,
    pub cli_chat_proxy_base_url: Option<String>,
    pub telemetry_endpoint: Option<String>,
}

/// Nested telemetry toggles the pager reads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub trace_upload_enabled: bool,
}

impl TelemetryConfig {
    pub fn is_trace_upload_enabled(&self) -> bool {
        self.trace_upload_enabled
    }
}

/// Nested grok.com / auth config the pager touches at spawn.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GrokComConfig {
    pub enabled: bool,
}

/// Simplified stand-in for upstream's top-level agent `Config`.
/// Fields added as the pager compile surface demands them (PR7).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(default)]
    pub agent: AgentSelectionConfig,
    #[serde(default)]
    pub endpoints: EndpointsConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub grok_com_config: GrokComConfig,
}

impl Config {
    /// Upstream builds from a TOML table; stub returns defaults.
    pub fn new_from_toml_cfg(_table: &toml::Table) -> anyhow::Result<Self> {
        Ok(Self::default())
    }

    /// Upstream loads from disk; stub returns defaults in Ok.
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self::default())
    }

    pub fn configure_refresher(&self) {}

    pub fn start_system_power_listener(&self) {}
}
