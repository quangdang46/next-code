//! Façade stub of upstream `xai-grok-shell::auth` — third highest-frequency
//! pager import group (`AuthMeta` 30 hits, `GateInfo` 17, `AuthManager` 5).
//! `meta.rs` is vendored near-verbatim (small, self-contained). `AuthManager`
//! (upstream ~107k-line `manager.rs`, token refresh / OIDC / device-code /
//! keyring persistence) is reduced to a construction-shaped stub — no real
//! credential storage or refresh loop in this compile-stub layer.

mod credential_provider;
mod meta;

pub use credential_provider::CredentialProvider;
pub use meta::{AuthMeta, GateInfo};

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub fn default_coding_data_retention_opt_out() -> bool {
    true
}

/// Simplified stand-in for upstream `GrokComConfig` (OIDC/OAuth2/external
/// provider sub-configs dropped — see module doc).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrokComConfig {
    #[serde(default)]
    pub grok_ws_origin: String,
    #[serde(default)]
    pub grok_ws_url: String,
    #[serde(default)]
    pub token_header: String,
    #[serde(default)]
    pub auth_provider_command: Option<String>,
    #[serde(default)]
    pub auth_provider_label: Option<String>,
}

impl GrokComConfig {
    pub fn auth_scope(&self) -> String {
        "default".to_string()
    }
}

/// Simplified stand-in for upstream `AuthManager`. No in-memory bearer
/// cache, no disk-backed auth file, no token refresh — construction-shaped
/// only, matching the constructor signature the future pager calls
/// (`AuthManager::new(grok_home, grok_com_config)`).
#[derive(Debug, Clone)]
pub struct AuthManager {
    path: PathBuf,
    grok_com_config: GrokComConfig,
}

impl AuthManager {
    pub fn new(grok_home: &Path, grok_com_config: GrokComConfig) -> Self {
        Self {
            path: grok_home.join("auth.json"),
            grok_com_config,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn grok_com_config(&self) -> &GrokComConfig {
        &self.grok_com_config
    }

    pub fn configure_refresher(&self) {}

    pub fn start_system_power_listener(&self) {}
}
