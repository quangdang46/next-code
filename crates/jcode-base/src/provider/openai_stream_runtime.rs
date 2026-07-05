//! Stub: items from the downstream OpenAI stream runtime crate.
//!
//! The real implementations live in `jcode-provider-openai-runtime` with
//! `pub(super)` visibility. These local stubs satisfy the `use` imports in
//! `openai.rs`; the items are never referenced in code body.

use crate::auth::codex::CodexCredentials;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Placeholder for downstream `PersistentWsResult`.
#[allow(dead_code)]
pub(crate) enum PersistentWsResult {
    Success,
    NotAvailable,
    Failed(String),
}

/// Placeholder for downstream `is_retryable_error`.
#[allow(dead_code)]
pub(crate) fn is_retryable_error(_error_str: &str) -> bool {
    false
}

/// Placeholder for downstream `openai_access_token`.
#[allow(dead_code)]
pub(crate) async fn openai_access_token(
    _credentials: &Arc<RwLock<CodexCredentials>>,
) -> anyhow::Result<String> {
    anyhow::bail!("OpenAI stream runtime not available")
}
