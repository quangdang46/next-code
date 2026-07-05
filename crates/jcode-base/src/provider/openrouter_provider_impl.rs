//! Stub: Provider impl for the base-crate OpenRouterProvider.
//!
//! The real `impl Provider for OpenRouterProvider` lives in the downstream
//! `jcode-provider-openrouter-runtime` crate. This file is kept as a tiny
//! stub so the `#[path]` module declaration in `openrouter.rs` resolves.

use super::OpenRouterProvider;
use super::{EventStream, Provider};
use anyhow::Result;
use async_trait::async_trait;
use jcode_message_types::{Message, StreamEvent, ToolDefinition};
use std::sync::Arc;

#[async_trait]
impl Provider for OpenRouterProvider {
    async fn complete(
        &self,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        _system: &str,
        _resume_session_id: Option<&str>,
    ) -> Result<EventStream> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tx.try_send(Ok(StreamEvent::Error {
            message: "OpenRouter provider runtime not available: the external crate was not registered by the composition root".into(),
            retry_after_secs: None,
        }))
        .ok();
        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    fn name(&self) -> &str {
        "openrouter"
    }

    fn fork(&self) -> Arc<dyn Provider> {
        // The downstream runtime crate has the real Provider impl and handles
        // forking. This stub returns a minimal fresh OpenRouterProvider.
        match super::OpenRouterProvider::new() {
            Ok(p) => Arc::new(p),
            Err(_) => {
                let empty_models_cache = super::ModelsCache::default();
                let empty_refresh = super::ModelCatalogRefreshState::default();
                let empty_routing = super::ProviderRouting::default();
                let empty_cache = super::EndpointsCache::default();
                let empty_tracker = super::EndpointRefreshTracker::default();
                Arc::new(super::OpenRouterProvider {
                    client: reqwest::Client::new(),
                    model: Arc::new(tokio::sync::RwLock::new(String::new())),
                    reasoning_effort: Arc::new(tokio::sync::RwLock::new(None)),
                    api_base: String::new(),
                    auth: super::ProviderAuth::None { label: String::new() },
                    supports_provider_features: false,
                    supports_model_catalog: false,
                    profile_id: None,
                    reasoning_effort_support: None,
                    max_tokens: None,
                    extra_body: None,
                    static_models: Vec::new(),
                    static_context_limits: std::collections::HashMap::new(),
                    send_openrouter_headers: false,
                    models_cache: Arc::new(tokio::sync::RwLock::new(empty_models_cache)),
                    model_catalog_refresh: Arc::new(std::sync::Mutex::new(empty_refresh)),
                    provider_routing: Arc::new(tokio::sync::RwLock::new(empty_routing)),
                    provider_pin: Arc::new(std::sync::Mutex::new(None)),
                    endpoints_cache: Arc::new(tokio::sync::RwLock::new(empty_cache)),
                    endpoint_refresh: Arc::new(std::sync::Mutex::new(empty_tracker)),
                })
            }
        }
    }
}
