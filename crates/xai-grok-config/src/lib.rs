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
//! `~/.next-code`). Managed-config merge is intentionally **not** ported —
//! [`load_effective_config_disk_only`] stays empty until `[ui]` ↔ next-code
//! display schema is designed.

use std::path::PathBuf;
use std::sync::OnceLock;

static GROK_HOME: OnceLock<PathBuf> = OnceLock::new();

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

/// Disk-only effective config. Shim returns empty TOML table (Grok `[ui]` keys
/// are not bridged to next-code `[display]` yet).
pub fn load_effective_config_disk_only() -> std::io::Result<toml::Value> {
    Ok(toml::Value::Table(toml::map::Map::new()))
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
}
