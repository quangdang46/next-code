//! Stub of upstream `xai-grok-agent::config` — only the enums/struct shapes
//! the future pager's `agents_modal.rs` imports (`BuiltinAgentName`,
//! `AgentDefinition`, `AgentScope`, `PromptMode`). Upstream also carries
//! `ToolServerConfig` / `SubagentCapabilityMode` fields on `AgentDefinition`
//! (from `xai-grok-tools` / `xai-tool-types`); those are dropped here since
//! nothing in this compile-stub layer consumes them yet.

use serde::{Deserialize, Serialize};

/// Built-in agent identifiers. Variant list matches upstream 1:1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltinAgentName {
    GrokBuild,
    GrokBuildConcise,
    GrokBuildPlan,
    GrokBuildPlanNoSubagents,
    GrokBuildAskUser,
    Codex,
    Opencode,
    GeneralPurpose,
    Explore,
    Plan,
    BrowserUse,
    GrokBuildOrchestrator,
}

/// How an agent's prompt body combines with the base template.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptMode {
    /// Body is appended to the base template. Default.
    #[default]
    Extend,
    /// Body IS the complete system prompt.
    Full,
}

/// Where the agent definition was discovered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentScope {
    /// .grok/agents/ (project-level, highest priority)
    Project,
    /// ~/.grok/agents/ (user-level)
    User,
    /// ~/.grok/bundled/agents/ (lowest-priority bundled cache)
    Bundled,
    /// Built-in agent.
    #[default]
    BuiltIn,
}

impl AgentScope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
            Self::Bundled => "bundled",
            Self::BuiltIn => "built-in",
        }
    }
}

/// Simplified stand-in for upstream's `AgentDefinition` (tool_config /
/// capability_mode fields dropped — see module doc).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    #[serde(skip)]
    pub plugin_name: Option<String>,
    #[serde(default)]
    pub prompt_mode: PromptMode,
}
