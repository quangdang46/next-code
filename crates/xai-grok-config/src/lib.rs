pub mod config;
pub mod appearance_config;

use std::path::PathBuf;

/// Return the grok home directory: `$GROK_HOME` or `~/.grok`.
pub fn grok_home() -> PathBuf {
    std::env::var("GROK_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".grok")))
        .unwrap_or_else(|| PathBuf::from(".grok"))
}

/// The default grok home path (`~/.grok`).
pub fn default_grok_home() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".grok"))
        .unwrap_or_else(|| PathBuf::from(".grok"))
}

/// The user's grok home, if resolvable.
pub fn user_grok_home() -> Option<PathBuf> {
    std::env::var("GROK_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".grok")))
}

/// Load the effective layered config from disk (managed.toml → config.toml).
/// Stub: returns an empty table.
pub fn load_effective_config_disk_only() -> Result<appearance_config::RawAppearanceConfig, Box<dyn std::error::Error>> {
    let mut table = toml::Table::new();
    let ui = toml::Table::new();
    table.insert("ui".into(), toml::Value::Table(ui));
    Ok(toml::Value::Table(table))
}
