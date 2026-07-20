//! Stub of upstream `xai-grok-shell::agent::models` — `MvpAgent` shape.

use crate::agent::config::Config;

/// Thin stand-in for the in-process agent handle the pager spawns.
#[derive(Debug, Default)]
pub struct MvpAgent {
    pub config: Config,
}

impl MvpAgent {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}
