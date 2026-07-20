//! Stub of upstream `xai-grok-shell::auth::credential_provider`.

use std::sync::Arc;

use crate::auth::AuthManager;

#[derive(Debug, Clone, Default)]
pub struct CredentialProvider;

impl CredentialProvider {
    pub fn new() -> Self {
        Self
    }
}

/// Opaque storage/HTTP client handle used by restore + trace upload paths.
#[derive(Debug, Clone, Default)]
pub struct StorageClient;

impl StorageClient {
    pub fn new() -> Self {
        Self
    }
}

/// Upstream builds a live OTEL auth provider; stub returns empty config.
pub fn build_default_otel_layer_config() -> xai_grok_telemetry::otel_layer::OtelLayerConfig {
    xai_grok_telemetry::otel_layer::OtelLayerConfig::default()
}

/// Build a storage client pointed at the CLI chat proxy (no-op stub).
pub fn build_storage_client_for_proxy(
    _proxy_base: &str,
    _deployment_key: Option<String>,
    _alpha_test_key: Option<String>,
    _auth: Option<Arc<AuthManager>>,
    _extra: Option<()>,
    _session_id: Option<String>,
    _user_agent: &str,
) -> StorageClient {
    StorageClient::new()
}
