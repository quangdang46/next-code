use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    OAuth,
    ApiKey { env_var: String },
    Env { env_var: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginProvider {
    pub id: String,
    pub name: String,
    pub auth_methods: Vec<AuthMethod>,
    pub env_keys: Vec<String>,
}

/// Tracks which providers have credentials available and what type.
#[derive(Debug, Clone, Default)]
pub struct Integration {
    providers: HashMap<String, LoginProvider>,
    connections: HashMap<String, ConnectionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionInfo {
    OAuth { label: String },
    ApiKey { env_var: String },
    Env { env_var: String },
    NotConfigured,
}

impl Integration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_provider(&mut self, provider: LoginProvider) {
        self.providers.insert(provider.id.clone(), provider);
    }

    pub fn set_connection(&mut self, provider_id: &str, conn: ConnectionInfo) {
        self.connections.insert(provider_id.to_string(), conn);
    }

    pub fn connection_for(&self, provider_id: &str) -> ConnectionInfo {
        self.connections
            .get(provider_id)
            .cloned()
            .unwrap_or(ConnectionInfo::NotConfigured)
    }

    pub fn available_auth_methods(&self, provider_id: &str) -> Vec<&AuthMethod> {
        self.providers
            .get(provider_id)
            .map(|p| p.auth_methods.iter().collect())
            .unwrap_or_default()
    }

    pub fn has_any_credential(&self, provider_id: &str) -> bool {
        !matches!(self.connection_for(provider_id), ConnectionInfo::NotConfigured)
    }

    pub fn connected_providers(&self) -> Vec<String> {
        self.connections
            .iter()
            .filter(|(_, c)| !matches!(c, ConnectionInfo::NotConfigured))
            .map(|(id, _)| id.clone())
            .collect()
    }
}
