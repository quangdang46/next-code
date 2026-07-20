//! Stub of upstream `xai-grok-shell::leader` — pager ACP leader cluster types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeaderConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct LeaderHandle;

impl LeaderHandle {
    pub fn new(_config: LeaderConfig) -> Self {
        Self
    }
}

pub mod protocol {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct LeaderMessage {
        pub payload: String,
    }
}

pub mod transport {
    #[derive(Debug, Default)]
    pub struct LeaderTransport;
}
