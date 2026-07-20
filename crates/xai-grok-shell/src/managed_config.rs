//! Stub of upstream `xai-grok-shell::managed_config`.

use std::sync::Arc;

use crate::agent::config::Config;
use crate::auth::AuthManager;

#[derive(Debug, thiserror::Error)]
pub enum ManagedConfigError {
    #[error("managed-config stub: {0}")]
    Stub(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedConfigSync {
    Synced,
    Skipped,
    Failed,
}

/// Upstream loads managed (fleet) config overlays; stub returns empty Ok.
pub fn load() -> anyhow::Result<Config> {
    Ok(Config::default())
}

pub fn load_optional() -> Option<Config> {
    None
}

pub async fn ensure_managed_policy_present(_auth: &Arc<AuthManager>) {}

pub fn managed_policy_gate() -> Result<(), String> {
    Ok(())
}

pub async fn sync() -> Result<bool, ManagedConfigError> {
    Ok(false)
}

pub fn clear_orphan() {}

pub fn has_principal() -> bool {
    false
}
