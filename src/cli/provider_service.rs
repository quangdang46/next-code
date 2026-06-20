use std::sync::Arc;
use anyhow::Result;
use jcode_provider_service::boot;
use jcode_provider_service::catalog::{CatalogService, ModelInfo, ProviderInfo, InMemoryCatalog};
use jcode_provider_service::store::{DefaultProviderService, InMemoryCredentialStore};
use jcode_provider_service::integration::InMemoryIntegration;
use jcode_provider_service::service::ProviderService;
use crate::bus::{Bus, BusEvent};

pub struct ProviderCliService { svc: DefaultProviderService }

impl ProviderCliService {
    pub fn new() -> Result<Self> {
        let bus = Bus::global();
        let catalog_on_updated = {
            let bus = bus;
            move || bus.publish(BusEvent::CatalogUpdated)
        };
        let integration_on_updated = {
            let bus = bus;
            move || bus.publish(BusEvent::IntegrationUpdated)
        };
        let catalog = Arc::new(
            InMemoryCatalog::new()
                .with_on_updated(Box::new(catalog_on_updated)),
        );
        let integration = Arc::new(
            InMemoryIntegration::new()
                .with_on_updated(Box::new(integration_on_updated)),
        );
        let credential = Arc::new(InMemoryCredentialStore::new());
        let svc = DefaultProviderService::new(catalog, integration, credential);
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { boot::register_builtins::<jcode_keyring_store::MockKeyringStore>(svc.catalog(), svc.integration()).await })?;
        Ok(Self { svc })
    }
    pub fn list_providers(&self) -> Result<Vec<ProviderInfo>> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { self.svc.catalog().list_providers().await.map_err(Into::into) })
    }
    pub fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let p = self.list_providers()?;
        Ok(p.into_iter().flat_map(|p| p.models).collect())
    }
}
