//! Stub: Provider impl for the base-crate OpenAIProvider.
//!
//! The real `impl Provider for OpenAIProvider` lives in the downstream
//! `jcode-provider-openai-runtime` crate. This file is kept as a tiny stub
//! so the `#[path]` module declaration in `openai.rs` resolves and the
//! `OpenAIProvider` struct satisfies the `Provider` trait bounds.

use super::OpenAIProvider;
use super::{EventStream, Provider};
use anyhow::Result;
use async_trait::async_trait;
use jcode_message_types::{Message, StreamEvent, ToolDefinition};
use std::sync::Arc;

#[async_trait]
impl Provider for OpenAIProvider {
    async fn complete(
        &self,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        _system: &str,
        _resume_session_id: Option<&str>,
    ) -> Result<EventStream> {
        // The real runtime is in the downstream jcode-provider-openai-runtime
        // crate. If reached, the composition root did not register it.
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tx.try_send(Ok(StreamEvent::Error {
            message: "OpenAI provider runtime not available: the external crate was not registered by the composition root".into(),
            retry_after_secs: None,
        }))
        .ok();
        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(super::OpenAIProvider {
            client: self.client.clone(),
            credentials: Arc::clone(&self.credentials),
            credential_mode: Arc::clone(&self.credential_mode),
            model: Arc::clone(&self.model),
            prompt_cache_key: self.prompt_cache_key.clone(),
            prompt_cache_retention: self.prompt_cache_retention.clone(),
            max_output_tokens: self.max_output_tokens,
            reasoning_effort: Arc::clone(&self.reasoning_effort),
            service_tier: Arc::clone(&self.service_tier),
            native_compaction_mode: self.native_compaction_mode,
            native_compaction_threshold_tokens: self.native_compaction_threshold_tokens,
            transport_mode: Arc::clone(&self.transport_mode),
            websocket_cooldowns: Arc::clone(&self.websocket_cooldowns),
            websocket_failure_streaks: Arc::clone(&self.websocket_failure_streaks),
            persistent_ws: Arc::clone(&self.persistent_ws),
            temperature: Arc::clone(&self.temperature),
        })
    }
}
