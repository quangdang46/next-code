//! Stub of upstream `xai-grok-shell::agent::init`.

use crate::agent::config::Config;
use crate::agent::models::MvpAgent;

/// Upstream builds a full agent stack; this returns a bare handle.
pub fn init(config: Config) -> anyhow::Result<MvpAgent> {
    Ok(MvpAgent::new(config))
}
