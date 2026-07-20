//! Façade stub of upstream `xai-grok-shell::config` — disk/load helpers
//! return empty TOML/`Result` shapes the pager expects.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::util::config::RemoteSettings;

/// Simplified stand-in for upstream's `MemoryConfig`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct MemoryConfig {
    pub enabled: bool,
}

/// Effective merged config as a TOML value (empty table stub).
pub fn load_effective_config() -> anyhow::Result<toml::Value> {
    Ok(toml::Value::Table(toml::map::Map::new()))
}

/// Disk-only `config.toml` parse (no merge).
pub fn load_from_disk() -> anyhow::Result<toml::Value> {
    Ok(toml::Value::Table(toml::map::Map::new()))
}

/// Org-managed config overlay.
pub fn load_managed_config() -> anyhow::Result<toml::Value> {
    Ok(toml::Value::Table(toml::map::Map::new()))
}

/// Merged requirements.toml layers (MDM/system/user).
pub fn load_merged_requirements() -> Option<toml::Value> {
    None
}

/// Walk parents for project-level `.grok/config.toml` files.
pub fn find_project_configs(_cwd: &Path) -> Vec<PathBuf> {
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
    _path: &Path,
    _plugin_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn dismissed_plugin_ctas() -> HashSet<String> {
    HashSet::new()
}

pub fn dismissed_plugin_ctas_in_file(_path: &Path) -> HashSet<String> {
    HashSet::new()
}

/// `[features] leader_mode` / `[leader] enabled` TOML opt-in (stub: unset).
pub fn use_leader_from_toml_opt(_raw: &toml::Value) -> Option<bool> {
    None
}

/// Placeholder retained for call sites that still pass RemoteSettings.
pub fn remote_settings_stub() -> RemoteSettings {
    RemoteSettings::default()
}
