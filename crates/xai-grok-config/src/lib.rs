//! Minimal config shim for Face render substrate (PR2).
//!
//! Mirrors `xai-grok-config` path helpers that pager-render calls
//! (`default_grok_home`, `grok_home`, `user_grok_home`, disk config load).

use std::path::PathBuf;
use std::sync::OnceLock;

static GROK_HOME: OnceLock<PathBuf> = OnceLock::new();

/// The default user grok directory (`~/.grok`) used when `GROK_HOME` is unset.
///
/// Uses [`dunce::canonicalize`] like upstream so Windows paths stay free of
/// `\\?\` verbatim prefixes.
pub fn default_grok_home() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dunce::canonicalize(&home).unwrap_or(home).join(".grok")
}

/// Per-user config directory: `$GROK_HOME` or `~/.grok`. Created if needed.
pub fn grok_home() -> PathBuf {
    GROK_HOME
        .get_or_init(|| {
            let grok_home = if let Ok(v) = std::env::var("GROK_HOME") {
                if v.is_empty() {
                    default_grok_home()
                } else {
                    PathBuf::from(v)
                }
            } else {
                default_grok_home()
            };
            let _ = std::fs::create_dir_all(&grok_home);
            grok_home
        })
        .clone()
}

/// `Some(grok_home())` when `$GROK_HOME` is set or a home directory resolves;
/// `None` otherwise (never falls back to cwd-relative `.grok` for scanners).
pub fn user_grok_home() -> Option<PathBuf> {
    let resolvable = std::env::var_os("GROK_HOME").is_some() || dirs::home_dir().is_some();
    resolvable.then(grok_home)
}

/// Disk-only effective config. Shim returns empty TOML table.
pub fn load_effective_config_disk_only() -> std::io::Result<toml::Value> {
    Ok(toml::Value::Table(toml::map::Map::new()))
}
