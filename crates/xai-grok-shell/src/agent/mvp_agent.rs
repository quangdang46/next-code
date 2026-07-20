//! Stub of upstream `xai-grok-shell::agent::mvp_agent`.
//!
//! Upstream: `impl acp::Agent for MvpAgent` only. `Rc<MvpAgent>: Agent` comes
//! from agent-client-protocol's blanket `impl<T: Agent> Agent for Rc<T>` —
//! do not add an orphan `impl Agent for Rc<MvpAgent>` here.

use std::path::Path;
use std::sync::Arc;

use agent_client_protocol as acp;
use async_trait::async_trait;

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

#[async_trait(?Send)]
impl acp::Agent for MvpAgent {
    async fn initialize(
        &self,
        _args: acp::InitializeRequest,
    ) -> acp::Result<acp::InitializeResponse> {
        // Upstream: `InitializeResponse::new(ProtocolVersion::V1)` then builders.
        Ok(acp::InitializeResponse::new(acp::ProtocolVersion::V1)
            .agent_capabilities(acp::AgentCapabilities::default())
            .auth_methods(vec![]))
    }

    async fn authenticate(
        &self,
        _args: acp::AuthenticateRequest,
    ) -> acp::Result<acp::AuthenticateResponse> {
        Ok(acp::AuthenticateResponse::new())
    }

    async fn new_session(
        &self,
        _args: acp::NewSessionRequest,
    ) -> acp::Result<acp::NewSessionResponse> {
        Ok(acp::NewSessionResponse::new(acp::SessionId::new(
            "stub-session",
        )))
    }

    async fn prompt(&self, _args: acp::PromptRequest) -> acp::Result<acp::PromptResponse> {
        Ok(acp::PromptResponse::new(acp::StopReason::EndTurn))
    }

    async fn cancel(&self, _args: acp::CancelNotification) -> acp::Result<()> {
        Ok(())
    }
}
