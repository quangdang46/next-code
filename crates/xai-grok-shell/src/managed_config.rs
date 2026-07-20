//! Stub of upstream `xai-grok-shell::managed_config`.

use crate::agent::config::Config;

/// Upstream loads managed (fleet) config overlays; stub returns empty Ok.
pub fn load() -> anyhow::Result<Config> {
    Ok(Config::default())
}

pub fn load_optional() -> Option<Config> {
    None
}
