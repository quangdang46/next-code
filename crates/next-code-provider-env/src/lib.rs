use std::sync::{LazyLock, RwLock};

use next_code_provider_metadata::{is_safe_env_file_name, is_safe_env_key_name};

pub mod unified_auth;
pub use unified_auth::{
    MigrateReport, auth_json_path, canonical_provider_id_for_env_key, known_legacy_api_key_files,
    load_api_key_from_unified_auth, migrate_all_known_legacy_api_keys,
    migrate_legacy_env_file_into_unified, provider_ids_for_env_key, save_api_key_to_unified_auth,
    unified_auth_has_api_key,
};

/// Fallback resolvers consulted by [`load_api_key_from_env_or_config`] after the
/// environment and config-file lookups fail. Higher-level crates register
/// resolvers at startup so this leaf crate does not need to depend on auth.
type ApiKeyFallbackResolver = fn(&str) -> Option<String>;

static API_KEY_FALLBACK_RESOLVERS: LazyLock<RwLock<Vec<ApiKeyFallbackResolver>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

/// Register a fallback API-key resolver consulted when env/config lookups miss.
pub fn register_api_key_fallback_resolver(resolver: ApiKeyFallbackResolver) {
    API_KEY_FALLBACK_RESOLVERS
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .push(resolver);
}

fn resolve_api_key_fallback(env_key: &str) -> Option<String> {
    let resolvers = API_KEY_FALLBACK_RESOLVERS
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for resolver in resolvers.iter() {
        if let Some(key) = resolver(env_key) {
            return Some(key);
        }
    }
    None
}

/// Characters that editors, terminals, and `cat` render invisibly but that
/// corrupt a credential when embedded in it. Rust's [`str::trim`] only removes
/// ASCII whitespace, so these survive a plain trim and silently break auth
/// (see GitHub issue #376). [`char::is_whitespace`] covers Unicode White_Space
/// (NBSP U+00A0, the en/em spaces U+2002-U+200A, line/paragraph separators,
/// etc.); the explicit cases below are zero-width characters and the BOM, which
/// are not classified as whitespace.
fn is_invisible_boundary_char(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '\u{200B}' // zero-width space
                | '\u{200C}' // zero-width non-joiner
                | '\u{200D}' // zero-width joiner
                | '\u{2060}' // word joiner
                | '\u{FEFF}' // BOM / zero-width no-break space
        )
}

/// Strip leading/trailing invisible (Unicode whitespace and zero-width)
/// characters and one optional layer of surrounding quotes from a loaded
/// secret or config value.
///
/// Exposed so other credential loaders (e.g. the Cursor key reader) can apply
/// the same sanitizing as [`load_api_key_from_env_or_config`].
pub fn sanitize_secret_value(raw: &str) -> &str {
    raw.trim_matches(is_invisible_boundary_char)
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches(is_invisible_boundary_char)
}

/// Sanitize a loaded value and surface a warning when Unicode invisible
/// characters were present, so the failure mode in issue #376 is no longer
/// silent. Returns `None` for values that are empty after sanitizing.
fn clean_loaded_value(raw: &str, env_key: &str) -> Option<String> {
    let cleaned = sanitize_secret_value(raw);
    if cleaned.is_empty() {
        return None;
    }
    // A plain ASCII trim is what we previously did; if it leaves a different
    // result than the Unicode-aware sanitize, hidden characters were stripped.
    let ascii_only = raw.trim().trim_matches('"').trim_matches('\'').trim();
    if ascii_only != cleaned {
        next_code_logging::warn(&format!(
            "Stripped Unicode invisible or non-ASCII whitespace characters from '{}' while loading credentials; verify the value contains no hidden characters",
            env_key
        ));
    }
    Some(cleaned.to_string())
}

pub fn load_api_key_from_env_or_config(env_key: &str, file_name: &str) -> Option<String> {
    if !is_safe_env_key_name(env_key) {
        next_code_logging::warn(&format!(
            "Ignoring invalid API key variable name '{}' while loading credentials",
            env_key
        ));
        return None;
    }
    if !is_safe_env_file_name(file_name) {
        next_code_logging::warn(&format!(
            "Ignoring invalid env file name '{}' while loading credentials",
            file_name
        ));
        return None;
    }

    if let Ok(key) = std::env::var(env_key)
        && let Some(key) = clean_loaded_value(&key, env_key)
    {
        return Some(key);
    }

    // Unified store (`~/.next-code/auth.json`) wins over legacy app-config `*.env`.
    if let Some(key) = load_api_key_from_unified_auth(env_key)
        && let Some(key) = clean_loaded_value(&key, env_key)
    {
        return Some(key);
    }

    let config_path = next_code_storage::app_config_dir().ok()?.join(file_name);
    next_code_storage::harden_secret_file_permissions(&config_path);
    let content = std::fs::read_to_string(&config_path).ok();
    let prefix = format!("{}=", env_key);

    if let Some(content) = content.as_deref() {
        for line in content.lines() {
            if let Some(key) = line.strip_prefix(&prefix)
                && let Some(key) = clean_loaded_value(key, env_key)
            {
                return Some(key);
            }
        }

        if env_key == "ZHIPU_API_KEY" {
            let legacy_prefix = "ZAI_API_KEY=";
            for line in content.lines() {
                if let Some(key) = line.strip_prefix(legacy_prefix)
                    && let Some(key) = clean_loaded_value(key, "ZAI_API_KEY")
                {
                    return Some(key);
                }
            }
        }
    }

    if env_key == "ZHIPU_API_KEY"
        && let Ok(key) = std::env::var("ZAI_API_KEY")
        && let Some(key) = clean_loaded_value(&key, "ZAI_API_KEY")
    {
        return Some(key);
    }

    if let Some(key) = resolve_api_key_fallback(env_key) {
        return Some(key);
    }

    None
}

pub fn load_env_value_from_env_or_config(env_key: &str, file_name: &str) -> Option<String> {
    if !is_safe_env_key_name(env_key) {
        next_code_logging::warn(&format!(
            "Ignoring invalid variable name '{}' while loading config value",
            env_key
        ));
        return None;
    }
    if !is_safe_env_file_name(file_name) {
        next_code_logging::warn(&format!(
            "Ignoring invalid env file name '{}' while loading config value",
            file_name
        ));
        return None;
    }

    if let Ok(value) = std::env::var(env_key)
        && let Some(value) = clean_loaded_value(&value, env_key)
    {
        return Some(value);
    }

    load_env_value_from_config_file(env_key, file_name)
}

/// Load a value only from the saved env file under the next-code config dir,
/// ignoring the process environment.
///
/// [`load_env_value_from_env_or_config`] prefers the process env var, which is
/// correct for ambient configuration but wrong right after an explicit
/// `/login`: a stale env var inherited by a long-lived server process would
/// silently win over the credential the user just saved (issue #453). This
/// reader lets the auth-change path resolve what the file actually contains.
pub fn load_env_value_from_config_file(env_key: &str, file_name: &str) -> Option<String> {
    if !is_safe_env_key_name(env_key) {
        next_code_logging::warn(&format!(
            "Ignoring invalid variable name '{}' while loading config value",
            env_key
        ));
        return None;
    }
    if !is_safe_env_file_name(file_name) {
        next_code_logging::warn(&format!(
            "Ignoring invalid env file name '{}' while loading config value",
            file_name
        ));
        return None;
    }

    let config_path = next_code_storage::app_config_dir().ok()?.join(file_name);
    next_code_storage::harden_secret_file_permissions(&config_path);
    let content = std::fs::read_to_string(config_path).ok()?;
    let prefix = format!("{}=", env_key);

    for line in content.lines() {
        if let Some(value) = line.strip_prefix(&prefix)
            && let Some(value) = clean_loaded_value(value, env_key)
        {
            return Some(value);
        }
    }

    None
}

pub fn save_env_value_to_env_file(
    env_key: &str,
    file_name: &str,
    value: Option<&str>,
) -> anyhow::Result<()> {
    if !is_safe_env_key_name(env_key) {
        anyhow::bail!("Invalid variable name: {}", env_key);
    }
    if !is_safe_env_file_name(file_name) {
        anyhow::bail!("Invalid env file name: {}", file_name);
    }

    let config_dir = next_code_storage::app_config_dir()?;
    let file_path = config_dir.join(file_name);
    next_code_storage::upsert_env_file_value(&file_path, env_key, value)?;

    if let Some(value) = value {
        next_code_core::env::set_var(env_key, value);
    } else {
        next_code_core::env::remove_var(env_key);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::MutexGuard;

    struct EnvGuard {
        _lock: MutexGuard<'static, ()>,
        saved: Vec<(&'static str, Option<OsString>)>,
    }

    impl EnvGuard {
        fn new(keys: &[&'static str]) -> Self {
            let lock = unified_auth::TEST_ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let saved = keys
                .iter()
                .map(|key| (*key, std::env::var_os(key)))
                .collect::<Vec<_>>();
            for key in keys {
                next_code_core::env::remove_var(key);
            }
            Self { _lock: lock, saved }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.saved.drain(..) {
                match value {
                    Some(value) => next_code_core::env::set_var(key, value),
                    None => next_code_core::env::remove_var(key),
                }
            }
        }
    }

    #[test]
    fn loads_api_key_from_env_before_config_file() {
        let temp = tempfile::tempdir().expect("tempdir");
        let _guard = EnvGuard::new(&["NEXT_CODE_HOME", "NEXT_CODE_PROVIDER_ENV_TEST_KEY"]);
        next_code_core::env::set_var("NEXT_CODE_HOME", temp.path());

        save_env_value_to_env_file(
            "NEXT_CODE_PROVIDER_ENV_TEST_KEY",
            "provider-env-test.env",
            Some("file-key"),
        )
        .expect("save file key");
        next_code_core::env::set_var("NEXT_CODE_PROVIDER_ENV_TEST_KEY", "env-key");

        assert_eq!(
            load_api_key_from_env_or_config("NEXT_CODE_PROVIDER_ENV_TEST_KEY", "provider-env-test.env")
                .as_deref(),
            Some("env-key")
        );
    }

    #[test]
    fn loads_and_removes_values_from_sandboxed_config_file() {
        let temp = tempfile::tempdir().expect("tempdir");
        let _guard = EnvGuard::new(&["NEXT_CODE_HOME", "NEXT_CODE_PROVIDER_ENV_TEST_VALUE"]);
        next_code_core::env::set_var("NEXT_CODE_HOME", temp.path());

        save_env_value_to_env_file(
            "NEXT_CODE_PROVIDER_ENV_TEST_VALUE",
            "provider-env-test.env",
            Some("file-value"),
        )
        .expect("save file value");

        next_code_core::env::remove_var("NEXT_CODE_PROVIDER_ENV_TEST_VALUE");
        assert_eq!(
            load_env_value_from_env_or_config(
                "NEXT_CODE_PROVIDER_ENV_TEST_VALUE",
                "provider-env-test.env"
            )
            .as_deref(),
            Some("file-value")
        );

        save_env_value_to_env_file(
            "NEXT_CODE_PROVIDER_ENV_TEST_VALUE",
            "provider-env-test.env",
            None,
        )
        .expect("remove file value");
        assert_eq!(
            load_env_value_from_env_or_config(
                "NEXT_CODE_PROVIDER_ENV_TEST_VALUE",
                "provider-env-test.env"
            ),
            None
        );
    }

    #[test]
    fn accepts_legacy_zai_key_for_zhipu() {
        let temp = tempfile::tempdir().expect("tempdir");
        let _guard = EnvGuard::new(&["NEXT_CODE_HOME", "ZHIPU_API_KEY", "ZAI_API_KEY"]);
        next_code_core::env::set_var("NEXT_CODE_HOME", temp.path());

        save_env_value_to_env_file("ZAI_API_KEY", "zai.env", Some("legacy-zai-key"))
            .expect("save legacy key");
        next_code_core::env::remove_var("ZAI_API_KEY");

        assert_eq!(
            load_api_key_from_env_or_config("ZHIPU_API_KEY", "zai.env").as_deref(),
            Some("legacy-zai-key")
        );
    }

    #[test]
    fn sanitize_strips_unicode_invisible_characters() {
        // Zero-width space, BOM, NBSP, en space around the value.
        assert_eq!(
            sanitize_secret_value("\u{200B}sk-key123\u{FEFF}"),
            "sk-key123"
        );
        assert_eq!(sanitize_secret_value("\u{00A0}sk-key\u{2002}"), "sk-key");
        // Quotes plus invisible padding both stripped.
        assert_eq!(
            sanitize_secret_value("\u{FEFF}\"sk-quoted\"\u{200B}"),
            "sk-quoted"
        );
        // Interior characters are preserved.
        assert_eq!(
            sanitize_secret_value("sk-mid\u{200B}dle"),
            "sk-mid\u{200B}dle"
        );
        // Empty after sanitize.
        assert_eq!(sanitize_secret_value("\u{200B}\u{FEFF}"), "");
    }

    #[test]
    fn loads_api_key_with_zero_width_space_from_config_file() {
        let temp = tempfile::tempdir().expect("tempdir");
        let _guard = EnvGuard::new(&["NEXT_CODE_HOME", "NEXT_CODE_PROVIDER_FOO_API_KEY"]);
        next_code_core::env::set_var("NEXT_CODE_HOME", temp.path());

        // Write an env file with a U+200B zero-width space prefixed onto the key,
        // mirroring issue #376's reproduction.
        let config_dir = next_code_storage::app_config_dir().expect("config dir");
        std::fs::create_dir_all(&config_dir).expect("create config dir");
        std::fs::write(
            config_dir.join("provider-foo.env"),
            "NEXT_CODE_PROVIDER_FOO_API_KEY=\u{200B}sk-mykey123\n",
        )
        .expect("write env file");

        assert_eq!(
            load_api_key_from_env_or_config("NEXT_CODE_PROVIDER_FOO_API_KEY", "provider-foo.env")
                .as_deref(),
            Some("sk-mykey123")
        );
    }

    #[test]
    fn loads_api_key_with_invisible_chars_from_env_var() {
        let temp = tempfile::tempdir().expect("tempdir");
        let _guard = EnvGuard::new(&["NEXT_CODE_HOME", "NEXT_CODE_PROVIDER_BAR_API_KEY"]);
        next_code_core::env::set_var("NEXT_CODE_HOME", temp.path());
        // NBSP + BOM padding around the env-provided key.
        next_code_core::env::set_var("NEXT_CODE_PROVIDER_BAR_API_KEY", "\u{00A0}sk-env-key\u{FEFF}");

        assert_eq!(
            load_api_key_from_env_or_config("NEXT_CODE_PROVIDER_BAR_API_KEY", "provider-bar.env")
                .as_deref(),
            Some("sk-env-key")
        );
    }
}
