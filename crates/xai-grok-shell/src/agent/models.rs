//! Stub of upstream `xai-grok-shell::agent::models`.

use std::sync::Arc;

use crate::agent::config::Config;
use crate::auth::{AuthManager, GrokComConfig};
use crate::util::config::RemoteSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshStrategy {
    Online,
    Offline,
    OnlineIfUncached,
}

#[derive(Debug, Default)]
pub struct ModelsManager {
    pub config: Config,
}

impl ModelsManager {
    pub fn from_config(
        cfg: &Config,
        _prefetched: Option<()>,
        _auth: Arc<AuthManager>,
    ) -> Result<Self, String> {
        Ok(Self {
            config: cfg.clone(),
        })
    }

    pub async fn list_models(&self, _strategy: RefreshStrategy) {}

    pub fn start_auth_refresh_watcher<N>(&self, _notifier: N) {}
}

#[derive(Debug, Default)]
pub struct EarlyPrefetchResult {
    pub settings: Option<RemoteSettings>,
}

#[derive(Debug, Default)]
pub struct EarlyPrefetchHandle {
    finished: bool,
    result: EarlyPrefetchResult,
}

impl EarlyPrefetchHandle {
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn join(self) -> Result<EarlyPrefetchResult, String> {
        Ok(self.result)
    }
}

/// Upstream may return a GrokAuth; Face stub aliases the auth snapshot.
pub type GrokAuth = crate::auth::AuthSnapshot;

pub fn start_early_prefetch_with_auth(_auth: Option<GrokAuth>) -> Option<EarlyPrefetchHandle> {
    Some(EarlyPrefetchHandle {
        finished: true,
        result: EarlyPrefetchResult::default(),
    })
}

pub fn start_early_prefetch(_grok_com_config: Option<GrokComConfig>) -> Option<EarlyPrefetchHandle> {
    Some(EarlyPrefetchHandle {
        finished: true,
        result: EarlyPrefetchResult::default(),
    })
}
