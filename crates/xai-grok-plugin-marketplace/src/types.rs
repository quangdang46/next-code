//! Marketplace source / entry types (narrow stub of upstream `types.rs`).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A configured marketplace source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSource {
    /// User-facing display name.
    pub name: String,
    /// How to access the marketplace.
    pub kind: SourceKind,
}

/// How to access a marketplace source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceKind {
    /// A local directory containing a `plugins/` subdirectory.
    Local { path: PathBuf },
    /// A git repo. Cloned/pulled to a persistent cache on refresh.
    Git { url: String, branch: Option<String> },
}

/// A plugin found by scanning a marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceEntry {
    /// Plugin name (from manifest or index).
    pub name: String,
    /// Version string (from manifest).
    pub version: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Category (from index).
    pub category: Option<String>,
    /// Author name.
    pub author: Option<String>,
    /// Tags/keywords.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Matcher keywords.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Matcher domains.
    #[serde(default)]
    pub domains: Vec<String>,
    /// Homepage URL.
    pub homepage: Option<String>,
    /// Relative path within marketplace.
    pub relative_path: String,
    /// Number of skills discovered.
    pub skill_count: usize,
    /// Whether the plugin has hooks.
    pub has_hooks: bool,
    /// Whether the plugin has agents.
    pub has_agents: bool,
    /// Whether the plugin has MCP configuration.
    pub has_mcp: bool,
    /// Remote git URL for URL-sourced plugins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
    /// Git ref for remote URL sources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_subdir: Option<String>,
    /// Structured inventory from the marketplace catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<xai_hooks_plugins_types::PluginComponents>,
}

/// Result of a marketplace scan.
#[derive(Debug, Clone)]
pub struct MarketplaceScan {
    /// Discovered plugins.
    pub entries: Vec<MarketplaceEntry>,
    /// Whether a `plugin-index.json` catalog was loaded.
    pub catalog_loaded: bool,
}
