//! Stub of upstream `xai-grok-agent::plugins::manifest`. `PluginManifest`
//! carries only the fields the future pager touches; `load_manifest` is a
//! no-op that always reports `NotFound` (no real `plugin.json`/`toml`
//! parsing in this compile-stub layer).

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<Author>,
}

#[derive(Debug, Clone)]
pub enum ManifestLoadResult {
    Found(PluginManifest),
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("manifest stub: not implemented")]
    NotImplemented,
}

/// Upstream parses `plugin.json`/`.grok-plugin.toml` from `plugin_root`;
/// this stub never reads disk and always reports `NotFound`.
pub fn load_manifest(_plugin_root: &Path) -> Result<ManifestLoadResult, ManifestError> {
    Ok(ManifestLoadResult::NotFound)
}

/// Upstream derives a display name from the install dirname; stub mirrors
/// the signature and returns the dirname unchanged.
pub fn name_from_dirname(dirname: &str) -> String {
    dirname.to_string()
}
