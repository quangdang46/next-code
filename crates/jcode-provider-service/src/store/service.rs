//! Default [`ProviderService`] and [`RouteResolver`] implementations.
//!
//! Phase 3 + 6 of the master plan. This is the runtime facade the session
//! runner (and eventually the CLI) holds behind an
//! `Arc<dyn ProviderService>`. It composes the catalog, integration, and
//! credential services into a single handle and resolves
//! `(provider, model)` requests into concrete [`Route`]s.
//!
//! The default route construction is intentionally minimal — the real
//! per-provider route templates land when each provider adopts the new
//! `Route`-based path in Phase 6. For now we emit a route with
//! `protocol = openai-chat` as a placeholder so the wiring compiles
//! end-to-end.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use jcode_llm_core::endpoint::{Endpoint, PathSpec};
use jcode_llm_core::route::Route;
use jcode_llm_core::schema::ModelRef;

use crate::catalog::CatalogService;
use crate::credential::CredentialService;
use crate::integration::ConnectionStatus;
use crate::integration::IntegrationService;
use crate::service::{ProviderService, ResolveError, ResolvedRoute, RouteResolver};
use crate::types::{ModelId, ProviderId, ProviderProfile};

/// Default composite service. Wraps an existing catalog, integration, and
/// credential service.
pub struct DefaultProviderService {
    catalog: Arc<dyn CatalogService>,
    integration: Arc<dyn IntegrationService>,
    credentials: Arc<dyn CredentialService>,
}

impl DefaultProviderService {
    pub fn new(
        catalog: Arc<dyn CatalogService>,
        integration: Arc<dyn IntegrationService>,
        credentials: Arc<dyn CredentialService>,
    ) -> Self {
        Self {
            catalog,
            integration,
            credentials,
        }
    }
}

impl ProviderService for DefaultProviderService {
    fn catalog(&self) -> &dyn CatalogService {
        self.catalog.as_ref()
    }

    fn integration(&self) -> &dyn IntegrationService {
        self.integration.as_ref()
    }

    fn credentials(&self) -> &dyn CredentialService {
        self.credentials.as_ref()
    }

    fn resolver(&self) -> &dyn RouteResolver {
        self
    }
}

#[async_trait]
impl RouteResolver for DefaultProviderService {
    async fn resolve_route(
        &self,
        provider: &ProviderId,
        model: &ModelId,
    ) -> Result<ResolvedRoute, ResolveError> {
        // 1. Verify the provider is in the catalog.
        let _info = self
            .catalog
            .provider(provider)
            .await
            .map_err(|_| ResolveError::UnknownProvider(provider.clone()))?;

        // 2. Verify the provider is connected.
        let status = self
            .integration
            .detect(provider)
            .await
            .map_err(ResolveError::Integration)?;
        if !status.is_connected() {
            return Err(ResolveError::NotConnected(provider.clone()));
        }

        // 3. Build the route.
        let route = build_default_route(provider, model);
        Ok(ResolvedRoute {
            provider: provider.clone(),
            model: model.clone(),
            route,
        })
    }

    async fn resolve_profile(
        &self,
        profile: &ProviderProfile,
        default_model: Option<&ModelId>,
    ) -> Result<(ProviderId, ModelId), ResolveError> {
        // For now we only support ById — the others (Named, ByLabel, WithAuth)
        // land in Phase 4 (CLI) when we wire the config parser.
        let id = match profile {
            ProviderProfile::ById { id } => id.clone(),
            ProviderProfile::WithAuth { id, .. } => id.clone(),
            ProviderProfile::Named { .. } | ProviderProfile::ByLabel { .. } => {
                return Err(ResolveError::UnknownProvider(ProviderId::from(
                    profile.describe(),
                )))
            }
        };
        // If the caller pinned a model, use it. Otherwise ask the catalog
        // for the provider's first model.
        let model = if let Some(m) = default_model {
            m.clone()
        } else {
            let models = self.catalog.models(&id).await?;
            models
                .first()
                .map(|m| m.id.clone())
                .ok_or(crate::catalog::CatalogError::NoModels(id.clone()))?
        };
        Ok((id, model))
    }
}

/// Build a placeholder route. Phase 6 will replace this with the real
/// per-provider route templates. For now we emit a `Route` with the
/// provider's standard base URL and the model name in the body overlay.
fn build_default_route(provider: &ProviderId, model: &ModelId) -> Route {
    let base_url = default_base_url(provider);
    let mut defaults = HashMap::new();
    defaults.insert("temperature".into(), serde_json::json!(0.0));

    Route {
        id: format!("{}/{}", provider, model),
        provider: ModelRef {
            provider_id: jcode_llm_core::schema::ProviderId::from(provider.as_str()),
            id: model.as_str().to_string(),
            variant: None,
        },
        protocol: default_protocol(provider),
        endpoint: Endpoint {
            base_url,
            path: PathSpec::Static(default_path(provider)),
            query: None,
        },
        auth: HashMap::new(),
        framing: jcode_llm_core::framing::Framing::Sse,
        transport: jcode_llm_core::transport::Transport::Http,
        defaults,
        body_overlay: Some(serde_json::json!({ "model": model.as_str() })),
    }
}

fn default_base_url(provider: &ProviderId) -> String {
    match provider.as_str() {
        "anthropic" => "https://api.anthropic.com".into(),
        "openai" => "https://api.openai.com".into(),
        "gemini" => "https://generativelanguage.googleapis.com".into(),
        "openrouter" => "https://openrouter.ai/api".into(),
        "bedrock" => "https://bedrock-runtime.us-east-1.amazonaws.com".into(),
        "copilot" => "https://api.githubcopilot.com".into(),
        _ => "https://localhost".into(),
    }
}

fn default_path(provider: &ProviderId) -> String {
    match provider.as_str() {
        "anthropic" => "/v1/messages".into(),
        "openai" => "/v1/chat/completions".into(),
        "gemini" => "/v1beta/models/{model}:generateContent".into(),
        "openrouter" => "/v1/chat/completions".into(),
        "bedrock" => "/model/{model}/invoke".into(),
        "copilot" => "/chat/completions".into(),
        _ => "/".into(),
    }
}

fn default_protocol(provider: &ProviderId) -> String {
    match provider.as_str() {
        "anthropic" => "anthropic-messages-2023-01-01".into(),
        "openai" | "openrouter" | "copilot" => "openai-chat-2024".into(),
        "gemini" => "gemini-1.5".into(),
        "bedrock" => "bedrock-converse-2024".into(),
        _ => "openai-chat-2024".into(),
    }
}

/// Convenience: assert that the given provider has at least one
/// credential, and return the connection status.
pub async fn require_connected(
    integration: &dyn IntegrationService,
    provider: &ProviderId,
) -> Result<ConnectionStatus, ResolveError> {
    let status = integration
        .detect(provider)
        .await
        .map_err(ResolveError::Integration)?;
    if !status.is_connected() {
        return Err(ResolveError::NotConnected(provider.clone()));
    }
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{InMemoryCatalog, ModelInfo, ModelTier, ProviderInfo};
    use crate::credential::{CredentialService, CredentialType};
    use crate::integration::{AuthMethod, LoginProvider};
    use crate::store::{InMemoryCredentialStore, KeyringCredentialStore, PersistentIntegration};
    use jcode_keyring_store::MockKeyringStore;

    async fn fixture() -> (
        Arc<dyn CatalogService>,
        Arc<dyn IntegrationService>,
        Arc<dyn CredentialService>,
    ) {
        let catalog = Arc::new(InMemoryCatalog::new());
        catalog
            .register_provider(ProviderInfo {
                id: "anthropic".into(),
                name: "Anthropic".into(),
                enabled: true,
                is_connected: true,
                models: vec![ModelInfo {
                    id: "claude-haiku-4-5".into(),
                    provider: "anthropic".into(),
                    name: "Claude Haiku 4.5".into(),
                    cost_per_million_input: Some(0.8),
                    cost_per_million_output: Some(4.0),
                    context_window: 200_000,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    tier: Some(ModelTier::Nano),
                }],
            })
            .await
            .unwrap();

        let keyring = Arc::new(MockKeyringStore::new());
        let creds: Arc<dyn CredentialService> = Arc::new(KeyringCredentialStore::new(keyring));
        let integration: Arc<dyn IntegrationService> =
            Arc::new(PersistentIntegration::<MockKeyringStore>::new(creds.clone()));
        integration
            .register(LoginProvider {
                id: "anthropic".into(),
                label: "Anthropic".into(),
                auth_methods: vec![AuthMethod::ApiKey {
                    env_var: "ANTHROPIC_API_KEY".into(),
                }],
                env_keys: vec!["ANTHROPIC_API_KEY".into()],
                oauth_preferred: false,
            })
            .await
            .unwrap();
        creds
            .upsert(crate::credential::Credential::new(
                "anthropic".into(),
                "default",
                CredentialType::ApiKey {
                    key: "sk-test".into(),
                },
            ))
            .await
            .unwrap();

        (catalog, integration, creds)
    }

    #[tokio::test]
    async fn resolve_route_returns_prepared_route() {
        let (cat, int, creds) = fixture().await;
        let svc = DefaultProviderService::new(cat, int, creds);
        let r = svc
            .resolver()
            .resolve_route(&"anthropic".into(), &"claude-haiku-4-5".into())
            .await
            .unwrap();
        assert_eq!(r.provider.as_str(), "anthropic");
        assert_eq!(r.model.as_str(), "claude-haiku-4-5");
        assert_eq!(r.route.protocol, "anthropic-messages-2023-01-01");
        assert_eq!(r.route.endpoint.base_url, "https://api.anthropic.com");
    }

    #[tokio::test]
    async fn resolve_route_errors_when_not_connected() {
        let (cat, int, creds) = fixture().await;
        // Wipe the credential so detect() returns NotConfigured.
        let creds_clone = creds.clone();
        let all = creds.list(&"anthropic".into()).await.unwrap();
        for c in all {
            creds_clone.delete(&c.id).await.unwrap();
        }
        let svc = DefaultProviderService::new(cat, int, creds_clone);
        let err = svc
            .resolver()
            .resolve_route(&"anthropic".into(), &"claude-haiku-4-5".into())
            .await
            .unwrap_err();
        assert!(matches!(err, ResolveError::NotConnected(_)));
    }

    #[tokio::test]
    async fn resolve_route_errors_for_unknown_provider() {
        let (cat, int, creds) = fixture().await;
        let svc = DefaultProviderService::new(cat, int, creds);
        let err = svc
            .resolver()
            .resolve_route(&"mystery".into(), &"m".into())
            .await
            .unwrap_err();
        assert!(matches!(err, ResolveError::UnknownProvider(_)));
    }

    #[tokio::test]
    async fn resolve_profile_uses_provided_model() {
        let (cat, int, creds) = fixture().await;
        let svc = DefaultProviderService::new(cat, int, creds);
        let profile = ProviderProfile::ById {
            id: "anthropic".into(),
        };
        let (p, m) = svc
            .resolver()
            .resolve_profile(&profile, Some(&"claude-haiku-4-5".into()))
            .await
            .unwrap();
        assert_eq!(p.as_str(), "anthropic");
        assert_eq!(m.as_str(), "claude-haiku-4-5");
    }

    #[tokio::test]
    async fn resolve_profile_defaults_to_first_model() {
        let (cat, int, creds) = fixture().await;
        let svc = DefaultProviderService::new(cat, int, creds);
        let profile = ProviderProfile::ById {
            id: "anthropic".into(),
        };
        let (p, m) = svc.resolver().resolve_profile(&profile, None).await.unwrap();
        assert_eq!(p.as_str(), "anthropic");
        assert_eq!(m.as_str(), "claude-haiku-4-5");
    }
}
