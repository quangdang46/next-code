//! In-Face multi-provider auth for the next-code embed.
//!
//! Face chrome (welcome paste box + auth URL) drives credential capture.
//! Credentials write into `~/.next-code` via the same login helpers as CLI —
//! no "run `next-code login` in another terminal" handoff.

use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use tokio::sync::oneshot;

use super::login::LoginOptions;
use super::provider_init::save_named_api_key;
use crate::provider_catalog::{
    LoginProviderAuthKind, LoginProviderDescriptor, LoginProviderTarget, resolve_login_provider,
    resolve_login_provider_loose, resolve_openai_compatible_profile, tui_login_providers,
};

const METHOD_PREFIX: &str = "nextcode.";

#[derive(Debug)]
struct PendingFaceLogin {
    provider_id: String,
    auth_url: Option<String>,
    mode: String,
    kind: PendingKind,
    code_tx: Option<oneshot::Sender<String>>,
}

#[derive(Debug)]
enum PendingKind {
    ApiKey {
        env_file: String,
        key_name: String,
        optional: bool,
    },
    ScriptableOAuth,
    /// Device / complete-only (Copilot).
    ScriptableComplete,
}

static PENDING: Mutex<Option<PendingFaceLogin>> = Mutex::new(None);

pub fn method_id_for_provider(provider_id: &str) -> String {
    format!("{METHOD_PREFIX}{provider_id}")
}

pub fn provider_id_from_method(method_id: &str) -> Option<&str> {
    method_id.strip_prefix(METHOD_PREFIX)
}

pub fn is_nextcode_auth_method(method_id: &str) -> bool {
    method_id.starts_with(METHOD_PREFIX)
}

/// Advertise interactive next-code connect method (after `xai.api_key`).
pub fn connect_auth_method() -> agent_client_protocol::AuthMethod {
    use agent_client_protocol as acp;
    let mut meta = serde_json::Map::new();
    meta.insert("external_provider".into(), serde_json::Value::Bool(true));
    acp::AuthMethod::Agent(
        acp::AuthMethodAgent::new(
            acp::AuthMethodId::new(method_id_for_provider("connect")),
            "Connect provider",
        )
        .description("next-code multi-provider auth (use /connect <provider>)")
        .meta(meta),
    )
}

pub fn clear_pending() {
    if let Ok(mut g) = PENDING.lock() {
        *g = None;
    }
}

pub fn get_auth_url_payload() -> serde_json::Value {
    let g = PENDING.lock().ok();
    let pending = g.as_ref().and_then(|p| p.as_ref());
    match pending {
        Some(p) => serde_json::json!({
            "auth_url": p.auth_url,
            "mode": p.mode,
            "external_provider": true,
        }),
        None => serde_json::json!({}),
    }
}

pub async fn authenticate_method(method_id: &str) -> Result<()> {
    if method_id == "xai.api_key" {
        return Ok(());
    }
    let Some(provider_key) = provider_id_from_method(method_id) else {
        anyhow::bail!("Unknown auth method: {method_id}");
    };
    if provider_key == "connect" {
        anyhow::bail!("Pick a provider with /connect <provider> (Face dropdown after /connect ).");
    }

    let provider = resolve_login_provider(provider_key)
        .or_else(|| resolve_login_provider_loose(provider_key))
        .ok_or_else(|| anyhow!("Unknown provider {provider_key:?}"))?;

    match provider.auth_kind {
        LoginProviderAuthKind::ApiKey | LoginProviderAuthKind::Local | LoginProviderAuthKind::Hybrid => {
            run_api_key_face_login(provider).await
        }
        LoginProviderAuthKind::OAuth | LoginProviderAuthKind::DeviceCode => {
            run_oauth_face_login(provider).await
        }
        LoginProviderAuthKind::Cli => {
            anyhow::bail!(
                "{} uses CLI credentials (e.g. az login). Configure outside Face, then restart.",
                provider.display_name
            );
        }
    }
}

pub async fn submit_auth_code(code: &str) -> Result<()> {
    let tx = {
        let mut g = PENDING
            .lock()
            .map_err(|_| anyhow!("auth lock poisoned"))?;
        let pending = g
            .as_mut()
            .ok_or_else(|| anyhow!("No in-progress Face login"))?;
        pending
            .code_tx
            .take()
            .ok_or_else(|| anyhow!("Login is not waiting for a code"))?
    };
    let _ = tx.send(code.trim().to_string());
    Ok(())
}

async fn run_api_key_face_login(provider: LoginProviderDescriptor) -> Result<()> {
    let (env_file, key_name, setup_url, optional) = match provider.target {
        LoginProviderTarget::OpenRouter => (
            "openrouter.env".to_string(),
            "OPENROUTER_API_KEY".to_string(),
            "https://openrouter.ai/keys".to_string(),
            false,
        ),
        LoginProviderTarget::OpenAiApiKey => (
            "openai.env".to_string(),
            "OPENAI_API_KEY".to_string(),
            "https://platform.openai.com/api-keys".to_string(),
            false,
        ),
        LoginProviderTarget::ClaudeApiKey => (
            "anthropic.env".to_string(),
            "ANTHROPIC_API_KEY".to_string(),
            "https://console.anthropic.com/settings/keys".to_string(),
            false,
        ),
        LoginProviderTarget::Cursor => (
            "cursor.env".to_string(),
            "CURSOR_API_KEY".to_string(),
            "https://cursor.com".to_string(),
            false,
        ),
        LoginProviderTarget::OpenAiCompatible(profile) => {
            let resolved = resolve_openai_compatible_profile(profile);
            (
                resolved.env_file,
                resolved.api_key_env,
                resolved.setup_url,
                !resolved.requires_api_key,
            )
        }
        LoginProviderTarget::Gemini => (
            "gemini.env".to_string(),
            "GEMINI_API_KEY".to_string(),
            "https://aistudio.google.com/apikey".to_string(),
            false,
        ),
        other => {
            anyhow::bail!(
                "Face API-key login is not wired for {:?}. Use a catalog API-key provider.",
                other
            );
        }
    };

    let (tx, rx) = oneshot::channel();
    {
        let mut g = PENDING.lock().map_err(|_| anyhow!("auth lock poisoned"))?;
        *g = Some(PendingFaceLogin {
            provider_id: provider.id.to_string(),
            auth_url: Some(setup_url),
            mode: "loopback".into(),
            kind: PendingKind::ApiKey {
                env_file: env_file.clone(),
                key_name: key_name.clone(),
                optional,
            },
            code_tx: Some(tx),
        });
    }

    let code = tokio::time::timeout(Duration::from_secs(15 * 60), rx)
        .await
        .map_err(|_| anyhow!("Timed out waiting for API key paste"))?
        .map_err(|_| anyhow!("Login cancelled"))?;

    clear_pending();

    if code.is_empty() {
        if optional {
            return Ok(());
        }
        anyhow::bail!("No API key provided");
    }

    save_named_api_key(&env_file, &key_name, &code)?;
    crate::auth::AuthStatus::invalidate_cache();
    Ok(())
}

async fn run_oauth_face_login(provider: LoginProviderDescriptor) -> Result<()> {
    // Scriptable OAuth: print-auth-url path writes pending state; we capture URL
    // via a dedicated helper that does not require a TTY paste.
    let start = super::login::face_begin_scriptable(provider)
        .await
        .context("starting OAuth login")?;

    let (tx, rx) = oneshot::channel();
    let kind = if start.complete_only {
        PendingKind::ScriptableComplete
    } else {
        PendingKind::ScriptableOAuth
    };
    {
        let mut g = PENDING.lock().map_err(|_| anyhow!("auth lock poisoned"))?;
        *g = Some(PendingFaceLogin {
            provider_id: provider.id.to_string(),
            auth_url: Some(start.auth_url.clone()),
            mode: if start.complete_only {
                "device".into()
            } else {
                "loopback".into()
            },
            kind,
            code_tx: Some(tx),
        });
    }

    // Best-effort: open browser so user does not need a separate CLI.
    let _ = open::that_detached(&start.auth_url);

    if start.complete_only {
        // Copilot: wait for user to press Enter in Face (any submit) then --complete.
        let _ = tokio::time::timeout(Duration::from_secs(15 * 60), rx)
            .await
            .map_err(|_| anyhow!("Timed out waiting for device login"))?
            .map_err(|_| anyhow!("Login cancelled"))?;
        clear_pending();
        super::login::face_complete_scriptable(
            provider,
            LoginOptions {
                complete: true,
                ..Default::default()
            },
        )
        .await?;
    } else {
        let pasted = tokio::time::timeout(Duration::from_secs(15 * 60), rx)
            .await
            .map_err(|_| anyhow!("Timed out waiting for OAuth callback / code"))?
            .map_err(|_| anyhow!("Login cancelled"))?;
        clear_pending();
        if pasted.is_empty() {
            anyhow::bail!("No auth code / callback URL pasted");
        }
        let looks_like_url = pasted.contains("://") || pasted.contains('?') || pasted.contains('&');
        let mut opts = LoginOptions::default();
        if looks_like_url {
            opts.callback_url = Some(pasted);
        } else {
            opts.auth_code = Some(pasted);
        }
        super::login::face_complete_scriptable(provider, opts).await?;
    }

    crate::auth::AuthStatus::invalidate_cache();
    Ok(())
}

/// Skills list for Face Extensions Skills tab (`x.ai/skills/list`).
pub fn list_nextcode_skills(cwd: Option<&std::path::Path>) -> serde_json::Value {
    let registry = match crate::skill::SkillRegistry::load_for_working_dir(cwd) {
        Ok(r) => r,
        Err(_) => {
            return serde_json::json!({ "result": { "skills": [] } });
        }
    };
    let skills: Vec<serde_json::Value> = registry
        .list()
        .into_iter()
        .map(|skill| {
            let scope = if cwd.is_some_and(|wd| skill.path.starts_with(wd)) {
                "repo"
            } else {
                "user"
            };
            serde_json::json!({
                "name": skill.name,
                "description": skill.description,
                "path": skill.path.display().to_string(),
                "scope": scope,
                "enabled": true,
                "user_invocable": true,
                "disable_model_invocation": false,
                "has_user_specified_description": false,
            })
        })
        .collect();
    serde_json::json!({ "result": { "skills": skills } })
}

/// Session picker payload for Face `/resume` (`x.ai/session/list`).
pub fn list_nextcode_sessions(limit: usize) -> serde_json::Value {
    let Ok(base) = crate::storage::next_code_dir() else {
        return serde_json::json!({ "sessions": [] });
    };
    let dir = base.join("sessions");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return serde_json::json!({ "sessions": [] });
    };

    let mut rows: Vec<(String, String, String, String)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if stem.contains('.') {
            // skip journal sidecars etc.
            continue;
        }
        let Ok(session) = crate::session::Session::load_startup_stub(stem) else {
            continue;
        };
        let summary = session
            .display_title()
            .unwrap_or_else(|| session.display_name())
            .to_string();
        if summary.trim().is_empty() {
            continue;
        }
        let updated = session.updated_at.to_rfc3339();
        let created = session.created_at.to_rfc3339();
        rows.push((stem.to_string(), summary, updated, created));
    }

    rows.sort_by(|a, b| b.2.cmp(&a.2));
    rows.truncate(limit.max(1));

    let sessions: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, summary, updated, created)| {
            serde_json::json!({
                "sessionId": id,
                "summary": summary,
                "updatedAt": updated,
                "createdAt": created,
            })
        })
        .collect();

    serde_json::json!({ "sessions": sessions })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_id_roundtrip() {
        assert_eq!(method_id_for_provider("openrouter"), "nextcode.openrouter");
        assert_eq!(provider_id_from_method("nextcode.openrouter"), Some("openrouter"));
        assert!(is_nextcode_auth_method("nextcode.claude"));
        assert!(!is_nextcode_auth_method("xai.api_key"));
    }

    #[test]
    fn catalog_has_providers() {
        assert!(!tui_login_providers().is_empty());
    }
}
