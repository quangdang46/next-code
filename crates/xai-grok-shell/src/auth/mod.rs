//! Façade stub of upstream `xai-grok-shell::auth` — third highest-frequency
//! pager import group (`AuthMeta` 30 hits, `GateInfo` 17, `AuthManager` 5).
//! `meta.rs` is vendored near-verbatim (small, self-contained). `AuthManager`
//! (upstream ~107k-line `manager.rs`, token refresh / OIDC / device-code /
//! keyring persistence) is reduced to a construction-shaped stub — no real
//! credential storage or refresh loop in this compile-stub layer.

pub mod credential_provider;
mod meta;

pub use credential_provider::{
    CredentialProvider, StorageClient, build_default_otel_layer_config,
    build_storage_client_for_proxy,
};
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

    pub fn configure_refresher(
        self: &std::sync::Arc<Self>,
        _auth_provider_command: Option<String>,
        _diagnostic_uploader: Option<()>,
    ) -> bool {
        true
    }

    pub fn start_system_power_listener(self: &std::sync::Arc<Self>) {}

    pub fn refresh_notifier(&self) {}

    pub async fn auth(&self) -> anyhow::Result<AuthSnapshot> {
        Ok(AuthSnapshot::default())
    }
}

/// Lightweight auth snapshot returned by [`AuthManager::auth`].
#[derive(Debug, Clone, Default)]
pub struct AuthSnapshot {
    pub zdr_team: bool,
    pub key: String,
}

impl AuthSnapshot {
    pub fn is_zdr_team(&self) -> bool {
        self.zdr_team
    }
}

/// Best-effort interactive/non-interactive auth for CLI entrypoints.
pub async fn try_ensure_fresh_auth(
    _config: &GrokComConfig,
) -> Option<AuthSnapshot> {
    Some(AuthSnapshot::default())
}

/// Ensure auth for restore / non-interactive flows.
/// Returns `Ok(Some(auth))` when a usable credential is available.
pub async fn ensure_authenticated_or_noninteractive(
    _config: &GrokComConfig,
    _has_deployment_key: bool,
    _hint: Option<&str>,
) -> anyhow::Result<Option<AuthSnapshot>> {
    Ok(Some(AuthSnapshot::default()))
}

pub fn read_auth_json(_path: &Path) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::json!({}))
}

pub fn lookup_auth(
    _store: &serde_json::Value,
    _scope: &str,
) -> Option<serde_json::Value> {
    Some(serde_json::json!({}))
}

/// Shared API-key credential provider handle for voice / STT paths.
pub fn shared_api_key_provider(
    _auth: std::sync::Arc<AuthManager>,
) -> xai_grok_tools::types::SharedApiKeyProvider {
    std::sync::Arc::new(StubApiKeyProvider)
}

struct StubApiKeyProvider;

impl xai_grok_tools::types::ApiKeyProvider for StubApiKeyProvider {
    fn current_api_key(&self) -> Option<String> {
        None
    }
}
