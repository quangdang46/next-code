//! Stub of upstream `xai-grok-shell::agent::mvp_agent`.

use std::path::Path;
use std::sync::Arc;

use crate::agent::config::{AgentSelectionConfig, Config};
use crate::agent::models::ModelsManager;
use crate::auth::AuthManager;

/// Thin stand-in for the in-process agent handle the pager spawns.
#[derive(Debug, Default)]
pub struct MvpAgent {
    pub config: Config,
}

impl MvpAgent {
    pub fn new_stub(config: Config) -> Self {
        Self { config }
    }

    pub fn with_models<G>(
        _gateway: G,
        cfg: &Config,
        _auth_manager: Arc<AuthManager>,
        _models_manager: ModelsManager,
    ) -> Self {
        Self {
            config: cfg.clone(),
        }
    }

    pub fn resolve_agent_definition(
        _cwd: &Path,
        _agent_profile_path: Option<&Path>,
        _agent_config: &AgentSelectionConfig,
        acp_agent_profile: Option<xai_grok_agent::AgentDefinition>,
        _model_agent_type: Option<&str>,
    ) -> xai_grok_agent::AgentDefinition {
        acp_agent_profile.unwrap_or_default()
    }

    pub fn set_memory_config(&mut self, _config: crate::config::MemoryConfig) {}

    pub fn set_activity(&mut self, _activity: crate::agent::activity::AgentActivity) {}
}

pub fn warm_async_http_client() {}
