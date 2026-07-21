//! Config path shim for Face render substrate (PR3).
//!
//! Mirrors upstream `xai-grok-config` path helpers that pager-render calls, but
//! defaults the user home to **next-code** layout:
//!
//! | Upstream (grok-build) | This shim |
//! |-----------------------|-----------|
//! | `$GROK_HOME` or `~/.grok` | `$GROK_HOME` → `$NEXT_CODE_HOME` → `~/.next-code` |
//!
//! Provenance: `xai-org/grok-build` `paths.rs` (`default_grok_home` / `grok_home`
//! / `user_grok_home`) + next-code `next_code_dir()` (`NEXT_CODE_HOME` /
//! `~/.next-code`). PR10: [`load_effective_config_disk_only`] reads real
//! `config.toml`; Face `[ui]` ThemeKind ids persist beside next-code tables
//! (not remapped to `[display].theme` dark/light/auto).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub mod shell;

static GROK_HOME: OnceLock<PathBuf> = OnceLock::new();

/// Parse a boolean environment variable (`1`/`true`/`yes`/`on` vs `0`/`false`/`no`/`off`).
///
/// Returns `None` when unset or unrecognized — same contract as upstream
/// `xai_grok_config::env_bool` / shell `util::config::env_bool`.
pub fn env_bool(name: &str) -> Option<bool> {
    match std::env::var(name).ok()?.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" => Some(true),
        "0" | "false" | "no" | "off" | "disabled" => Some(false),
        _ => None,
    }
}

/// Merged `requirements.toml` overlay. Face stub: no disk merge — always `None`.
///
/// Callers use `.and_then(|req| req.get(...))` on the `Option<toml::Value>`.
pub fn load_merged_requirements() -> Option<toml::Value> {
    None
}

/// Default Face/user directory when no env override is set: `~/.next-code`.
///
/// Uses [`dunce::canonicalize`] like upstream grok-build so Windows paths stay
/// free of `\\?\` verbatim prefixes.
pub fn default_grok_home() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dunce::canonicalize(&home)
        .unwrap_or(home)
        .join(".next-code")
}

/// Resolve override directory without creating it.
///
/// Precedence (PR3 migration defaults):
/// 1. `$GROK_HOME` (compat with upstream docs / existing scripts)
/// 2. `$NEXT_CODE_HOME` (next-code product home)
/// 3. [`default_grok_home`] (`~/.next-code`)
fn resolve_home_override() -> PathBuf {
    if let Ok(v) = std::env::var("GROK_HOME") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(v) = std::env::var("NEXT_CODE_HOME") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    default_grok_home()
}

/// Per-user config directory. Created if needed.
///
/// Cached for process lifetime (same `OnceLock` pattern as upstream).
pub fn grok_home() -> PathBuf {
    GROK_HOME
        .get_or_init(|| {
            let home = resolve_home_override();
            let _ = std::fs::create_dir_all(&home);
            home
        })
        .clone()
}

/// `Some(grok_home())` when a user-global home can resolve; `None` otherwise.
///
/// Unlike [`grok_home`], this never invents a cwd-relative fallback for scanners:
/// needs `$GROK_HOME`, `$NEXT_CODE_HOME`, or a platform home directory.
pub fn user_grok_home() -> Option<PathBuf> {
    let resolvable = std::env::var_os("GROK_HOME").is_some()
        || std::env::var_os("NEXT_CODE_HOME").is_some()
        || dirs::home_dir().is_some();
    resolvable.then(grok_home)
}

/// Path to the user `config.toml` under [`grok_home`].
pub fn user_config_toml_path() -> PathBuf {
    grok_home().join("config.toml")
}

/// Disk-only effective config: parse `~/.next-code/config.toml` (or home
/// override). Missing file → empty table. Malformed file → `io::Error`.
///
/// Face settings live under `[ui]` (ThemeKind display names, compact_mode,
/// …). next-code brain keys (`[provider]`, `[display]`, …) coexist in the
/// same file; Face does **not** map ThemeKind down to origin dark/light.
pub fn load_effective_config_disk_only() -> std::io::Result<toml::Value> {
    load_config_toml_at(&user_config_toml_path())
}

/// Parse a config.toml path into a `toml::Value` table.
pub fn load_config_toml_at(path: &Path) -> std::io::Result<toml::Value> {
    if !path.exists() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    let content = std::fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    content.parse::<toml::Value>().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("parse {}: {e}", path.display()),
        )
    })
}

/// Read `config.toml` as a [`toml_edit::DocumentMut`] for in-place edits.
/// Missing / empty → empty doc. Non-empty unparseable → `None` (do not clobber).
pub fn read_config_document_for_edit(path: &Path) -> Option<toml_edit::DocumentMut> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => String::new(),
    };
    match content.parse() {
        Ok(d) => Some(d),
        Err(e) => {
            if content.is_empty() {
                return Some(toml_edit::DocumentMut::new());
            }
            tracing_warn_unparseable(path, &e);
            None
        }
    }
}

fn tracing_warn_unparseable(path: &Path, err: &impl std::fmt::Display) {
    // Keep this crate light — no tracing dep; stderr is fine for rare warn.
    eprintln!(
        "xai-grok-config: config.toml at {} is not valid TOML ({err}); refusing overwrite",
        path.display()
    );
}

/// Set `[section].key = value` in `config.toml`, preserving siblings.
pub fn set_toml_key(
    section: &str,
    key: &str,
    value: impl Into<toml_edit::Value>,
) -> std::io::Result<()> {
    set_toml_key_at(&user_config_toml_path(), section, key, value)
}

/// Path-injectable core of [`set_toml_key`].
pub fn set_toml_key_at(
    path: &Path,
    section: &str,
    key: &str,
    value: impl Into<toml_edit::Value>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let Some(mut doc) = read_config_document_for_edit(path) else {
        return Ok(());
    };
    doc[section][key] = toml_edit::value(value);
    std::fs::write(path, doc.to_string())
}

/// Set a nested `[section.sub].key` (e.g. `ui.contextual_hints.undo`).
pub fn set_toml_nested_key(
    section: &str,
    sub: &str,
    key: &str,
    value: impl Into<toml_edit::Value>,
) -> std::io::Result<()> {
    let path = user_config_toml_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let Some(mut doc) = read_config_document_for_edit(&path) else {
        return Ok(());
    };
    doc[section][sub][key] = toml_edit::value(value);
    std::fs::write(path, doc.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_home_is_dot_next_code() {
        let path = default_grok_home();
        assert_eq!(
            path.file_name().and_then(|s| s.to_str()),
            Some(".next-code"),
            "got {path:?}"
        );
    }

    #[test]
    fn resolve_prefers_grok_home_over_next_code_home() {
        // Pure helper: does not touch OnceLock / grok_home().
        let prev_g = std::env::var_os("GROK_HOME");
        let prev_n = std::env::var_os("NEXT_CODE_HOME");
        unsafe {
            std::env::set_var("GROK_HOME", "/tmp/grok-override-pr3");
            std::env::set_var("NEXT_CODE_HOME", "/tmp/next-code-override-pr3");
        }
        assert_eq!(
            resolve_home_override(),
            PathBuf::from("/tmp/grok-override-pr3")
        );
        match prev_g {
            Some(v) => unsafe { std::env::set_var("GROK_HOME", v) },
            None => unsafe { std::env::remove_var("GROK_HOME") },
        }
        match prev_n {
            Some(v) => unsafe { std::env::set_var("NEXT_CODE_HOME", v) },
            None => unsafe { std::env::remove_var("NEXT_CODE_HOME") },
        }
    }

    #[test]
    fn resolve_uses_next_code_home_when_grok_unset() {
        let prev_g = std::env::var_os("GROK_HOME");
        let prev_n = std::env::var_os("NEXT_CODE_HOME");
        unsafe {
            std::env::remove_var("GROK_HOME");
            std::env::set_var("NEXT_CODE_HOME", "/tmp/next-code-only-pr3");
        }
        assert_eq!(
            resolve_home_override(),
            PathBuf::from("/tmp/next-code-only-pr3")
        );
        match prev_g {
            Some(v) => unsafe { std::env::set_var("GROK_HOME", v) },
            None => unsafe { std::env::remove_var("GROK_HOME") },
        }
        match prev_n {
            Some(v) => unsafe { std::env::set_var("NEXT_CODE_HOME", v) },
            None => unsafe { std::env::remove_var("NEXT_CODE_HOME") },
        }
    }

    #[test]
    fn set_toml_key_preserves_sibling_tables_and_loads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "[provider]\ndefault_model = \"x\"\n\n[ui]\ncompact_mode = false\n",
        )
        .unwrap();

        set_toml_key_at(&path, "ui", "theme", "Grok Night").unwrap();

        let loaded = load_config_toml_at(&path).unwrap();
        assert_eq!(
            loaded
                .get("ui")
                .and_then(|u| u.get("theme"))
                .and_then(|v| v.as_str()),
            Some("Grok Night")
        );
        assert_eq!(
            loaded
                .get("provider")
                .and_then(|p| p.get("default_model"))
                .and_then(|v| v.as_str()),
            Some("x")
        );
        assert_eq!(
            loaded
                .get("ui")
                .and_then(|u| u.get("compact_mode"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn missing_config_loads_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("absent.toml");
        let v = load_config_toml_at(&path).unwrap();
        assert!(v.as_table().unwrap().is_empty());
    }
}

/// Remote feature flags (subset). Upstream lives in config_types; pager passes
/// this into folder_trust::feature_enabled.
#[derive(Debug, Clone, Default)]
pub struct RemoteSettings {
    pub folder_trust_enabled: Option<bool>,
}

