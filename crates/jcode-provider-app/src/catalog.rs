use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub type CategoryId = String;
pub type ModelId = String;
pub type ProviderCategory = String;

/// A provider entry in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEntry {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub is_connected: bool,
}

/// A model entry in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: ModelId,
    pub category_id: ProviderCategory,
    pub name: String,
    pub cost_per_million_input: f64,
    pub cost_per_million_output: f64,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
    pub context_window: u64,
}

/// The in-memory provider/model catalog.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    providers: HashMap<String, ProviderEntry>,
    models: Vec<ModelEntry>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_provider(&mut self, entry: ProviderEntry) {
        self.providers.insert(entry.id.clone(), entry);
    }

    pub fn add_model(&mut self, entry: ModelEntry) {
        self.models.push(entry);
    }

    pub fn providers(&self) -> Vec<&ProviderEntry> {
        self.providers.values().collect()
    }

    pub fn provider(&self, id: &str) -> Option<&ProviderEntry> {
        self.providers.get(id)
    }

    pub fn models(&self) -> &[ModelEntry] {
        &self.models
    }

    pub fn models_for_provider(&self, provider_id: &str) -> Vec<&ModelEntry> {
        self.models.iter().filter(|m| m.category_id == provider_id).collect()
    }

    pub fn connected_providers(&self) -> Vec<&ProviderEntry> {
        self.providers.values().filter(|p| p.is_connected).collect()
    }

    pub fn set_connected(&mut self, id: &str, connected: bool) {
        if let Some(entry) = self.providers.get_mut(id) {
            entry.is_connected = connected;
        }
    }
}
