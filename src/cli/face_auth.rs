//! In-Face multi-provider auth for the next-code embed.
//!
//! Face chrome (welcome paste box + auth URL) drives credential capture.
//! Credentials write into `~/.next-code` via the same login helpers as CLI —
//! no "run `next-code login` in another terminal" handoff.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use tokio::sync::oneshot;

use super::login::LoginOptions;
use super::provider_init::save_named_api_key;
use crate::provider_catalog::{
    CUSTOM_PROVIDER_SENTINEL, LoginProviderAuthKind, LoginProviderDescriptor, LoginProviderTarget,
    is_valid_custom_provider_id, normalize_custom_provider_id, resolve_login_provider,
    resolve_login_provider_loose, resolve_openai_compatible_profile,
};

const METHOD_PREFIX: &str = "nextcode.";

/// Bumped when Face ext_method wire shapes for skills/MCP change.
/// Embedded in list payloads (ignored by Face serde) and printable via
/// `NEXT_CODE_FACE_WIRE_REV` in `--version` only if we surface it — for now
/// operators can grep the binary / list payload for this token.
pub const FACE_EXT_WIRE_REV: &str = "20260722f-http-mcp";

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
    /// Waiting for Face paste (API key / Azure multi-step field).
    ApiKeyPaste,
    ScriptableOAuth,
    /// Device / complete-only (Copilot).
    ScriptableComplete,
}

static PENDING: Mutex<Option<PendingFaceLogin>> = Mutex::new(None);
/// Paste/Enter that arrived before `face_prompt_paste` armed `code_tx`.
/// Consumed when the waiter starts so premature submit is not lost and does
/// not surface as an Internal error.
static EARLY_AUTH_CODE: Mutex<Option<String>> = Mutex::new(None);
/// Last successful Face `/connect` credential path (one-line toast / status).
static LAST_CONNECT_CREDENTIAL_PATH: Mutex<Option<String>> = Mutex::new(None);

pub fn method_id_for_provider(provider_id: &str) -> String {
    format!("{METHOD_PREFIX}{provider_id}")
}

pub fn provider_id_from_method(method_id: &str) -> Option<&str> {
    method_id.strip_prefix(METHOD_PREFIX)
}

pub fn is_nextcode_auth_method(method_id: &str) -> bool {
    method_id.starts_with(METHOD_PREFIX)
}

pub fn last_connect_credential_path() -> Option<String> {
    LAST_CONNECT_CREDENTIAL_PATH
        .lock()
        .ok()
        .and_then(|g| g.clone())
}

fn remember_connect_credential_path(path: impl Into<String>) {
    if let Ok(mut g) = LAST_CONNECT_CREDENTIAL_PATH.lock() {
        *g = Some(path.into());
    }
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
    if let Ok(mut g) = EARLY_AUTH_CODE.lock() {
        *g = None;
    }
}

fn take_early_auth_code() -> Option<String> {
    EARLY_AUTH_CODE.lock().ok().and_then(|mut g| g.take())
}

fn store_early_auth_code(code: String) {
    if let Ok(mut g) = EARLY_AUTH_CODE.lock() {
        *g = Some(code);
    }
}

pub fn get_auth_url_payload() -> serde_json::Value {
    let g = PENDING.lock().ok();
    let pending = g.as_ref().and_then(|p| p.as_ref());
    let credential_path = LAST_CONNECT_CREDENTIAL_PATH
        .lock()
        .ok()
        .and_then(|guard| guard.clone());
    match pending {
        Some(p) => {
            let prompt = match p.kind {
                PendingKind::ApiKeyPaste => "api_key",
                PendingKind::ScriptableOAuth => "auth_code",
                PendingKind::ScriptableComplete => "device",
            };
            serde_json::json!({
                "auth_url": p.auth_url,
                "mode": p.mode,
                "prompt": prompt,
                "external_provider": true,
                "credential_path": credential_path,
            })
        }
        None => serde_json::json!({
            "credential_path": credential_path,
        }),
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
        anyhow::bail!("Pick a provider with /connect (opens the Face picker).");
    }

    // OpenCode Other → custom provider id + API key.
    if provider_key == CUSTOM_PROVIDER_SENTINEL {
        return run_custom_provider_face_login().await;
    }

    if let Some(provider) = resolve_login_provider(provider_key)
        .or_else(|| resolve_login_provider_loose(provider_key))
    {
        // Multi-step / non-paste targets before the generic API-key/OAuth branches.
        match provider.target {
            LoginProviderTarget::AutoImport => return run_auto_import_face_login().await,
            LoginProviderTarget::Azure => return run_azure_face_login(provider).await,
            LoginProviderTarget::Bedrock => return run_bedrock_face_login(provider).await,
            _ => {}
        }

        return match provider.auth_kind {
            LoginProviderAuthKind::ApiKey
            | LoginProviderAuthKind::Local
            | LoginProviderAuthKind::Hybrid => run_api_key_face_login(provider).await,
            LoginProviderAuthKind::OAuth | LoginProviderAuthKind::DeviceCode => {
                run_oauth_face_login(provider).await
            }
            LoginProviderAuthKind::Cli => {
                anyhow::bail!(
                    "{} uses CLI credentials (e.g. az login). Configure outside Face, then restart.",
                    provider.display_name
                );
            }
        };
    }

    // models.dev long-tail / free-text custom id: store API key under provider id
    // (OpenCode auth.set twin). Requires a valid OpenCode-style provider id.
    if !is_valid_custom_provider_id(provider_key) {
        anyhow::bail!("Unknown provider {provider_key:?}");
    }
    run_models_dev_api_key_face_login(provider_key).await
}

pub async fn submit_auth_code(code: &str) -> Result<()> {
    let code = code.trim().to_string();
    if code.is_empty() {
        // Empty submit is a no-op (Face already gates empty Enter).
        return Ok(());
    }

    let tx = {
        let mut g = PENDING
            .lock()
            .map_err(|_| anyhow!("auth lock poisoned"))?;
        match g.as_mut() {
            None => {
                // Premature submit (Enter/paste-trailing-newline before the
                // authenticate RPC armed the waiter). Buffer for face_prompt_paste
                // instead of Internal error → AuthFailed toast.
                drop(g);
                store_early_auth_code(code);
                return Ok(());
            }
            Some(pending) => match pending.code_tx.take() {
                Some(tx) => tx,
                None => {
                    // Duplicate submit after the oneshot was already consumed
                    // (Enter repeat / second SubmitAuthCode). Ignore — do not
                    // fail the in-flight authenticate that already got the code.
                    return Ok(());
                }
            },
        }
    };
    let _ = tx.send(code);
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
        other => {
            anyhow::bail!(
                "{} ({other:?}) cannot use Face API-key login.",
                provider.display_name
            );
        }
    };

    let code = face_prompt_paste(provider.id, &setup_url).await?;

    if code.is_empty() {
        clear_pending();
        if optional {
            return Ok(());
        }
        anyhow::bail!("No API key provided");
    }

    clear_pending();
    save_named_api_key(&env_file, &key_name, &code)?;
    crate::auth::AuthStatus::invalidate_cache();
    if let Ok(path) = crate::provider_catalog::auth_json_path() {
        remember_connect_credential_path(path.display().to_string());
        crate::logging::info(&format!(
            "Face /connect saved API key; credential path: {}",
            path.display()
        ));
    }
    Ok(())
}

/// Bedrock: same as legacy TUI — API key paste + default region `us-east-2`.
async fn run_bedrock_face_login(provider: LoginProviderDescriptor) -> Result<()> {
    let setup_url = "https://console.aws.amazon.com/bedrock/home#/api-keys";
    let code = face_prompt_paste(provider.id, setup_url).await?;
    clear_pending();
    if code.is_empty() {
        anyhow::bail!("No API key provided");
    }

    save_named_api_key(
        crate::provider::bedrock::ENV_FILE,
        crate::provider::bedrock::API_KEY_ENV,
        &code,
    )?;
    crate::provider_catalog::save_env_value_to_env_file(
        crate::provider::bedrock::REGION_ENV,
        crate::provider::bedrock::ENV_FILE,
        Some("us-east-2"),
    )?;
    crate::auth::AuthStatus::invalidate_cache();
    if let Ok(path) = crate::storage::app_config_dir() {
        let stored = path.join(crate::provider::bedrock::ENV_FILE);
        remember_connect_credential_path(stored.display().to_string());
    }
    Ok(())
}

/// Azure OpenAI: Face multi-step paste (endpoint → deployment → API key).
/// Entra ID remains CLI (`nextcode login azure`); Face uses API-key path only.
async fn run_azure_face_login(provider: LoginProviderDescriptor) -> Result<()> {
    use crate::auth::azure;

    let docs = "https://portal.azure.com/#create/Microsoft.CognitiveServicesOpenAI";
    let endpoint_raw = face_prompt_paste(provider.id, docs).await?;
    let endpoint = match azure::normalize_endpoint(&endpoint_raw) {
        Some(endpoint) => endpoint,
        None => {
            clear_pending();
            anyhow::bail!(
                "Invalid Azure OpenAI endpoint. Use https://<resource>.openai.azure.com (or the full /openai/v1 URL)."
            );
        }
    };

    let model = face_prompt_paste(provider.id, docs).await?;
    if model.is_empty() {
        clear_pending();
        anyhow::bail!("No deployment/model name provided");
    }

    let key = face_prompt_paste(provider.id, docs).await?;
    clear_pending();
    if key.is_empty() {
        anyhow::bail!("No API key provided");
    }

    let assignments = [
        (azure::ENDPOINT_ENV, endpoint),
        (azure::MODEL_ENV, model),
        (azure::USE_ENTRA_ENV, "0".to_string()),
        (azure::API_KEY_ENV, key),
    ];
    save_face_env_vars(azure::ENV_FILE, &assignments)?;
    azure::apply_runtime_env()?;
    crate::auth::AuthStatus::invalidate_cache();
    if let Ok(path) = crate::storage::app_config_dir() {
        remember_connect_credential_path(path.join(azure::ENV_FILE).display().to_string());
    }
    Ok(())
}

async fn run_auto_import_face_login() -> Result<()> {
    let imported = super::provider_init::maybe_run_external_auth_auto_import_flow()
        .await?
        .unwrap_or(0);
    if imported == 0 {
        anyhow::bail!(
            "No existing logins were imported. Either none were found, nothing was approved, or validation failed."
        );
    }
    crate::auth::AuthStatus::invalidate_cache();
    Ok(())
}

/// OpenCode Other: prompt custom provider id, then API key → auth.json.
async fn run_custom_provider_face_login() -> Result<()> {
    let docs = "https://opencode.ai/docs/providers";
    let raw_id = face_prompt_paste(CUSTOM_PROVIDER_SENTINEL, docs).await?;
    let Some(provider_id) = normalize_custom_provider_id(&raw_id)
        .filter(|id| is_valid_custom_provider_id(id))
    else {
        clear_pending();
        anyhow::bail!(
            "Provider ids must start with a lowercase letter or number and only use lowercase letters, numbers, hyphens, and underscores"
        );
    };
    run_models_dev_api_key_face_login(&provider_id).await
}

/// Store API key under models.dev / custom provider id in `~/.next-code/auth.json`.
async fn run_models_dev_api_key_face_login(provider_id: &str) -> Result<()> {
    let setup_url = format!("https://models.dev/{provider_id}");
    let code = face_prompt_paste(provider_id, &setup_url).await?;
    clear_pending();
    if code.is_empty() {
        anyhow::bail!("No API key provided");
    }
    crate::provider_catalog::save_api_key_for_provider_id(provider_id, &code)?;
    crate::auth::AuthStatus::invalidate_cache();
    if let Ok(path) = crate::provider_catalog::auth_json_path() {
        remember_connect_credential_path(path.display().to_string());
        crate::logging::info(&format!(
            "Face /connect saved API key for {provider_id}; credential path: {}",
            path.display()
        ));
    }
    Ok(())
}

/// Show Face welcome paste chrome and wait for submit_auth_code.
async fn face_prompt_paste(provider_id: &str, auth_url: &str) -> Result<String> {
    // Honor a submit that raced ahead of this waiter (Ctrl+V + trailing
    // newline → Enter before PENDING was armed).
    if let Some(early) = take_early_auth_code() {
        {
            let mut g = PENDING.lock().map_err(|_| anyhow!("auth lock poisoned"))?;
            *g = Some(PendingFaceLogin {
                provider_id: provider_id.to_string(),
                auth_url: Some(auth_url.to_string()),
                mode: "loopback".into(),
                kind: PendingKind::ApiKeyPaste,
                code_tx: None,
            });
        }
        return Ok(early);
    }

    let (tx, rx) = oneshot::channel();
    {
        let mut g = PENDING.lock().map_err(|_| anyhow!("auth lock poisoned"))?;
        *g = Some(PendingFaceLogin {
            provider_id: provider_id.to_string(),
            auth_url: Some(auth_url.to_string()),
            mode: "loopback".into(),
            kind: PendingKind::ApiKeyPaste,
            code_tx: Some(tx),
        });
    }

    // Re-check after arming: a submit may have buffered between the early
    // take and the lock above.
    if let Some(early) = take_early_auth_code() {
        let mut g = PENDING.lock().map_err(|_| anyhow!("auth lock poisoned"))?;
        if let Some(pending) = g.as_mut() {
            pending.code_tx = None;
        }
        return Ok(early);
    }

    match tokio::time::timeout(Duration::from_secs(15 * 60), rx).await {
        Ok(Ok(code)) => Ok(code),
        Ok(Err(_)) => {
            clear_pending();
            Err(anyhow!("Login cancelled"))
        }
        Err(_) => {
            clear_pending();
            Err(anyhow!("Timed out waiting for paste"))
        }
    }
}

fn save_face_env_vars(env_file: &str, vars: &[(&str, String)]) -> Result<()> {
    if !crate::provider_catalog::is_safe_env_file_name(env_file) {
        anyhow::bail!("Invalid env file name: {env_file}");
    }
    for (key, _) in vars {
        if !crate::provider_catalog::is_safe_env_key_name(key) {
            anyhow::bail!("Invalid env key name: {key}");
        }
    }
    let config_dir = crate::storage::app_config_dir()?;
    std::fs::create_dir_all(&config_dir)?;
    crate::platform::set_directory_permissions_owner_only(&config_dir)?;
    let file_path = config_dir.join(env_file);
    let mut content = String::new();
    for (key, value) in vars {
        content.push_str(&format!("{key}={value}\n"));
    }
    std::fs::write(&file_path, &content)?;
    crate::platform::set_permissions_owner_only(&file_path)?;
    for (key, value) in vars {
        crate::env::set_var(key, value);
    }
    Ok(())
}

async fn run_oauth_face_login(provider: LoginProviderDescriptor) -> Result<()> {
    // Scriptable OAuth: print-auth-url path writes pending state; we capture URL
    // via a dedicated helper that does not require a TTY paste.
    let start = super::login::face_begin_scriptable(provider)
        .await
        .context("starting OAuth login")?;

    let kind = if start.complete_only {
        PendingKind::ScriptableComplete
    } else {
        PendingKind::ScriptableOAuth
    };
    let mode = if start.complete_only {
        "device".to_string()
    } else {
        "loopback".to_string()
    };

    // Honor submit that raced ahead while begin_scriptable was in flight.
    let early = take_early_auth_code();
    let (code_tx, rx) = if early.is_some() {
        (None, None)
    } else {
        let (tx, rx) = oneshot::channel();
        (Some(tx), Some(rx))
    };
    {
        let mut g = PENDING.lock().map_err(|_| anyhow!("auth lock poisoned"))?;
        *g = Some(PendingFaceLogin {
            provider_id: provider.id.to_string(),
            auth_url: Some(start.auth_url.clone()),
            mode,
            kind,
            code_tx,
        });
    }

    // Best-effort: open browser so user does not need a separate CLI.
    let _ = open::that_detached(&start.auth_url);

    if start.complete_only {
        if let Some(rx) = rx {
            match tokio::time::timeout(Duration::from_secs(15 * 60), rx).await {
                Ok(Ok(_)) => {}
                Ok(Err(_)) => {
                    clear_pending();
                    anyhow::bail!("Login cancelled");
                }
                Err(_) => {
                    clear_pending();
                    anyhow::bail!("Timed out waiting for device login");
                }
            }
        }
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
        let pasted = if let Some(code) = early.or_else(take_early_auth_code) {
            let mut g = PENDING.lock().map_err(|_| anyhow!("auth lock poisoned"))?;
            if let Some(pending) = g.as_mut() {
                pending.code_tx = None;
            }
            code
        } else {
            let rx = rx.expect("oauth waiter armed without early code");
            match tokio::time::timeout(Duration::from_secs(15 * 60), rx).await {
                Ok(Ok(code)) => code,
                Ok(Err(_)) => {
                    clear_pending();
                    anyhow::bail!("Login cancelled");
                }
                Err(_) => {
                    clear_pending();
                    anyhow::bail!("Timed out waiting for OAuth callback / code");
                }
            }
        };
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

/// Resolve Face `cwd` params. Face often sends `"."`; treat that as the process cwd.
pub fn resolve_face_cwd(cwd: Option<&Path>) -> Option<PathBuf> {
    match cwd {
        None => std::env::current_dir().ok(),
        Some(p) if p.as_os_str().is_empty() || p == Path::new(".") => {
            std::env::current_dir().ok()
        }
        Some(p) => Some(p.to_path_buf()),
    }
}

/// Skills list for Face Extensions Skills tab (`x.ai/skills/list`).
///
/// Wire shape matches grok-shell `ExtMethodResult { result: SkillsListResponse }`:
/// `{ "result": { "skills": [SkillInfo, ...] } }` so Face's
/// `wrapper.result.skills → Vec<SkillInfo>` parser succeeds.
///
/// Uses the same SkillRegistry sources as `$` / `availableCommands`
/// (`load_global` + best-effort project overlay). Never returns an empty
/// list solely because project-local overlay I/O failed.
pub fn list_nextcode_skills(cwd: Option<&Path>) -> serde_json::Value {
    let cwd = resolve_face_cwd(cwd);
    // Match `$` / InitializeResponse: global skills must always appear.
    // Project overlay is best-effort so a bad project dir cannot blank the tab.
    let mut registry = match crate::skill::SkillRegistry::load_global() {
        Ok(r) => r,
        Err(_) => {
            return serde_json::json!({
                "result": { "skills": [], "wireRev": FACE_EXT_WIRE_REV }
            });
        }
    };
    if let Ok(overlay) = crate::skill::SkillRegistry::load_project_overlay(cwd.as_deref()) {
        registry.merge_overlay(overlay);
    }
    let skills: Vec<serde_json::Value> = registry
        .list()
        .into_iter()
        .map(|skill| {
            let scope = if cwd
                .as_deref()
                .is_some_and(|wd| skill.path.starts_with(wd))
            {
                "repo"
            } else {
                "user"
            };
            // Fields required by xai_grok_tools::SkillInfo (Face deserializes this type).
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
    serde_json::json!({
        "result": {
            "skills": skills,
            "wireRev": FACE_EXT_WIRE_REV,
        }
    })
}

/// MCP server list for Face Extensions MCP Servers tab (`x.ai/mcp/list`).
///
/// Face parses `McpsListResponse { servers }` from `result` (or the top-level
/// object). Uses the catalog loader, then **probes** HTTP servers (initialize +
/// tools/list) so status/tools match runtime. Stdio rows stay `ready` without
/// spawning (spawn happens in the session/pool).
pub async fn list_nextcode_mcps(cwd: Option<&Path>) -> serde_json::Value {
    let cwd = resolve_face_cwd(cwd);
    let config = crate::mcp::McpConfig::load_catalog_for_dir(cwd.as_deref());
    let mut names: Vec<String> = config.servers.keys().cloned().collect();
    names.sort();

    let mut set = tokio::task::JoinSet::new();
    for name in names {
        let Some(cfg) = config.servers.get(&name).cloned() else {
            continue;
        };
        set.spawn(async move {
            let enabled = cfg.is_enabled();
            let is_http = cfg.is_http();
            let config_type = if is_http { "http" } else { "stdio" };

            let (status, source_label, tools) = if !enabled {
                (
                    "unavailable",
                    "~/.next-code/mcp.json".to_string(),
                    Vec::new(),
                )
            } else if is_http {
                match tokio::time::timeout(
                    Duration::from_secs(20),
                    crate::mcp::McpClient::connect(name.clone(), &cfg),
                )
                .await
                {
                    Ok(Ok(client)) => {
                        let tools: Vec<serde_json::Value> = client
                            .tools()
                            .into_iter()
                            .map(|t| {
                                serde_json::json!({
                                    "name": t.name,
                                    "description": t.description,
                                    "enabled": true,
                                })
                            })
                            .collect();
                        ("ready", "~/.next-code/mcp.json".to_string(), tools)
                    }
                    Ok(Err(e)) => (
                        "unavailable",
                        truncate_label(&format!("HTTP connect failed: {e}"), 120),
                        Vec::new(),
                    ),
                    Err(_) => (
                        "unavailable",
                        "HTTP connect timed out".to_string(),
                        Vec::new(),
                    ),
                }
            } else {
                ("ready", "~/.next-code/mcp.json".to_string(), Vec::new())
            };

            let mut entry = serde_json::json!({
                "name": name,
                "source": "local",
                "sourceLabel": source_label,
                "type": config_type,
                "session": {
                    "enabled": enabled,
                    "status": status,
                    "tools": tools,
                    "authRequired": false,
                    "setupRequired": false,
                }
            });
            if let Some(url) = cfg.url.as_ref() {
                entry
                    .as_object_mut()
                    .unwrap()
                    .insert("url".into(), serde_json::json!(url));
            }
            if !cfg.command.trim().is_empty() {
                let obj = entry.as_object_mut().unwrap();
                obj.insert("command".into(), serde_json::json!(cfg.command));
                if !cfg.args.is_empty() {
                    obj.insert("args".into(), serde_json::json!(cfg.args));
                }
            }
            entry
        });
    }

    let mut servers = Vec::new();
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(entry) => servers.push(entry),
            Err(e) => {
                eprintln!("[nextcode.face] mcp list probe task failed: {e}");
            }
        }
    }
    servers.sort_by(|a, b| {
        let an = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let bn = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        an.cmp(bn)
    });

    serde_json::json!({
        "result": {
            "servers": servers,
            "wireRev": FACE_EXT_WIRE_REV,
        }
    })
}

fn truncate_label(s: &str, max: usize) -> String {
    let t = s.trim().replace('\n', " ");
    if t.chars().count() <= max {
        t
    } else {
        let truncated: String = t.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

/// Empty-but-valid marketplace list so the Extensions fetch set does not error.
pub fn list_nextcode_marketplace() -> serde_json::Value {
    serde_json::json!({ "result": { "sources": [] } })
}

/// Cheap list-row metrics from flat `sessions/<id>.json` (+ journal appends).
///
/// Startup stubs omit transcript vectors, so Face resume briefs scan messages
/// here without a full `Session::load`. Skips system-reminder / display_role
/// system noise (same visibility idea as transcript preview).
#[derive(Debug, Default, Clone)]
struct SessionListBrief {
    first_prompt: Option<String>,
    num_messages: usize,
    user_messages: usize,
    assistant_messages: usize,
}

const FIRST_PROMPT_MAX_CHARS: usize = 72;

fn load_session_list_brief(sessions_dir: &Path, stem: &str) -> SessionListBrief {
    let mut messages: Vec<serde_json::Value> = Vec::new();
    let snapshot_path = sessions_dir.join(format!("{stem}.json"));
    if let Ok(raw) = std::fs::read_to_string(&snapshot_path)
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw)
        && let Some(arr) = value.get("messages").and_then(|m| m.as_array())
    {
        messages.extend(arr.iter().cloned());
    }
    let journal_path = sessions_dir.join(format!("{stem}.journal.jsonl"));
    if let Ok(raw) = std::fs::read_to_string(&journal_path) {
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                continue;
            };
            if let Some(appended) = value.get("append_messages").and_then(|v| v.as_array()) {
                messages.extend(appended.iter().cloned());
            }
        }
    }

    let mut brief = SessionListBrief::default();
    for msg in &messages {
        let display_role = msg
            .get("display_role")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if display_role.eq_ignore_ascii_case("system") {
            continue;
        }
        let text = list_message_text(msg);
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed.contains("<system-reminder>") {
            continue;
        }
        let role = msg
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        brief.num_messages += 1;
        match role.as_str() {
            "user" => {
                brief.user_messages += 1;
                if brief.first_prompt.is_none() {
                    brief.first_prompt = Some(truncate_label(trimmed, FIRST_PROMPT_MAX_CHARS));
                }
            }
            "assistant" => brief.assistant_messages += 1,
            _ => {}
        }
    }
    brief
}

fn list_message_text(msg: &serde_json::Value) -> String {
    let Some(content) = msg.get("content") else {
        return String::new();
    };
    if let Some(s) = content.as_str() {
        return s.to_string();
    }
    let Some(arr) = content.as_array() else {
        return String::new();
    };
    let mut parts = Vec::new();
    for part in arr {
        if part.get("type").and_then(|t| t.as_str()) == Some("text")
            && let Some(t) = part.get("text").and_then(|t| t.as_str())
        {
            parts.push(t);
        }
    }
    parts.join("\n")
}

/// Session picker payload for Face `/resume` (`x.ai/session/list`).
///
/// Shape matches Face `parse_session_picker_entries`: `sessionId`, `summary`,
/// timestamps, plus `cwd` / `modelId` / `source` so welcome grouping and
/// resume-by-cwd work against `~/.next-code/sessions`. Also emits
/// `customTitle`, `firstPrompt`, `shortName`, and user/assistant counts.
///
/// `summary` is Claude Code–style: custom/generated title → first user prompt
/// brief → memorable short_name last. Animal names are not the scannable title
/// when a chat brief exists.
pub fn list_nextcode_sessions(limit: usize) -> serde_json::Value {
    let Ok(base) = crate::storage::next_code_dir() else {
        return serde_json::json!({ "sessions": [] });
    };
    let dir = base.join("sessions");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return serde_json::json!({ "sessions": [] });
    };

    let mut rows: Vec<serde_json::Value> = Vec::new();
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
        let brief = load_session_list_brief(&dir, stem);
        // Claude Code–style scannable title: custom/generated title → first
        // user prompt brief → memorable short_name / id last resort.
        // Do not promote animal short_name when a chat brief exists.
        let summary = session
            .display_title()
            .map(|s| s.to_string())
            .or_else(|| brief.first_prompt.clone())
            .unwrap_or_else(|| session.display_name().to_string());
        if summary.trim().is_empty() {
            continue;
        }
        let last_active = session
            .last_active_at
            .unwrap_or(session.updated_at)
            .to_rfc3339();
        let updated = session.updated_at.to_rfc3339();
        let created = session.created_at.to_rfc3339();
        let cwd = session.working_dir.clone().unwrap_or_default();
        let model_id = session.model.clone();
        let short_name = session.short_name.clone();
        let custom_title = session.custom_title.clone();
        rows.push(serde_json::json!({
            "sessionId": stem,
            "summary": summary,
            "customTitle": custom_title,
            "shortName": short_name,
            "firstPrompt": brief.first_prompt,
            "updatedAt": updated,
            "createdAt": created,
            "lastActiveAt": last_active,
            "cwd": cwd,
            "modelId": model_id,
            "numMessages": brief.num_messages,
            "userMessages": brief.user_messages,
            "assistantMessages": brief.assistant_messages,
            "source": "local",
        }));
    }

    rows.sort_by(|a, b| {
        let a_key = a
            .get("lastActiveAt")
            .or_else(|| a.get("updatedAt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let b_key = b
            .get("lastActiveAt")
            .or_else(|| b.get("updatedAt"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        b_key.cmp(a_key)
    });
    rows.truncate(limit.max(1));

    serde_json::json!({ "sessions": rows })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Face auth pending state is process-global; serialize tests that touch it.
    static AUTH_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn list_mcps_sync(cwd: Option<&Path>) -> serde_json::Value {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime")
            .block_on(list_nextcode_mcps(cwd))
    }

    #[test]
    fn method_id_roundtrip() {
        assert_eq!(method_id_for_provider("openrouter"), "nextcode.openrouter");
        assert_eq!(provider_id_from_method("nextcode.openrouter"), Some("openrouter"));
        assert!(is_nextcode_auth_method("nextcode.claude"));
        assert!(!is_nextcode_auth_method("xai.api_key"));
    }

    #[test]
    fn submit_auth_code_buffers_when_not_waiting() {
        let _guard = AUTH_TEST_LOCK.lock().expect("auth test lock");
        clear_pending();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        // Premature submit must not Internal-error; it buffers for face_prompt_paste.
        rt.block_on(async {
            submit_auth_code("sk-early-key")
                .await
                .expect("premature submit should Ok");
        });
        assert_eq!(take_early_auth_code().as_deref(), Some("sk-early-key"));
        clear_pending();
    }

    #[test]
    fn submit_auth_code_duplicate_after_take_is_noop() {
        let _guard = AUTH_TEST_LOCK.lock().expect("auth test lock");
        clear_pending();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        rt.block_on(async {
            let (tx, rx) = tokio::sync::oneshot::channel();
            {
                let mut g = PENDING.lock().expect("lock");
                *g = Some(PendingFaceLogin {
                    provider_id: "opencode-go".into(),
                    auth_url: Some("https://opencode.ai/go".into()),
                    mode: "loopback".into(),
                    kind: PendingKind::ApiKeyPaste,
                    code_tx: Some(tx),
                });
            }
            submit_auth_code("sk-first").await.expect("first");
            let got = rx.await.expect("oneshot value");
            assert_eq!(got, "sk-first");
            // Second submit after oneshot consumed — must Ok, not "not waiting".
            submit_auth_code("sk-second").await.expect("duplicate");
            clear_pending();
        });
    }

    #[test]
    fn catalog_has_providers() {
        use crate::provider_catalog::tui_login_providers;
        assert!(!tui_login_providers().is_empty());
    }

    #[test]
    fn skills_list_wire_shape_has_result_skills_array() {
        let payload = list_nextcode_skills(Some(Path::new(".")));
        let skills = payload
            .pointer("/result/skills")
            .and_then(|v| v.as_array())
            .expect("Face expects result.skills array");
        eprintln!("SKILLS_COUNT={}", skills.len());
        for skill in skills.iter().take(3) {
            eprintln!(
                "SKILL_SAMPLE name={:?} path={:?}",
                skill.get("name"),
                skill.get("path")
            );
        }
        for skill in skills {
            assert!(skill.get("name").and_then(|v| v.as_str()).is_some());
            assert!(skill.get("description").and_then(|v| v.as_str()).is_some());
            assert!(skill.get("path").and_then(|v| v.as_str()).is_some());
            let scope = skill.get("scope").and_then(|v| v.as_str()).unwrap_or("");
            assert!(
                matches!(scope, "user" | "repo" | "local" | "plugin" | "bundled" | "server"),
                "unexpected scope {scope}"
            );
        }
        // On a developer machine with ~/.agents/skills or ~/.next-code/skills,
        // the dialog must not return an empty list while `$` works.
        // Soft-check: if global dirs exist, count must be > 0.
        let home = dirs::home_dir().unwrap_or_default();
        let has_global = home.join(".agents").join("skills").is_dir()
            || home.join(".next-code").join("skills").is_dir();
        if has_global {
            assert!(
                !skills.is_empty(),
                "skills list empty despite global skill dirs; Face would show No matches"
            );
        }
    }

    #[test]
    fn mcp_list_wire_shape_has_result_servers_array() {
        let payload = list_mcps_sync(None);
        eprintln!(
            "MCP_PAYLOAD={}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        let servers = payload
            .pointer("/result/servers")
            .and_then(|v| v.as_array())
            .expect("Face expects result.servers array");
        eprintln!("MCP_COUNT={}", servers.len());
        for server in servers {
            assert!(server.get("name").and_then(|v| v.as_str()).is_some());
            assert_eq!(
                server.get("source").and_then(|v| v.as_str()),
                Some("local")
            );
            assert!(server.pointer("/session/enabled").and_then(|v| v.as_bool()).is_some());
        }
        let mcp_json = dirs::home_dir()
            .unwrap_or_default()
            .join(".next-code")
            .join("mcp.json");
        if mcp_json.is_file() {
            assert!(
                !servers.is_empty(),
                "mcp.json exists but list_nextcode_mcps returned no servers"
            );
        }
    }

    #[test]
    fn mcp_list_includes_http_servers_from_user_mcp_json() {
        let payload = list_mcps_sync(None);
        let servers = payload
            .pointer("/result/servers")
            .and_then(|v| v.as_array())
            .expect("servers array");
        let mcp_json = dirs::home_dir()
            .unwrap_or_default()
            .join(".next-code")
            .join("mcp.json");
        if !mcp_json.is_file() {
            return;
        }
        assert!(
            !servers.is_empty(),
            "HTTP-only mcp.json must still list servers in Face catalog"
        );
        let names: Vec<&str> = servers
            .iter()
            .filter_map(|s| s.get("name").and_then(|v| v.as_str()))
            .collect();
        assert!(
            names.iter().any(|n| *n == "exa" || *n == "deepwiki" || *n == "livekit-docs"),
            "expected known HTTP server names, got {names:?}"
        );
        for server in servers {
            let is_http_name = matches!(
                server.get("name").and_then(|v| v.as_str()),
                Some("exa" | "deepwiki" | "livekit-docs" | "twilio-docs")
            );
            if !is_http_name {
                continue;
            }
            assert_eq!(
                server.get("type").and_then(|v| v.as_str()),
                Some("http")
            );
            let status = server.pointer("/session/status").and_then(|v| v.as_str());
            let label = server.get("sourceLabel").and_then(|v| v.as_str()).unwrap_or("");
            // Healthy → ready + tools; offline/auth → unavailable with connect error (not the old honesty stub).
            assert!(
                status == Some("ready")
                    || (status == Some("unavailable")
                        && (label.contains("HTTP connect") || label.contains("timed out"))),
                "unexpected HTTP status={status:?} label={label:?}"
            );
            assert_ne!(
                label,
                "HTTP — next-code connects stdio MCP only",
                "honesty stub must be gone after HTTP port"
            );
        }
    }

    #[test]
    fn mcp_list_face_mcps_list_response_roundtrip() {
        let payload = list_mcps_sync(None);
        let body = serde_json::to_string(&payload).expect("serialize");
        let converted = xai_grok_pager::views::mcps_modal::parse_mcp_list_ext_response(&body)
            .expect("Face mcp list parse");
        let mcp_json = dirs::home_dir()
            .unwrap_or_default()
            .join(".next-code")
            .join("mcp.json");
        if mcp_json.is_file() {
            assert!(
                !converted.is_empty(),
                "Face convert_list_response empty despite mcp.json; UI would show No matches"
            );
        }
    }

    #[test]
    fn skills_list_count_matches_available_commands_source() {
        let payload = list_nextcode_skills(None);
        let n = payload
            .pointer("/result/skills")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let global = crate::skill::SkillRegistry::load_global()
            .map(|r| r.list().len())
            .unwrap_or(0);
        // Dialog may include project overlay on top of global (`$` / welcome
        // often uses session cwd effective set ≈ global+overlay too).
        assert!(
            n >= global,
            "dialog skills count {n} < load_global count {global}"
        );
        assert!(n >= 40, "expected ~47 skills on this machine, got {n}");
        assert_eq!(
            payload.pointer("/result/wireRev").and_then(|v| v.as_str()),
            Some(FACE_EXT_WIRE_REV)
        );
    }

    #[test]
    fn skills_list_face_skill_info_roundtrip() {
        let payload = list_nextcode_skills(None);
        let wrapper = payload.clone();
        let inner = wrapper.get("result").unwrap_or(&wrapper);
        let skills_val = match inner.get("skills") {
            Some(v) if !v.is_null() => v.clone(),
            _ => serde_json::json!([]),
        };
        let parsed: Result<
            Vec<xai_grok_tools::implementations::skills::types::SkillInfo>,
            _,
        > = serde_json::from_value(skills_val);
        assert!(
            parsed.is_ok(),
            "Face SkillInfo deserialize failed: {:?}\ninner={}",
            parsed.err(),
            inner
        );
        let skills = parsed.unwrap();
        let home = dirs::home_dir().unwrap_or_default();
        let has_global = home.join(".agents").join("skills").is_dir()
            || home.join(".next-code").join("skills").is_dir();
        if has_global {
            assert!(
                !skills.is_empty(),
                "SkillInfo roundtrip empty despite global skill dirs"
            );
        }
    }
}
