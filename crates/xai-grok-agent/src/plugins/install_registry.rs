//! Stub of upstream `xai-grok-agent::plugins::install_registry`. Field/enum
//! shapes match upstream; disk persistence (`save`/`load_from`) is a no-op
//! placeholder — this compile-stub layer does not touch the real plugin
//! install directory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallKind {
    Git,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceProvenance {
    pub source_url_or_path: String,
    pub source_display_name: String,
    pub plugin_subdir: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoPlugin {
    pub name: String,
    #[serde(default)]
    pub subdir: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledRepo {
    pub kind: InstallKind,
    pub installed_at: String,
    pub updated_at: String,
    pub path: PathBuf,
    #[serde(default)]
    pub plugins: HashMap<String, RepoPlugin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketplace: Option<MarketplaceProvenance>,
}

#[derive(Debug, Default)]
pub struct InstallRegistry {
    install_dir: PathBuf,
    repos: HashMap<String, InstalledRepo>,
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("install registry stub: not implemented")]
    NotImplemented,
}

impl InstallRegistry {
    /// Real upstream loads from `resolve_install_dir()` on disk; this stub
    /// always returns an empty registry (no disk I/O in this compile layer).
    pub fn load() -> Self {
        Self::empty(resolve_install_dir())
    }

    pub fn load_from(install_dir: PathBuf) -> Self {
        Self::empty(install_dir)
    }

    pub fn try_load_from(install_dir: PathBuf) -> Result<Self, InstallError> {
        Ok(Self::empty(install_dir))
    }

    pub fn empty(install_dir: PathBuf) -> Self {
        Self {
            install_dir,
            repos: HashMap::new(),
        }
    }

    pub fn save(&self) -> Result<(), InstallError> {
        Ok(())
    }

    pub fn save_atomic(&self) -> Result<(), InstallError> {
        Ok(())
    }

    pub fn get_repo(&self, repo_key: &str) -> Option<&InstalledRepo> {
        self.repos.get(repo_key)
    }

    pub fn get_repo_mut(&mut self, repo_key: &str) -> Option<&mut InstalledRepo> {
        self.repos.get_mut(repo_key)
    }

    pub fn find_plugin(&self, plugin_name: &str) -> Option<(&str, &InstalledRepo)> {
        self.repos
            .iter()
            .find(|(_, repo)| repo.plugins.contains_key(plugin_name))
            .map(|(k, v)| (k.as_str(), v))
    }

    pub fn insert(&mut self, repo_key: String, repo: InstalledRepo) {
        self.repos.insert(repo_key, repo);
    }

    pub fn remove(&mut self, repo_key: &str) -> Option<InstalledRepo> {
        self.repos.remove(repo_key)
    }

    pub fn list(&self) -> Vec<(&str, &InstalledRepo)> {
        self.repos.iter().map(|(k, v)| (k.as_str(), v)).collect()
    }

    pub fn install_dir(&self) -> &Path {
        &self.install_dir
    }
}

/// Upstream resolves this under the Grok home dir; stub returns a
/// placeholder relative path (never actually read/written by this crate).
pub fn resolve_install_dir() -> PathBuf {
    PathBuf::from(".").join("installed-plugins")
}

pub fn repo_key(source: &str) -> String {
    source.to_string()
}
