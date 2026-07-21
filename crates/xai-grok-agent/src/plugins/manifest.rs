//! Stub of upstream `xai-grok-agent::plugins::manifest`.

use std::path::{Path, PathBuf};

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

impl PluginManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("name is required".into());
        }
        Ok(())
    }

    pub fn skill_dirs(&self, _root: &Path) -> Vec<PathBuf> {
        vec![]
    }

    pub fn command_dirs(&self, _root: &Path) -> Vec<PathBuf> {
        vec![]
    }

    pub fn agent_dirs(&self, _root: &Path) -> Vec<PathBuf> {
        vec![]
    }

    pub fn hooks_path(&self, _root: &Path) -> Option<PathBuf> {
        None
    }

    pub fn inline_hooks(&self) -> Option<serde_json::Value> {
        None
    }

    pub fn mcp_config_path(&self, _root: &Path) -> Option<PathBuf> {
        None
    }

    pub fn inline_mcp_servers(&self) -> Option<serde_json::Value> {
        None
    }

    pub fn lsp_config_path(&self, _root: &Path) -> Option<PathBuf> {
        None
    }

    pub fn inline_lsp_servers(&self) -> Option<serde_json::Value> {
        None
    }
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

pub fn load_manifest(_plugin_root: &Path) -> Result<ManifestLoadResult, ManifestError> {
    Ok(ManifestLoadResult::NotFound)
}

pub fn name_from_dirname(dirname: &str) -> String {
    dirname.to_string()
}
