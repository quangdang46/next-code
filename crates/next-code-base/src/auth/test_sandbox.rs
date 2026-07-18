use std::path::{Path, PathBuf};
use std::sync::MutexGuard;

use crate::provider_catalog::{OpenAiCompatibleProfile, openai_compatible_profiles};

pub struct AuthTestSandbox {
    _lock: MutexGuard<'static, ()>,
    temp: tempfile::TempDir,
    saved_env: Vec<(String, Option<String>)>,
}

impl AuthTestSandbox {
    pub fn new() -> anyhow::Result<Self> {
        let lock = crate::storage::lock_test_env();
        let temp = tempfile::Builder::new()
            .prefix("next-code-auth-lifecycle-")
            .tempdir()?;
        let saved_env = tracked_env_vars()
            .into_iter()
            .map(|key| {
                let value = std::env::var(&key).ok();
                (key, value)
            })
            .collect::<Vec<_>>();

        for (key, _) in &saved_env {
            crate::env::remove_var(key);
        }

        std::fs::create_dir_all(temp.path().join("config").join("next-code"))?;
        std::fs::create_dir_all(temp.path().join("external"))?;
        crate::env::set_var("NEXT_CODE_HOME", temp.path());
        crate::provider_catalog::force_apply_openai_compatible_profile_env(None);
        reset_global_auth_state();

        Ok(Self {
            _lock: lock,
            temp,
            saved_env,
        })
    }

    pub fn root(&self) -> &Path {
        self.temp.path()
    }

    pub fn config_dir(&self) -> PathBuf {
        self.root().join("config").join("next-code")
    }

    pub fn external_dir(&self) -> PathBuf {
        self.root().join("external")
    }

    pub fn env_file_path(&self, file_name: &str) -> PathBuf {
        self.config_dir().join(file_name)
    }

    pub fn write_env_file(
        &self,
        file_name: &str,
        env_key: &str,
        value: &str,
    ) -> anyhow::Result<PathBuf> {
        let path = self.env_file_path(file_name);
        std::fs::create_dir_all(self.config_dir())?;
        std::fs::write(&path, format!("{}={}\n", env_key, value))?;
        next_code_core::fs::set_permissions_owner_only(&path)?;
        reset_global_auth_state();
        Ok(path)
    }

    pub fn write_openai_compatible_api_key(
        &self,
        profile: OpenAiCompatibleProfile,
        value: &str,
    ) -> anyhow::Result<PathBuf> {
        let resolved = crate::provider_catalog::resolve_openai_compatible_profile(profile);
        self.write_env_file(&resolved.env_file, &resolved.api_key_env, value)
    }
}

impl Drop for AuthTestSandbox {
    fn drop(&mut self) {
        for (key, value) in self.saved_env.drain(..) {
            if let Some(value) = value {
                crate::env::set_var(&key, value);
            } else {
                crate::env::remove_var(&key);
            }
        }
        reset_global_auth_state();
    }
}

fn reset_global_auth_state() {
    crate::auth::AuthStatus::invalidate_cache();
    crate::provider::clear_all_provider_unavailability_for_account();
    crate::provider::clear_all_model_unavailability_for_account();
}

fn tracked_env_vars() -> Vec<String> {
    let mut keys = [
        "NEXT_CODE_HOME",
        "XDG_CONFIG_HOME",
        "NEXT_CODE_OPENROUTER_API_BASE",
        "NEXT_CODE_OPENROUTER_API_KEY_NAME",
        "NEXT_CODE_OPENROUTER_ENV_FILE",
        "NEXT_CODE_OPENROUTER_CACHE_NAMESPACE",
        "NEXT_CODE_OPENROUTER_PROVIDER_FEATURES",
        "NEXT_CODE_OPENROUTER_TRANSPORT_STATE",
        "NEXT_CODE_OPENROUTER_ALLOW_NO_AUTH",
        "NEXT_CODE_OPENROUTER_PROVIDER",
        "NEXT_CODE_OPENROUTER_NO_FALLBACK",
        "NEXT_CODE_OPENROUTER_MODEL",
        "NEXT_CODE_OPENROUTER_MODEL_CATALOG",
        "NEXT_CODE_OPENROUTER_STATIC_MODELS",
        "NEXT_CODE_OPENROUTER_AUTH_HEADER",
        "NEXT_CODE_OPENROUTER_AUTH_HEADER_NAME",
        "NEXT_CODE_OPENROUTER_DYNAMIC_BEARER_PROVIDER",
        "NEXT_CODE_OPENAI_COMPAT_API_BASE",
        "NEXT_CODE_OPENAI_COMPAT_API_KEY_NAME",
        "NEXT_CODE_OPENAI_COMPAT_ENV_FILE",
        "NEXT_CODE_OPENAI_COMPAT_SETUP_URL",
        "NEXT_CODE_OPENAI_COMPAT_DEFAULT_MODEL",
        "NEXT_CODE_OPENAI_COMPAT_LOCAL_ENABLED",
        "NEXT_CODE_NAMED_PROVIDER_PROFILE",
        "NEXT_CODE_PROVIDER_PROFILE_ACTIVE",
        "NEXT_CODE_PROVIDER_PROFILE_NAME",
        "NEXT_CODE_RUNTIME_PROVIDER",
        "NEXT_CODE_ACTIVE_PROVIDER",
        "NEXT_CODE_FORCE_PROVIDER",
        "OPENAI_API_KEY",
        "OPENROUTER_API_KEY",
        "ANTHROPIC_API_KEY",
        "AZURE_OPENAI_ENDPOINT",
        "AZURE_OPENAI_MODEL",
        "AZURE_OPENAI_API_KEY",
        "AZURE_OPENAI_USE_ENTRA",
        "GOOGLE_API_KEY",
        "GEMINI_API_KEY",
        "CURSOR_API_KEY",
        "BEDROCK_API_KEY",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect::<std::collections::HashSet<_>>();

    for profile in openai_compatible_profiles() {
        keys.insert(profile.api_key_env.to_string());
    }

    let mut keys = keys.into_iter().collect::<Vec<_>>();
    keys.sort();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_isolates_next_code_home_and_config_dir() {
        let sandbox = AuthTestSandbox::new().expect("sandbox");

        assert_eq!(
            std::env::var("NEXT_CODE_HOME").ok().as_deref(),
            Some(sandbox.root().to_str().unwrap())
        );
        assert_eq!(
            crate::storage::app_config_dir().unwrap(),
            sandbox.config_dir()
        );
        assert!(sandbox.config_dir().starts_with(sandbox.root()));
        assert!(sandbox.external_dir().starts_with(sandbox.root()));
        assert!(sandbox.external_dir().exists());
    }

    #[test]
    fn sandbox_openai_compatible_key_file_is_read_from_temp_config() {
        let sandbox = AuthTestSandbox::new().expect("sandbox");
        sandbox
            .write_openai_compatible_api_key(
                crate::provider_catalog::CEREBRAS_PROFILE,
                "test-cerebras-key",
            )
            .expect("write key");

        assert_eq!(
            crate::provider_catalog::load_api_key_from_env_or_config(
                "CEREBRAS_API_KEY",
                "cerebras.env",
            )
            .as_deref(),
            Some("test-cerebras-key")
        );
        assert!(sandbox.env_file_path("cerebras.env").exists());
    }
}
