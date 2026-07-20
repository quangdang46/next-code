//! Stub of upstream `xai-grok-shell::agent::init`.

use std::sync::Arc;

use crate::agent::config::Config;
use crate::agent::models::ModelsManager;
use crate::auth::AuthManager;

/// Upstream builds a full agent stack; this returns defaults.
pub fn bootstrap(
    cfg: &Config,
    auth_manager: &Arc<AuthManager>,
    _prefetched: Option<()>,
) -> Result<(Config, ModelsManager), String> {
    let models = ModelsManager::from_config(cfg, None, auth_manager.clone())?;
    Ok((cfg.clone(), models))
}

pub fn init(config: Config) -> anyhow::Result<crate::agent::mvp_agent::MvpAgent> {
    Ok(crate::agent::mvp_agent::MvpAgent::new_stub(config))
}

pub fn update_telemetry_config(_cfg: &Config, _auth: &Arc<AuthManager>) {}

