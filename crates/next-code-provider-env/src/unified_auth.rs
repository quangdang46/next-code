//! OpenCode-compatible entries in `~/.next-code/auth.json`.
//!
//! next-code keeps Claude multi-account OAuth in `anthropic_accounts[]` on the
//! same file. Flat map keys (OpenCode provider ids) hold `{ "type": "api", "key" }`
//! (and optionally oauth/wellknown). See `docs/AUTH_CREDENTIAL_SOURCES.md`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::{Map, Value, json};

pub fn auth_json_path() -> Result<PathBuf> {
    Ok(next_code_storage::next_code_dir()?.join("auth.json"))
}

/// OpenCode / next-code provider ids that may hold an API key for `env_key`.
pub fn provider_ids_for_env_key(env_key: &str) -> &'static [&'static str] {
    match env_key {
        "ANTHROPIC_API_KEY" => &["anthropic", "claude"],
        "AZURE_OPENAI_API_KEY" => &["azure-openai-responses", "azure", "azure-openai"],
        "OPENAI_API_KEY" => &["openai", "openai-api"],
        "GEMINI_API_KEY" => &["google", "gemini"],
        "MISTRAL_API_KEY" => &["mistral"],
        "GROQ_API_KEY" => &["groq"],
        "CEREBRAS_API_KEY" => &["cerebras"],
        "XAI_API_KEY" => &["xai"],
        "OPENROUTER_API_KEY" => &["openrouter"],
        "AI_GATEWAY_API_KEY" => &["vercel-ai-gateway"],
        "ZHIPU_API_KEY" | "ZAI_API_KEY" => &["zai"],
        "OPENCODE_API_KEY" => &["opencode"],
        "OPENCODE_GO_API_KEY" => &["opencode-go", "opencode"],
        "HF_TOKEN" => &["huggingface"],
        "KIMI_API_KEY" => &["kimi-coding", "kimi", "moonshot"],
        "MINIMAX_API_KEY" => &["minimax"],
        "MINIMAX_CN_API_KEY" => &["minimax-cn"],
        "NEBIUS_API_KEY" => &["nebius"],
        "SCALEWAY_API_KEY" => &["scaleway"],
        "STACKIT_API_KEY" => &["stackit"],
        "TOGETHER_API_KEY" => &["togetherai", "together-ai", "together"],
        "DEEPINFRA_API_KEY" => &["deepinfra"],
        "FIREWORKS_API_KEY" => &["fireworks"],
        "CHUTES_API_KEY" => &["chutes"],
        "BASETEN_API_KEY" => &["baseten"],
        "CORTECS_API_KEY" => &["cortecs"],
        "COMTEGRA_API_KEY" => &["comtegra", "cgc"],
        "DEEPSEEK_API_KEY" => &["deepseek"],
        "FIRMWARE_API_KEY" => &["firmware"],
        "MOONSHOT_API_KEY" => &["moonshotai", "moonshot"],
        "PERPLEXITY_API_KEY" => &["perplexity"],
        "BAILIAN_CODING_PLAN_API_KEY" => &["alibaba-coding-plan", "bailian"],
        "CURSOR_API_KEY" => &["cursor"],
        "NVIDIA_API_KEY" => &["nvidia-nim", "nvidia"],
        "OLLAMA_API_KEY" => &["ollama"],
        "302AI_API_KEY" => &["302ai"],
        _ => &[],
    }
}

/// Canonical write key (first OpenCode-compatible id for the env var).
pub fn canonical_provider_id_for_env_key(env_key: &str) -> Option<&'static str> {
    provider_ids_for_env_key(env_key).first().copied()
}

fn extract_api_key_from_entry(entry: &Value) -> Option<String> {
    let object = entry.as_object()?;
    if object.get("type")?.as_str()? != "api" {
        return None;
    }
    object
        .get("key")?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

/// Read an API key from the unified `auth.json` map (OpenCode `{type:api,key}`).
pub fn load_api_key_from_unified_auth(env_key: &str) -> Option<String> {
    let path = auth_json_path().ok()?;
    if !path.exists() {
        return None;
    }
    next_code_storage::harden_secret_file_permissions(&path);
    let root: Value = next_code_storage::read_json(&path).ok()?;
    let object = root.as_object()?;
    for provider_id in provider_ids_for_env_key(env_key) {
        if let Some(entry) = object.get(*provider_id)
            && let Some(key) = extract_api_key_from_entry(entry)
        {
            return Some(key);
        }
    }
    None
}

/// True when `auth.json` has a usable `{type:api,key}` for this env var.
pub fn unified_auth_has_api_key(env_key: &str) -> bool {
    load_api_key_from_unified_auth(env_key).is_some()
}

fn load_auth_root() -> Result<Map<String, Value>> {
    let path = auth_json_path()?;
    if !path.exists() {
        return Ok(Map::new());
    }
    next_code_storage::harden_secret_file_permissions(&path);
    let root: Value = next_code_storage::read_json(&path)
        .with_context(|| format!("Could not read unified auth from {:?}", path))?;
    match root {
        Value::Object(map) => Ok(map),
        _ => Ok(Map::new()),
    }
}

fn write_auth_root(map: &Map<String, Value>) -> Result<()> {
    let path = auth_json_path()?;
    let value = Value::Object(map.clone());
    next_code_storage::write_json_secret(&path, &value)
        .with_context(|| format!("Could not write unified auth to {:?}", path))?;
    Ok(())
}

/// Upsert `{ "type": "api", "key": ... }` under an arbitrary provider id
/// (OpenCode `auth.set` twin for models.dev / custom Other).
///
/// Preserves `anthropic_accounts` and other entries. Does **not** overwrite
/// an existing oauth/wellknown entry for the same id.
pub fn save_api_key_for_provider_id(provider_id: &str, key: &str) -> Result<()> {
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        anyhow::bail!("Refusing to write API key for empty provider id");
    }
    if provider_id == "anthropic_accounts" || provider_id == "active_anthropic_account" {
        anyhow::bail!("Refusing to overwrite reserved auth.json key '{provider_id}'");
    }
    let key = key.trim();
    if key.is_empty() {
        anyhow::bail!("Refusing to write empty API key for {provider_id}");
    }

    let mut map = load_auth_root()?;
    if let Some(existing) = map.get(provider_id)
        && let Some(entry_type) = existing.get("type").and_then(Value::as_str)
        && entry_type != "api"
    {
        anyhow::bail!(
            "auth.json provider '{provider_id}' already has type '{entry_type}'; \
             refusing to replace with API key"
        );
    }

    map.insert(
        provider_id.to_string(),
        json!({
            "type": "api",
            "key": key,
        }),
    );
    write_auth_root(&map)?;
    Ok(())
}

/// Upsert `{ "type": "api", "key": ... }` under the canonical provider id.
///
/// Preserves `anthropic_accounts` and other OpenCode provider entries.
/// Does **not** delete legacy `app_config_dir()/*.env` files.
pub fn save_api_key_to_unified_auth(env_key: &str, key: &str) -> Result<&'static str> {
    let provider_id = canonical_provider_id_for_env_key(env_key).ok_or_else(|| {
        anyhow::anyhow!(
            "No unified auth provider id mapping for env key '{env_key}'; cannot write auth.json"
        )
    })?;
    save_api_key_for_provider_id(provider_id, key)?;
    Ok(provider_id)
}

/// Copy a key from legacy `app_config_dir()/<file>` into unified auth.json.
/// Leaves the legacy file intact. Returns true if a key was written.
pub fn migrate_legacy_env_file_into_unified(env_key: &str, file_name: &str) -> Result<bool> {
    if load_api_key_from_unified_auth(env_key).is_some() {
        return Ok(false);
    }
    let Some(key) = crate::load_env_value_from_config_file(env_key, file_name) else {
        return Ok(false);
    };
    if canonical_provider_id_for_env_key(env_key).is_none() {
        return Ok(false);
    }
    save_api_key_to_unified_auth(env_key, &key)?;
    Ok(true)
}

#[derive(Debug, Default, Clone)]
pub struct MigrateReport {
    pub migrated: Vec<String>,
    pub skipped: Vec<String>,
    pub auth_json: PathBuf,
    pub config_dir: PathBuf,
}

/// Known env-file pairs used by login / Face `/connect`.
pub fn known_legacy_api_key_files() -> &'static [(&'static str, &'static str)] {
    &[
        ("ANTHROPIC_API_KEY", "anthropic.env"),
        ("OPENAI_API_KEY", "openai.env"),
        ("OPENROUTER_API_KEY", "openrouter.env"),
        ("GEMINI_API_KEY", "gemini.env"),
        ("CURSOR_API_KEY", "cursor.env"),
        ("ZHIPU_API_KEY", "zai.env"),
        ("OPENCODE_API_KEY", "opencode.env"),
        ("OPENCODE_GO_API_KEY", "opencode-go.env"),
        ("KIMI_API_KEY", "kimi.env"),
        ("DEEPSEEK_API_KEY", "deepseek.env"),
        ("GROQ_API_KEY", "groq.env"),
        ("CEREBRAS_API_KEY", "cerebras.env"),
        ("XAI_API_KEY", "xai.env"),
        ("MISTRAL_API_KEY", "mistral.env"),
        ("BASETEN_API_KEY", "baseten.env"),
        ("CORTECS_API_KEY", "cortecs.env"),
        ("COMTEGRA_API_KEY", "comtegra.env"),
        ("MINIMAX_API_KEY", "minimax.env"),
        ("HF_TOKEN", "huggingface.env"),
        ("AI_GATEWAY_API_KEY", "vercel-ai-gateway.env"),
        ("AZURE_OPENAI_API_KEY", "azure-openai.env"),
        ("NVIDIA_API_KEY", "nvidia-nim.env"),
        ("OLLAMA_API_KEY", "ollama.env"),
        ("302AI_API_KEY", "302ai.env"),
    ]
}

/// Copy all known legacy `*.env` API keys into `auth.json`. Never deletes legacy.
pub fn migrate_all_known_legacy_api_keys() -> Result<MigrateReport> {
    let mut report = MigrateReport {
        auth_json: auth_json_path()?,
        config_dir: next_code_storage::app_config_dir()?,
        ..MigrateReport::default()
    };
    for (env_key, file_name) in known_legacy_api_key_files() {
        match migrate_legacy_env_file_into_unified(env_key, file_name)? {
            true => report.migrated.push(format!("{env_key} ({file_name})")),
            false => {
                if crate::load_env_value_from_config_file(env_key, file_name).is_some()
                    || load_api_key_from_unified_auth(env_key).is_some()
                {
                    report
                        .skipped
                        .push(format!("{env_key} (already in auth.json or unmapped)"));
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::MutexGuard;

    struct EnvGuard {
        _lock: MutexGuard<'static, ()>,
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvGuard {
        fn set_home(path: &std::path::Path) -> Self {
            let lock = TEST_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
            let previous = std::env::var_os("NEXT_CODE_HOME");
            next_code_core::env::set_var("NEXT_CODE_HOME", path);
            Self {
                _lock: lock,
                key: "NEXT_CODE_HOME",
                previous,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                next_code_core::env::set_var(self.key, previous);
            } else {
                next_code_core::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn roundtrip_api_key_preserves_anthropic_accounts() {
        let temp = tempfile::TempDir::new().unwrap();
        let _home = EnvGuard::set_home(temp.path());

        let auth_path = temp.path().join("auth.json");
        std::fs::write(
            &auth_path,
            r#"{
              "anthropic_accounts": [
                {
                  "label": "claude-1",
                  "access": "sk-ant-oat-test",
                  "refresh": "sk-ant-ort-test",
                  "expires": 9999999999999
                }
              ],
              "active_anthropic_account": "claude-1"
            }"#,
        )
        .unwrap();

        let provider = save_api_key_to_unified_auth("ANTHROPIC_API_KEY", "sk-ant-api-test").unwrap();
        assert_eq!(provider, "anthropic");

        let root: Value =
            serde_json::from_str(&std::fs::read_to_string(&auth_path).unwrap()).unwrap();
        assert_eq!(
            root["anthropic_accounts"][0]["access"],
            json!("sk-ant-oat-test")
        );
        assert_eq!(root["anthropic"]["type"], json!("api"));
        assert_eq!(root["anthropic"]["key"], json!("sk-ant-api-test"));
        assert_eq!(
            load_api_key_from_unified_auth("ANTHROPIC_API_KEY").as_deref(),
            Some("sk-ant-api-test")
        );
    }

    #[test]
    fn migrate_copies_legacy_env_without_deleting() {
        let temp = tempfile::TempDir::new().unwrap();
        let _home = EnvGuard::set_home(temp.path());

        let config_dir = next_code_storage::app_config_dir().unwrap();
        std::fs::create_dir_all(&config_dir).unwrap();
        let legacy = config_dir.join("openrouter.env");
        std::fs::write(&legacy, "OPENROUTER_API_KEY=or-test-key\n").unwrap();

        assert!(
            migrate_legacy_env_file_into_unified("OPENROUTER_API_KEY", "openrouter.env").unwrap()
        );
        assert_eq!(
            load_api_key_from_unified_auth("OPENROUTER_API_KEY").as_deref(),
            Some("or-test-key")
        );
        assert!(legacy.exists());
        assert!(std::fs::read_to_string(&legacy)
            .unwrap()
            .contains("OPENROUTER_API_KEY=or-test-key"));
    }

    #[test]
    fn refuses_to_overwrite_oauth_entry_with_api_key() {
        let temp = tempfile::TempDir::new().unwrap();
        let _home = EnvGuard::set_home(temp.path());

        let auth_path = temp.path().join("auth.json");
        std::fs::write(
            &auth_path,
            r#"{
              "openai": {
                "type": "oauth",
                "access": "access",
                "refresh": "refresh",
                "expires": 1
              }
            }"#,
        )
        .unwrap();

        let err = save_api_key_to_unified_auth("OPENAI_API_KEY", "sk-test").unwrap_err();
        assert!(err.to_string().contains("oauth"));
    }

    #[test]
    fn save_api_key_for_arbitrary_provider_id() {
        let temp = tempfile::TempDir::new().unwrap();
        let _home = EnvGuard::set_home(temp.path());

        save_api_key_for_provider_id("cohere", "sk-cohere-test").unwrap();
        let auth_path = temp.path().join("auth.json");
        let root: Value =
            serde_json::from_str(&std::fs::read_to_string(&auth_path).unwrap()).unwrap();
        assert_eq!(root["cohere"]["type"], json!("api"));
        assert_eq!(root["cohere"]["key"], json!("sk-cohere-test"));
    }
}
