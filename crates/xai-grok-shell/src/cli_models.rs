//! Stub of upstream `xai-grok-shell::cli_models`.

use serde::{Deserialize, Serialize};

use crate::agent::config::Config;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthStatus {
    ApiKey,
    LoggedIn(String),
    ModelCredentials(String),
    DeploymentKey,
    NotAuthenticated,
}

impl AuthStatus {
    pub fn resolve(_config: &Config) -> Self {
        Self::NotAuthenticated
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModelId(pub String);

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<str> for ModelId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ModelId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for ModelId {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AvailableModel {
    pub model_id: ModelId,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelsListState {
    pub current_model_id: ModelId,
    pub available_models: Vec<AvailableModel>,
}

/// Upstream lists models over ACP; stub returns empty defaults.
pub async fn list_models<T>(_tx: &T, _client_type: &str, _client_version: &str) -> anyhow::Result<ModelsListState> {
    Ok(ModelsListState::default())
}
