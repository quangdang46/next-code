//! Face `/connect` list sourced from models.dev (OpenCode `provider_next.all` twin).
//!
//! OpenCode builds Connect from `ModelsDev.get()` → `provider_next.all`, then
//! `PROVIDER_PRIORITY` Popular + rest + synthetic **Other** (custom id).
//! See `.tmp-research-plugins/opencode/.../dialog-provider.tsx`.

use std::sync::LazyLock;

/// OpenCode `PROVIDER_PRIORITY` order (`dialog-provider.tsx`).
pub const POPULAR_MODELS_DEV_IDS: &[&str] = &[
    "opencode",
    "opencode-go",
    "openai",
    "github-copilot",
    "anthropic",
    "google",
];

/// OpenCode `CUSTOM_PROVIDER_OPTION_VALUE`.
pub const CUSTOM_PROVIDER_SENTINEL: &str = "__opencode_custom_provider__";

/// OpenCode `CUSTOM_PROVIDER_ID` regex: `^[a-z0-9][a-z0-9-_]*$`.
pub fn is_valid_custom_provider_id(value: &str) -> bool {
    let id = normalize_custom_provider_id(value);
    let Some(id) = id.as_deref() else {
        return false;
    };
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Strip optional `@ai-sdk/` prefix (OpenCode `normalizeCustomProviderID`).
pub fn normalize_custom_provider_id(value: &str) -> Option<String> {
    let provider_id = value.trim().strip_prefix("@ai-sdk/").unwrap_or(value.trim());
    if provider_id.is_empty() {
        return None;
    }
    Some(provider_id.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelsDevProvider {
    pub id: String,
    pub name: String,
}

static EMBEDDED: LazyLock<Vec<ModelsDevProvider>> = LazyLock::new(parse_embedded);

fn parse_embedded() -> Vec<ModelsDevProvider> {
    let raw = include_str!("../assets/models_dev_providers.json");
    parse_models_dev_providers_json(raw).unwrap_or_default()
}

/// Parse `[{ "id", "name" }, ...]` (embedded snapshot or disk cache).
pub fn parse_models_dev_providers_json(raw: &str) -> Result<Vec<ModelsDevProvider>, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("models.dev providers json: {e}"))?;
    let arr = value
        .as_array()
        .ok_or_else(|| "expected top-level array".to_string())?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "provider missing id".to_string())?;
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(id);
        out.push(ModelsDevProvider {
            id: id.to_string(),
            name: name.to_string(),
        });
    }
    if out.is_empty() {
        return Err("empty models.dev provider list".into());
    }
    Ok(out)
}

/// Snapshot of models.dev provider ids + names (bundled; refreshed offline).
pub fn models_dev_connect_providers() -> &'static [ModelsDevProvider] {
    &EMBEDDED
}

/// Map models.dev / OpenCode provider id → next-code Face auth method id.
///
/// Special OAuth/device/multi-step flows keep next-code login ids; everything
/// else stays the models.dev id for OpenCode-compatible `auth.json` storage.
pub fn face_auth_id_for_models_dev(models_dev_id: &str) -> &str {
    match models_dev_id {
        "anthropic" => "claude",
        "google" => "gemini",
        "github-copilot" => "copilot",
        "amazon-bedrock" => "bedrock",
        // next-code openai-compat profile ids that differ from models.dev.
        "fireworks-ai" => "fireworks",
        "kimi-for-coding" => "kimi",
        other => other,
    }
}

/// Whether this Face auth id should open the multi-method picker (trailing space).
pub fn face_auth_id_needs_method_picker(face_auth_id: &str) -> bool {
    matches!(face_auth_id, "claude" | "openai" | "gemini")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_is_opencode_scale() {
        let list = models_dev_connect_providers();
        assert!(
            list.len() >= 150,
            "expected ~170 models.dev providers, got {}",
            list.len()
        );
        for id in POPULAR_MODELS_DEV_IDS {
            assert!(
                list.iter().any(|p| p.id == *id),
                "missing popular id {id}"
            );
        }
        assert!(list.iter().any(|p| p.id == "cohere"));
        assert!(list.iter().any(|p| p.id == "venice"));
    }

    #[test]
    fn custom_id_validation_matches_opencode() {
        assert!(is_valid_custom_provider_id("my-provider"));
        assert!(is_valid_custom_provider_id("a1"));
        assert!(is_valid_custom_provider_id("@ai-sdk/foo-bar"));
        assert!(!is_valid_custom_provider_id(""));
        assert!(!is_valid_custom_provider_id("Bad_Case"));
        assert!(!is_valid_custom_provider_id("-leading"));
        assert!(!is_valid_custom_provider_id("has space"));
    }

    #[test]
    fn auth_id_mapping_covers_specials() {
        assert_eq!(face_auth_id_for_models_dev("anthropic"), "claude");
        assert_eq!(face_auth_id_for_models_dev("google"), "gemini");
        assert_eq!(face_auth_id_for_models_dev("github-copilot"), "copilot");
        assert_eq!(face_auth_id_for_models_dev("amazon-bedrock"), "bedrock");
        assert_eq!(face_auth_id_for_models_dev("cohere"), "cohere");
        assert!(face_auth_id_needs_method_picker("claude"));
        assert!(!face_auth_id_needs_method_picker("cohere"));
    }
}
