//! Façade stub of upstream `xai-grok-shell::config` — top-level config
//! load/plugin-toggle helpers the future pager calls. No real
//! `config.toml` / managed-config disk I/O in this compile-stub layer;
//! `load_*` return defaults and the plugin-list mutators are no-ops.

use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::agent::config::Config;
use crate::util::config::RemoteSettings;

/// Simplified stand-in for upstream's `MemoryConfig` (index/embedding/
/// search/pruning/flush sub-configs collapsed to a single `enabled` flag
/// — see module doc).
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct MemoryConfig {
    pub enabled: bool,
}

/// Upstream merges `config.toml` + managed config + remote settings +
/// env overrides into an effective `Config`; this stub always returns
/// `Config::default()`.
pub fn load_effective_config() -> Config {
    Config::default()
}

/// Upstream parses `config.toml` from disk only (no merge); this stub
/// never reads disk.
pub fn load_from_disk(_path: &std::path::Path) -> Config {
    Config::default()
}

/// Upstream reads the org-managed config overlay; this stub always
/// reports none.
pub fn load_managed_config() -> Option<Config> {
    None
}

/// Upstream merges CCP `RemoteSettings` with local requirements; this
/// stub always returns defaults.
pub fn load_merged_requirements() -> RemoteSettings {
    RemoteSettings::default()
}

/// Upstream walks parent directories for project-level `.grok/config.toml`
/// files; this stub always returns empty (no filesystem walk).
pub fn find_project_configs(_cwd: &std::path::Path) -> Vec<PathBuf> {
    Vec::new()
}

pub fn add_disabled_plugin(_plugin_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn remove_disabled_plugin(_plugin_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn add_enabled_plugin(_plugin_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn remove_enabled_plugin(_plugin_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn add_dismissed_plugin_cta(_plugin_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn add_dismissed_plugin_cta_to_file(
    _path: &std::path::Path,
    _plugin_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn dismissed_plugin_ctas() -> HashSet<String> {
    HashSet::new()
}

pub fn dismissed_plugin_ctas_in_file(_path: &std::path::Path) -> HashSet<String> {
    HashSet::new()
}
