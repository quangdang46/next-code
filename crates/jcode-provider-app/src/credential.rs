use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub id: String,
    pub provider_id: String,
    pub label: String,
    pub credential_type: CredentialType,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialType {
    OAuth { access_token: String, refresh_token: Option<String>, expires_at: Option<String> },
    ApiKey { key: String },
}

/// Simple in-memory credential store. Will be backed by jcode-keyring-store / SQLite later.
#[derive(Debug, Clone, Default)]
pub struct CredentialStore {
    credentials: Vec<StoredCredential>,
}

impl CredentialStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, cred: StoredCredential) {
        self.credentials.push(cred);
    }

    pub fn list(&self, provider_id: &str) -> Vec<&StoredCredential> {
        self.credentials.iter().filter(|c| c.provider_id == provider_id).collect()
    }

    pub fn remove(&mut self, id: &str) {
        self.credentials.retain(|c| c.id != id);
    }
}
