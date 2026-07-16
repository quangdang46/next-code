use std::ffi::{OsStr, OsString};

/// Mutate the process environment for next-code runtime configuration.
///
/// Rust 2024 makes environment mutation unsafe because it can race with
/// concurrent environment access in foreign code. next-code intentionally mutates
/// process-local env vars to coordinate provider/runtime bootstrap before or
/// during task execution. We centralize that unsafety here so call sites remain
/// auditable.
pub fn set_var<K, V>(key: K, value: V)
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    // SAFETY: next-code treats these mutations as process-global configuration.
    // They are a pre-existing design choice used throughout startup, auth,
    // provider bootstrap, tests, and self-dev flows. Centralizing the unsafe
    // operation here makes the Rust 2024 requirement explicit without
    // scattering unsafe blocks across hundreds of call sites.
    unsafe {
        std::env::set_var(key, value);
    }
}

/// Remove a process environment variable used by next-code runtime configuration.
pub fn remove_var<K>(key: K)
where
    K: AsRef<OsStr>,
{
    // SAFETY: see `set_var` above; this is the corresponding centralized
    // removal operation for the same process-global configuration surface.
    unsafe {
        std::env::remove_var(key);
    }
}

/// Read a product-branded environment variable with dual-read.
///
/// Tries `NEXT_CODE_{suffix}` first (canonical), then falls back to the
/// legacy `JCODE_{suffix}` name so existing installs keep working during
/// the rebrand.
pub fn product_env(suffix: &str) -> Result<String, std::env::VarError> {
    let new_key = format!("NEXT_CODE_{suffix}");
    match std::env::var(&new_key) {
        Ok(v) => Ok(v),
        Err(std::env::VarError::NotPresent) => {
            let old_key = format!("JCODE_{suffix}");
            std::env::var(old_key)
        }
        Err(e) => Err(e),
    }
}

/// Like [`product_env`] but returns [`OsString`] (preserves non-UTF-8 values).
pub fn product_env_os(suffix: &str) -> Option<OsString> {
    let new_key = format!("NEXT_CODE_{suffix}");
    match std::env::var_os(&new_key) {
        Some(v) => Some(v),
        None => {
            let old_key = format!("JCODE_{suffix}");
            std::env::var_os(old_key)
        }
    }
}

/// Dual-read for env vars that are not simple `NEXT_CODE_`/`JCODE_` suffixes
/// (for example kill-switches that keep a different prefix).
///
/// Tries `new_key` first, then `old_key`.
pub fn product_var_full(new_key: &str, old_key: &str) -> Result<String, std::env::VarError> {
    match std::env::var(new_key) {
        Ok(v) => Ok(v),
        Err(std::env::VarError::NotPresent) => std::env::var(old_key),
        Err(e) => Err(e),
    }
}

/// Dual-read for non-suffix env vars returning [`OsString`].
pub fn product_var_full_os(new_key: &str, old_key: &str) -> Option<OsString> {
    std::env::var_os(new_key).or_else(|| std::env::var_os(old_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|p| p.into_inner())
    }

    #[test]
    fn product_env_prefers_next_code() {
        let _g = lock_env();
        set_var("NEXT_CODE_HOME", "/new");
        set_var("JCODE_HOME", "/old");
        assert_eq!(product_env("HOME").unwrap(), "/new");
        remove_var("NEXT_CODE_HOME");
        remove_var("JCODE_HOME");
    }

    #[test]
    fn product_env_falls_back_to_jcode() {
        let _g = lock_env();
        remove_var("NEXT_CODE_HOME");
        set_var("JCODE_HOME", "/legacy");
        assert_eq!(product_env("HOME").unwrap(), "/legacy");
        remove_var("JCODE_HOME");
    }

    #[test]
    fn product_var_full_dual_read() {
        let _g = lock_env();
        remove_var("NEXT_CODE_HOOKS_CONFIG");
        set_var("JCODE_HOOKS_CONFIG", "/hooks.toml");
        assert_eq!(
            product_var_full("NEXT_CODE_HOOKS_CONFIG", "JCODE_HOOKS_CONFIG").unwrap(),
            "/hooks.toml"
        );
        remove_var("JCODE_HOOKS_CONFIG");
    }
}
