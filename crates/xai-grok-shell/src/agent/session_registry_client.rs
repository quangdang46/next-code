//! Stub of upstream `xai-grok-shell::agent::session_registry_client`.

use std::sync::Arc;

use crate::auth::AuthManager;

#[derive(Debug, Clone, Default)]
pub struct SessionRegistryClient {
    pub proxy_url: String,
    pub user_agent: String,
    pub deployment_key: Option<String>,
    pub alpha_test_key: Option<String>,
    pub session_id: Option<String>,
    pub auth: Option<Arc<AuthManager>>,
}

impl SessionRegistryClient {
    pub fn new(proxy_url: impl Into<String>, user_agent: impl Into<String>) -> Self {
        Self {
            proxy_url: proxy_url.into(),
            user_agent: user_agent.into(),
            ..Default::default()
        }
    }

    pub fn with_deployment_key(mut self, key: Option<String>) -> Self {
        self.deployment_key = key;
        self
    }

    pub fn with_alpha_test_key(mut self, key: Option<String>) -> Self {
        self.alpha_test_key = key;
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_auth(mut self, auth: Arc<AuthManager>) -> Self {
        self.auth = Some(auth);
        self
    }

    pub async fn list_sessions(&self) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }

    pub async fn search(
        &self,
        _query: Option<&str>,
        _limit: i64,
    ) -> anyhow::Result<Vec<RemoteSessionHit>> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone, Default)]
pub struct RemoteSessionHit {
    pub session_id: String,
    pub title: String,
    pub updated_at: String,
    pub summary: String,
    pub first_prompt: Option<String>,
}
