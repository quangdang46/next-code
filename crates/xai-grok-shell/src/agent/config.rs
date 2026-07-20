//! Façade stub of upstream `xai-grok-shell::agent::config` — grown for PR7 pager.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use xai_grok_shared::ui_config::{ContextualHints, UiConfig};

use crate::auth::GrokComConfig;
use crate::sampling::types::ReasoningEffort;
use crate::util::config::RemoteSettings;

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

pub const DEFAULT_AGENT_TYPE: &str = "grok-build-plan";

pub fn default_agent_type() -> String {
    DEFAULT_AGENT_TYPE.to_owned()
}

pub const CLI_CHAT_PROXY_BASE_URL_DEFAULT: &str = "https://cli-chat-proxy.grok.com/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PermissionMode {
    #[default]
    Plan,
    AcceptEdits,
    BypassPermissions,
    Auto,
    DontAsk,
}

impl PermissionMode {
    pub const VALID_VALUES: &'static [&'static str] = &[
        "plan",
        "accept-edits",
        "bypass-permissions",
        "auto",
        "dont-ask",
        "default",
        "ask",
        "always-approve",
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BoolFlag {
    #[default]
    Unset,
    True,
    False,
}

impl BoolFlag {
    pub fn env(_name: &str) -> Self {
        Self::Unset
    }

    pub fn requirement(self, v: Option<bool>) -> Self {
        match v {
            Some(true) => Self::True,
            Some(false) => Self::False,
            None => self,
        }
    }

    pub fn config(self, v: Option<bool>) -> Self {
        self.requirement(v)
    }

    pub fn managed(self, v: Option<bool>) -> Self {
        self.requirement(v)
    }

    pub fn feature_flag(self, v: Option<bool>) -> Self {
        self.requirement(v)
    }

    pub fn default(self, _v: bool) -> Self {
        self
    }

    pub fn resolve(self) -> ResolvedBool {
        ResolvedBool {
            value: matches!(self, Self::True),
            source: ConfigSource::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedBool {
    pub value: bool,
    pub source: ConfigSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConfigSource {
    #[default]
    Default,
    User,
    Project,
    Managed,
    Env,
    Cli,
    Remote,
    Requirement,
}

impl fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Default => "default",
            Self::User => "user",
            Self::Project => "project",
            Self::Managed => "managed",
            Self::Env => "env",
            Self::Cli => "cli",
            Self::Remote => "remote",
            Self::Requirement => "requirement",
        };
        f.write_str(s)
    }
}

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
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub prompt_body: Option<String>,
    #[serde(default)]
    pub tool_config: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentDefinitionError {
    #[error("agent-definition stub: not implemented")]
    NotImplemented,
}

impl AgentDefinition {
    pub fn from_file(_path: &std::path::Path) -> Result<Self, AgentDefinitionError> {
        Err(AgentDefinitionError::NotImplemented)
    }

    pub fn from_json(_json: &serde_json::Value) -> Result<Self, AgentDefinitionError> {
        Err(AgentDefinitionError::NotImplemented)
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

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

#[derive(Debug, Clone, Default)]
pub struct CliAgentOverrides {
    pub tools: Option<Vec<String>>,
    pub disallowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub permission_mode: Option<PermissionMode>,
    pub permission_rules: Vec<xai_grok_workspace::permission::types::PermissionRule>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EndpointsConfig {
    pub api_base: Option<String>,
    pub cli_chat_proxy_base_url: Option<String>,
    pub telemetry_endpoint: Option<String>,
    pub xai_api_base_url: String,
    pub alpha_test_key: Option<String>,
    pub models_base_url: Option<String>,
    pub models_list_url: Option<String>,
    pub feedback_base_url: Option<String>,
    pub trace_upload_url: Option<String>,
    pub trace_upload_bucket: Option<String>,
    pub trace_upload_region: Option<String>,
    pub trace_upload_credentials_file: Option<String>,
    pub trace_upload_credentials: Option<String>,
    pub trace_upload_endpoint_url: Option<String>,
    pub deployment_key: Option<String>,
    pub managed_config_url: Option<String>,
    pub otel_exporter_otlp_endpoint: Option<String>,
    pub otel_exporter_otlp_traces_endpoint: Option<String>,
    pub otel_exporter_otlp_headers: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceBucketUrlSource {
    Config,
    Env,
    Default,
    None,
}

impl fmt::Display for TraceBucketUrlSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Config => "config",
            Self::Env => "env",
            Self::Default => "default",
            Self::None => "none",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedTraceBucketUrl {
    pub value: String,
    pub source: TraceBucketUrlSource,
}

impl EndpointsConfig {
    pub fn from_config_value(_config: &toml::Value) -> Self {
        Self::default()
    }

    pub fn proxy_url(&self) -> String {
        blank_as_unset(&self.cli_chat_proxy_base_url)
            .or_else(|| blank_as_unset(&self.api_base))
            .unwrap_or_else(|| CLI_CHAT_PROXY_BASE_URL_DEFAULT.to_string())
    }

    pub fn with_deployment_key(mut self, key: Option<String>) -> Self {
        self.deployment_key = key;
        self
    }

    pub fn resolve_trace_bucket_url(&self) -> Option<ResolvedTraceBucketUrl> {
        self.trace_upload_bucket.as_ref().map(|b| ResolvedTraceBucketUrl {
            value: b.clone(),
            source: TraceBucketUrlSource::Config,
        })
    }

    pub fn has_noninteractive_upload_auth(&self) -> bool {
        self.trace_upload_credentials.is_some()
            || self.trace_upload_credentials_file.is_some()
            || self.deployment_key.is_some()
    }

    pub fn resolve_upload_method(
        &self,
        _auth_token: Option<String>,
    ) -> Option<crate::session::repo_changes::UploadMethod> {
        if self.has_noninteractive_upload_auth() || self.trace_upload_bucket.is_some() {
            Some(crate::session::repo_changes::UploadMethod::Direct {
                service_account_key: None,
            })
        } else {
            None
        }
    }

    pub fn resolve_direct_upload_method(
        &self,
    ) -> Option<crate::session::repo_changes::UploadMethod> {
        if self.trace_upload_bucket.is_some() || self.trace_upload_url.is_some() {
            Some(crate::session::repo_changes::UploadMethod::Direct {
                service_account_key: None,
            })
        } else {
            None
        }
    }

    pub fn resolve_otlp_traces_endpoint(&self) -> String {
        self.otel_exporter_otlp_traces_endpoint
            .clone()
            .or_else(|| {
                self.otel_exporter_otlp_endpoint
                    .as_ref()
                    .map(|b| format!("{b}/v1/traces"))
            })
            .unwrap_or_default()
    }

    pub fn resolve_otlp_headers(&self) -> Vec<(String, String)> {
        vec![]
    }

    pub fn resolve_otlp_export_interval(&self) -> Option<std::time::Duration> {
        None
    }

    pub fn resolve_otlp_timeout(&self) -> Option<std::time::Duration> {
        None
    }

    pub fn resolve_traces_export_enabled(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub trace_upload_enabled: bool,
    /// Upstream field used by pager trace_cmd (`Option` so unset ≠ false).
    pub trace_upload: Option<bool>,
}

impl TelemetryConfig {
    pub fn is_trace_upload_enabled(&self) -> bool {
        self.trace_upload_enabled || self.trace_upload.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelsConfig {
    pub default: Option<String>,
    pub allowlist: Vec<String>,
    pub denylist: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeResolutionContext<'a> {
    pub raw_config: &'a toml::Value,
    pub remote_settings: Option<&'a RemoteSettings>,
    pub is_headless: bool,
    pub cli_subagents: Option<bool>,
    pub cli_web_search_model: Option<&'a str>,
    pub cli_session_summary_model: Option<&'a str>,
    pub cli_experimental_memory: bool,
    pub cli_no_memory: bool,
    pub disable_web_search: bool,
    pub todo_gate: bool,
    pub laziness_debug_log: Option<&'a std::path::Path>,
    pub storage_mode: Option<&'a str>,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("model switch incompatible with agent")]
pub struct ModelSwitchIncompatibleAgentError;

impl ModelSwitchIncompatibleAgentError {
    pub fn from_acp_error(_err: &agent_client_protocol::Error) -> Option<Self> {
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompatSessionSource {
    pub sessions: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CompatSessionsConfig {
    pub claude: CompatSessionSource,
    pub codex: CompatSessionSource,
    pub cursor: CompatSessionSource,
}

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
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub models: ModelsConfig,
    #[serde(skip)]
    pub mode: AgentMode,
    #[serde(skip)]
    pub remote_settings: Option<RemoteSettings>,
    #[serde(skip)]
    pub cli_agent_overrides: CliAgentOverrides,
    #[serde(skip)]
    pub cli_agents: Vec<AgentDefinition>,
    #[serde(skip)]
    pub memory_config: Option<crate::config::MemoryConfig>,
    #[serde(skip)]
    pub agent_profile_path: Option<PathBuf>,
    #[serde(skip)]
    pub client_version: Option<String>,
    #[serde(skip)]
    pub subagents_enabled: bool,
    #[serde(skip)]
    pub subagent_model_overrides: HashMap<String, String>,
    #[serde(skip)]
    pub subagent_toggle: HashMap<String, bool>,
    #[serde(skip)]
    pub cli_subagents: Option<bool>,
    #[serde(skip)]
    pub default_yolo_mode: bool,
    #[serde(skip)]
    pub default_auto_mode: bool,
    #[serde(skip)]
    pub reasoning_effort_override: Option<ReasoningEffort>,
    #[serde(skip)]
    pub default_model_override: Option<String>,
}

impl Config {
    pub fn new_from_toml_cfg(_raw: &toml::Value) -> Result<Self, String> {
        Ok(Self::default())
    }

    pub fn load() -> Result<Self, String> {
        Ok(Self::default())
    }

    pub fn resolve_runtime_fields(&mut self, _ctx: &RuntimeResolutionContext<'_>) {}

    pub fn configure_refresher(&self) {}

    pub fn start_system_power_listener(&self) {}

    pub fn is_trace_upload_enabled(&self) -> bool {
        self.telemetry.is_trace_upload_enabled()
    }
}

pub fn resolve_compat_sessions_from_raw(
    _raw: Result<&toml::Value, ()>,
    _remote: Option<&RemoteSettings>,
) -> CompatSessionsConfig {
    CompatSessionsConfig::default()
}

fn blank_as_unset(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

pub fn resolve_external_otel_config(
    info: xai_grok_telemetry::external::config::ExternalClientInfo,
) -> xai_grok_telemetry::external::config::ExternalOtelConfig {
    xai_grok_telemetry::external::config::ExternalOtelConfig { client: info }
}

pub fn is_telemetry_explicitly_disabled_sync() -> bool {
    false
}
