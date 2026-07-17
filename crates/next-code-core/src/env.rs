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

/// Read a product-branded environment variable (`NEXT_CODE_{suffix}`).
pub fn product_env(suffix: &str) -> Result<String, std::env::VarError> {
    std::env::var(format!("NEXT_CODE_{suffix}"))
}

/// Like [`product_env`] but returns [`OsString`] (preserves non-UTF-8 values).
pub fn product_env_os(suffix: &str) -> Option<OsString> {
    std::env::var_os(format!("NEXT_CODE_{suffix}"))
}

/// Read a full env key name (for kill-switches that are not `NEXT_CODE_*` suffixes).
pub fn product_var_full(key: &str) -> Result<String, std::env::VarError> {
    std::env::var(key)
}

/// Like [`product_var_full`] returning [`OsString`].
pub fn product_var_full_os(key: &str) -> Option<OsString> {
    std::env::var_os(key)
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
    fn product_env_reads_next_code() {
        let _g = lock_env();
        set_var("NEXT_CODE_HOME", "/new");
        assert_eq!(product_env("HOME").unwrap(), "/new");
        remove_var("NEXT_CODE_HOME");
    }

    #[test]
    fn product_env_missing_is_err() {
        let _g = lock_env();
        remove_var("NEXT_CODE_HOME");
        assert!(product_env("HOME").is_err());
    }

    #[test]
    fn product_var_full_reads_key() {
        let _g = lock_env();
        remove_var("NEXT_CODE_HOOKS_CONFIG");
        set_var("NEXT_CODE_HOOKS_CONFIG", "/hooks.toml");
        assert_eq!(
            product_var_full("NEXT_CODE_HOOKS_CONFIG").unwrap(),
            "/hooks.toml"
        );
        remove_var("NEXT_CODE_HOOKS_CONFIG");
    }
}
