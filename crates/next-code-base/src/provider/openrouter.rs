//! OpenRouter / OpenAI-compatible provider shared helpers (compatibility shim).
//!
//! The OpenRouter provider *runtime* (`OpenRouterProvider`) now lives in the
//! downstream `next-code-provider-openrouter-runtime` crate so provider edits do
//! not rebuild the base -> app-core -> tui spine. The binary's composition
//! root registers a parameterized factory via
//! [`crate::provider::external::register_openrouter_factory`].
//!
//! Base keeps what its own routing/auth/TUI surfaces share with the runtime:
//! - the env-derived endpoint/key-name/auth-mode configuration helpers,
//! - [`OpenRouterTransportState`] (used by the TUI header and auth lifecycle),
//! - the credential probe (`has_credentials`), and
//! - re-exports of the pure catalog/cache types from
//!   `next-code-provider-openrouter`.

use std::sync::Mutex;

use crate::env::{product_env, product_env_os};
use crate::provider_catalog::{
    OPENAI_COMPAT_PROFILE, ResolvedOpenAiCompatibleProfile, is_safe_env_file_name,
    is_safe_env_key_name, load_api_key_from_env_or_config, normalize_api_base,
    openai_compatible_profile_is_configured, openai_compatible_profiles,
    resolve_openai_compatible_profile, resolve_openai_compatible_profile_selection,
};
pub use next_code_provider_openrouter::{
    EndpointInfo, ModelInfo, ModelPricing, ModelTimestampIndex, ProviderRouting,
    all_model_timestamps, load_endpoints_disk_cache_public, load_model_pricing_disk_cache_public,
    load_model_timestamp_index, model_created_timestamp, model_created_timestamp_from_index,
};

/// Whether the standard OpenRouter public catalog (disk cache) lists a model.
///
/// Returns `None` when no fresh catalog cache exists (unknown), so callers can
/// stay optimistic. Returns `Some(false)` for models OpenRouter definitively
/// does not serve (e.g. `openai/gpt-5.3-codex-spark`, a ChatGPT-subscription
/// exclusive), letting the model picker skip fabricated OpenRouter fallback
/// routes that would 400 with "not a valid model ID" at request time.
pub fn standard_catalog_lists_model(model_id: &str) -> Option<bool> {
    let cache = next_code_provider_openrouter::load_disk_cache_entry_for_namespace("openrouter")?;
    if cache.models.is_empty() {
        return None;
    }
    Some(cache.models.iter().any(|model| model.id == model_id))
}

/// Schedule a background catalog refresh for a direct OpenAI-compatible
/// profile through the composition-root hook (implemented by the runtime
/// crate). Kept at its historical path for callers.
pub(crate) fn maybe_schedule_openai_compatible_profile_catalog_refresh(
    profile: crate::provider_catalog::OpenAiCompatibleProfile,
    context: &'static str,
) -> bool {
    super::external::maybe_schedule_profile_catalog_refresh(profile, context)
}

/// Schedule a background refresh of the standard public OpenRouter catalog
/// through the composition-root hook. Kept at its historical path.
pub(crate) fn maybe_schedule_standard_openrouter_catalog_refresh(context: &'static str) -> bool {
    super::external::maybe_schedule_standard_openrouter_catalog_refresh(context)
}

/// Whether OpenRouter/OpenAI-compatible credentials are available.
pub fn has_credentials() -> bool {
    if matches!(
        configured_dynamic_bearer_provider().as_deref(),
        Some("azure")
    ) {
        return crate::auth::azure::has_configuration();
    }
    if configured_allow_no_auth() {
        return true;
    }
    get_api_key().is_some()
}

/// Resolve the configured API key for the OpenRouter/OpenAI-compatible slot.
pub fn get_api_key() -> Option<String> {
    // Resolve autodetection once so key-name + env-file lookups do not double-scan
    // the 36-profile catalog on cold serve.
    let profile = autodetected_openai_compatible_profile();
    let key_name = api_key_name_from_profile(profile.as_ref());
    let env_file = env_file_from_profile(profile.as_ref());
    load_api_key_from_env_or_config(&key_name, &env_file)
}

/// OpenRouter API base URL
const DEFAULT_API_BASE: &str = "https://openrouter.ai/api/v1";
const DEFAULT_API_KEY_NAME: &str = "OPENROUTER_API_KEY";
const DEFAULT_ENV_FILE: &str = "openrouter.env";

/// Process-local memo for openai-compatible autodetection.
/// Cleared on config reload so tests / runtime pin changes recompute.
static AUTODETECT_CACHE: Mutex<Option<Option<ResolvedOpenAiCompatibleProfile>>> = Mutex::new(None);

fn clear_autodetect_cache() {
    if let Ok(mut guard) = AUTODETECT_CACHE.lock() {
        *guard = None;
    }
}

/// Drop the openai-compatible autodetection memo (config reload / auth invalidate).
pub(crate) fn invalidate_autodetect_cache() {
    clear_autodetect_cache();
}

fn ensure_autodetect_cache_invalidation_hook() {
    use std::sync::Once;
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        crate::config::on_config_reloaded(clear_autodetect_cache);
    });
}

fn explicit_openrouter_runtime_configured() -> bool {
    [
        "OPENROUTER_API_BASE",
        "OPENROUTER_API_KEY_NAME",
        "OPENROUTER_ENV_FILE",
        "OPENROUTER_DYNAMIC_BEARER_PROVIDER",
    ]
    .iter()
    .any(|suffix| crate::env::product_env_os(suffix).is_some())
}

/// When `config.provider.default_provider` names a built-in openai-compatible
/// profile and that profile already has credentials, use it immediately.
/// Returns `None` on pin miss so callers fall through to the full scan.
fn config_pinned_openai_compatible_profile() -> Option<ResolvedOpenAiCompatibleProfile> {
    let pref = crate::config::config()
        .provider
        .default_provider
        .as_deref()?
        .trim();
    if pref.is_empty() {
        return None;
    }

    let profile = resolve_openai_compatible_profile_selection(pref)?;
    if profile.id == OPENAI_COMPAT_PROFILE.id {
        return None;
    }
    if !openai_compatible_profile_is_configured(profile) {
        return None;
    }
    Some(resolve_openai_compatible_profile(profile))
}

fn scan_unique_openai_compatible_profile() -> Option<ResolvedOpenAiCompatibleProfile> {
    let mut matches = openai_compatible_profiles()
        .iter()
        .filter(|profile| profile.id != OPENAI_COMPAT_PROFILE.id)
        .filter_map(|profile| {
            if openai_compatible_profile_is_configured(*profile) {
                Some(resolve_openai_compatible_profile(*profile))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if matches.len() == 1 {
        matches.pop()
    } else {
        None
    }
}

fn autodetected_openai_compatible_profile_uncached() -> Option<ResolvedOpenAiCompatibleProfile> {
    if explicit_openrouter_runtime_configured() {
        return None;
    }

    if load_api_key_from_env_or_config(DEFAULT_API_KEY_NAME, DEFAULT_ENV_FILE).is_some() {
        return None;
    }

    let compat = resolve_openai_compatible_profile(OPENAI_COMPAT_PROFILE);
    if load_api_key_from_env_or_config(&compat.api_key_env, &compat.env_file).is_some() {
        return Some(compat);
    }

    // Config pin first: skip the 36-profile scan when default_provider already
    // resolves (e.g. opencode-go). Pin miss falls through to the unique-match scan.
    if let Some(pinned) = config_pinned_openai_compatible_profile() {
        return Some(pinned);
    }

    scan_unique_openai_compatible_profile()
}

fn autodetected_openai_compatible_profile() -> Option<ResolvedOpenAiCompatibleProfile> {
    ensure_autodetect_cache_invalidation_hook();
    if let Ok(guard) = AUTODETECT_CACHE.lock()
        && let Some(cached) = guard.as_ref()
    {
        return cached.clone();
    }

    let result = autodetected_openai_compatible_profile_uncached();
    if let Ok(mut guard) = AUTODETECT_CACHE.lock() {
        *guard = Some(result.clone());
    }
    result
}

fn api_key_name_from_profile(profile: Option<&ResolvedOpenAiCompatibleProfile>) -> String {
    let raw = product_env("OPENROUTER_API_KEY_NAME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| profile.map(|p| p.api_key_env.clone()))
        .unwrap_or_else(|| DEFAULT_API_KEY_NAME.to_string());
    if is_safe_env_key_name(&raw) {
        raw
    } else {
        crate::logging::warn(&format!(
            "Ignoring invalid NEXT_CODE_OPENROUTER_API_KEY_NAME '{}'; using {}",
            raw, DEFAULT_API_KEY_NAME
        ));
        DEFAULT_API_KEY_NAME.to_string()
    }
}

fn env_file_from_profile(profile: Option<&ResolvedOpenAiCompatibleProfile>) -> String {
    let raw = product_env("OPENROUTER_ENV_FILE")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| profile.map(|p| p.env_file.clone()))
        .unwrap_or_else(|| DEFAULT_ENV_FILE.to_string());
    if is_safe_env_file_name(&raw) {
        raw
    } else {
        crate::logging::warn(&format!(
            "Ignoring invalid NEXT_CODE_OPENROUTER_ENV_FILE '{}'; using {}",
            raw, DEFAULT_ENV_FILE
        ));
        DEFAULT_ENV_FILE.to_string()
    }
}

fn configured_api_base() -> String {
    let raw = product_env("OPENROUTER_API_BASE")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| autodetected_openai_compatible_profile().map(|profile| profile.api_base))
        .unwrap_or_else(|| DEFAULT_API_BASE.to_string());
    normalize_api_base(&raw).unwrap_or_else(|| {
        crate::logging::warn(&format!(
            "Ignoring invalid NEXT_CODE_OPENROUTER_API_BASE '{}'; using {}",
            raw, DEFAULT_API_BASE
        ));
        DEFAULT_API_BASE.to_string()
    })
}

#[cfg(test)]
pub(crate) fn configured_api_key_name_for_test() -> String {
    api_key_name_from_profile(autodetected_openai_compatible_profile().as_ref())
}

fn parse_env_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn provider_features_enabled(api_base: &str) -> bool {
    if let Ok(raw) = product_env("OPENROUTER_PROVIDER_FEATURES") {
        if let Some(value) = parse_env_bool(&raw) {
            return value;
        }
        crate::logging::warn(&format!(
            "Ignoring invalid NEXT_CODE_OPENROUTER_PROVIDER_FEATURES '{}'; expected true/false",
            raw
        ));
    }
    api_base.contains("openrouter.ai")
}

fn configured_dynamic_bearer_provider() -> Option<String> {
    product_env("OPENROUTER_DYNAMIC_BEARER_PROVIDER")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
}

fn configured_allow_no_auth() -> bool {
    product_env("OPENROUTER_ALLOW_NO_AUTH")
        .ok()
        .and_then(|raw| parse_env_bool(&raw))
        .or_else(|| {
            autodetected_openai_compatible_profile().and_then(|profile| {
                if profile.requires_api_key {
                    None
                } else {
                    Some(true)
                }
            })
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenRouterTransportState {
    /// Real OpenRouter BYOK. The provider implementation is both the runtime identity
    /// and the HTTP transport.
    OpenRouterApiKey,
    /// Reserved historical variant for non-BYOK OpenRouter-compatible transports.
    /// A direct OpenAI-compatible endpoint that needs a user key, Azure credential,
    /// or provider-profile secret while reusing the OpenRouter-compatible transport.
    DirectApiKey,
    /// A direct local/no-auth OpenAI-compatible endpoint, for example Ollama or LM Studio.
    DirectNoAuth,
}

impl OpenRouterTransportState {
    pub fn from_current_env(runtime_provider: Option<&str>) -> Self {
        if let Some(state) = Self::from_env_marker() {
            return state;
        }

        let runtime_provider = runtime_provider
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty());

        if matches!(runtime_provider.as_deref(), Some("openrouter")) {
            return Self::OpenRouterApiKey;
        }

        if configured_allow_no_auth() {
            return Self::DirectNoAuth;
        }

        if Self::runtime_provider_is_direct_compatible(runtime_provider.as_deref())
            || product_env_os("NAMED_PROVIDER_PROFILE").is_some()
        {
            return Self::DirectApiKey;
        }

        let api_base = configured_api_base();
        if provider_features_enabled(&api_base) {
            Self::OpenRouterApiKey
        } else {
            Self::DirectApiKey
        }
    }

    fn from_env_marker() -> Option<Self> {
        let raw = crate::env::product_env("OPENROUTER_TRANSPORT_STATE").ok()?;
        let value = raw.trim().to_ascii_lowercase();
        if value.is_empty() {
            return None;
        }

        match value.as_str() {
            "openrouter" | "openrouter-api-key" | "openrouter_byok" | "openrouter-byok" => {
                Some(Self::OpenRouterApiKey)
            }
            "direct" | "direct-api-key" | "openai-compatible" | "compatible-api-key" => {
                Some(Self::DirectApiKey)
            }
            "direct-no-auth" | "no-auth" | "local" => Some(Self::DirectNoAuth),
            other => {
                crate::logging::warn(&format!(
                    "Ignoring invalid NEXT_CODE_OPENROUTER_TRANSPORT_STATE (or legacy NEXT_CODE_*) '{}'; expected openrouter-api-key, direct-api-key, or direct-no-auth",
                    other
                ));
                None
            }
        }
    }

    fn runtime_provider_is_direct_compatible(runtime_provider: Option<&str>) -> bool {
        matches!(runtime_provider, Some("openai-compatible" | "azure-openai"))
            || runtime_provider
                .and_then(crate::provider_catalog::openai_compatible_profile_by_id)
                .is_some()
    }

    pub fn accrues_user_api_key_cost(self) -> bool {
        matches!(self, Self::OpenRouterApiKey | Self::DirectApiKey)
    }

    pub fn is_real_openrouter(self) -> bool {
        matches!(self, Self::OpenRouterApiKey)
    }
}
