//! Adapted from upstream `xai-grok-voice` `auth.rs`. The bearer-resolution
//! trait shape is kept so the pager can build a `SharedVoiceAuth` the same
//! way upstream does; `require_bearer` (upstream, `#[cfg(feature = "audio")]`)
//! is dropped since this build has no STT network client to call it.

use std::future::{Future, ready};
use std::pin::Pin;
use std::sync::Arc;

pub trait VoiceAuthProvider: std::fmt::Debug + Send + Sync + 'static {
    fn bearer(&self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>>;
}

/// Shared provider handed to the voice pipeline.
pub type SharedVoiceAuth = Arc<dyn VoiceAuthProvider>;

/// A fixed bearer that never refreshes.
///
/// Used by tests / callers with no `AuthManager` — only a raw API key.
pub struct StaticVoiceAuth(pub String);

impl std::fmt::Debug for StaticVoiceAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StaticVoiceAuth")
            .field(&"<redacted>")
            .finish()
    }
}

impl VoiceAuthProvider for StaticVoiceAuth {
    fn bearer(&self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
        Box::pin(ready(Some(self.0.clone())))
    }
}

impl StaticVoiceAuth {
    /// Build a [`SharedVoiceAuth`] from a static key, trimming whitespace and
    /// rejecting an empty value.
    pub fn shared(key: impl Into<String>) -> Option<SharedVoiceAuth> {
        let key = key.into().trim().to_string();
        if key.is_empty() {
            return None;
        }
        Some(Arc::new(Self(key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn static_provider_resolves() {
        let provider = StaticVoiceAuth::shared("  sk-test  ").unwrap();
        assert_eq!(provider.bearer().await.as_deref(), Some("sk-test"));
    }

    #[test]
    fn static_provider_rejects_empty() {
        assert!(StaticVoiceAuth::shared("   ").is_none());
    }
}
