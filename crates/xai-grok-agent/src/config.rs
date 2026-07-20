//! Stub of upstream `xai-grok-agent::config` — shapes the pager's
//! `agents_modal.rs` imports (`BuiltinAgentName`, `AgentDefinition`, …).

use serde::{Deserialize, Serialize};

use xai_grok_tools::registry::types::ToolServerConfig;

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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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

/// Simplified stand-in for upstream's `AgentDefinition`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    #[serde(skip)]
    pub plugin_name: Option<String>,
    #[serde(default)]
    pub prompt_mode: PromptMode,
    #[serde(default)]
    pub scope: AgentScope,
    #[serde(default)]
    pub source_path: Option<std::path::PathBuf>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub tool_config: ToolServerConfig,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub prompt_body: Option<String>,
}

impl BuiltinAgentName {
    pub fn definition(self) -> AgentDefinition {
        AgentDefinition {
            name: format!("{self:?}"),
            description: String::new(),
            scope: AgentScope::BuiltIn,
            ..Default::default()
        }
    }

    pub fn subagent_variants() -> &'static [BuiltinAgentName] {
        &[
            Self::GeneralPurpose,
            Self::Explore,
            Self::Plan,
            Self::BrowserUse,
        ]
    }
}

impl std::str::FromStr for BuiltinAgentName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grok-build" | "GrokBuild" => Ok(Self::GrokBuild),
            "grok-build-concise" | "GrokBuildConcise" => Ok(Self::GrokBuildConcise),
            "grok-build-plan" | "GrokBuildPlan" => Ok(Self::GrokBuildPlan),
            "grok-build-plan-no-subagents" | "GrokBuildPlanNoSubagents" => {
                Ok(Self::GrokBuildPlanNoSubagents)
            }
            "grok-build-ask-user" | "GrokBuildAskUser" => Ok(Self::GrokBuildAskUser),
            "codex" | "Codex" => Ok(Self::Codex),
            "opencode" | "Opencode" => Ok(Self::Opencode),
            "general-purpose" | "GeneralPurpose" => Ok(Self::GeneralPurpose),
            "explore" | "Explore" => Ok(Self::Explore),
            "plan" | "Plan" => Ok(Self::Plan),
            "browser-use" | "BrowserUse" => Ok(Self::BrowserUse),
            "grok-build-orchestrator" | "GrokBuildOrchestrator" => Ok(Self::GrokBuildOrchestrator),
            other => Err(format!("unknown builtin agent: {other}")),
        }
    }
}
