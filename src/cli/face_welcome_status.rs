//! Gather next-code launch snapshot for Face welcome chrome.
//!
//! Copies formatting structure from legacy TUI `ui_header` (`build_auth_status_line`,
//! `build_persistent_header`, `build_header_lines`) without Face depending on
//! `next-code-tui` presentation. Face only paints `ProductWelcomeStatus`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::auth::{AuthState, AuthStatus, ActiveCredential, resolve_dual_credential_auth};
use crate::env::product_env;
use xai_grok_pager::product_welcome::{
    AuthDotEntry, AuthDotState, ProductWelcomeStatus, animal_version_label, compact_version_label,
    format_badge_line, format_built_line, format_client_animal_line, format_mcp_line,
    format_model_switch_parts, format_server_animal_line, format_sessions_line, format_skills_line,
    install_product_welcome_status,
};

fn map_auth_state(state: AuthState) -> AuthDotState {
    match state {
        AuthState::Available => AuthDotState::Available,
        AuthState::Expired => AuthDotState::Expired,
        AuthState::NotConfigured => AuthDotState::NotConfigured,
    }
}

fn provider_label(name: &str, state: AuthState, method: Option<&str>) -> String {
    match (state, method) {
        (AuthState::NotConfigured, _) => name.to_string(),
        (_, Some(method)) if !method.is_empty() => format!("{name}({method})"),
        _ => name.to_string(),
    }
}

fn dual_method_label(
    provider: next_code_provider_core::ActiveProvider,
    auth: &AuthStatus,
) -> Option<&'static str> {
    let runtime_provider = product_env("RUNTIME_PROVIDER").ok();
    let resolved = resolve_dual_credential_auth(provider, auth, runtime_provider.as_deref())?;
    Some(match (resolved.has_oauth, resolved.has_api_key) {
        (true, true) => match resolved.active {
            ActiveCredential::OAuth => "oauth*+key",
            ActiveCredential::ApiKey => "oauth+key*",
        },
        (true, false) => "oauth",
        (false, true) => "key",
        (false, false) => return None,
    })
}

fn rendered_width(entries: &[&str]) -> usize {
    if entries.is_empty() {
        return 0;
    }
    entries.iter().map(|label| label.len() + 3).sum::<usize>() + (entries.len() - 1)
}

/// Copy of TUI `build_auth_status_line` inventory (configured providers only).
pub(crate) fn build_auth_dot_entries(auth: &AuthStatus, max_width: usize) -> Vec<AuthDotEntry> {
    let anthropic_label = provider_label(
        "anthropic",
        auth.anthropic.state,
        dual_method_label(next_code_provider_core::ActiveProvider::Claude, auth),
    );
    let openai_label = provider_label(
        "openai",
        auth.openai,
        dual_method_label(next_code_provider_core::ActiveProvider::OpenAI, auth),
    );
    let gemini_label = if auth.gemini != AuthState::NotConfigured {
        provider_label("gemini", auth.gemini, Some("oauth"))
    } else {
        provider_label("gemini", auth.gemini, None)
    };
    let gemini_compact_label = if auth.gemini != AuthState::NotConfigured {
        provider_label("ge", auth.gemini, Some("oauth"))
    } else {
        provider_label("ge", auth.gemini, None)
    };

    let full_specs: Vec<(String, AuthState)> = vec![
        (anthropic_label, auth.anthropic.state),
        ("openrouter".to_string(), auth.openrouter),
        (openai_label, auth.openai),
        (provider_label("cursor", auth.cursor, None), auth.cursor),
        (provider_label("copilot", auth.copilot, None), auth.copilot),
        (gemini_label, auth.gemini),
        (
            provider_label("antigravity", auth.antigravity, None),
            auth.antigravity,
        ),
    ]
    .into_iter()
    .filter(|(_, state)| *state != AuthState::NotConfigured)
    .collect();

    let compact_specs: Vec<(String, AuthState)> = vec![
        (
            provider_label("an", auth.anthropic.state, None),
            auth.anthropic.state,
        ),
        ("or".to_string(), auth.openrouter),
        (provider_label("oa", auth.openai, None), auth.openai),
        (provider_label("cu", auth.cursor, None), auth.cursor),
        (provider_label("cp", auth.copilot, None), auth.copilot),
        (gemini_compact_label, auth.gemini),
        (
            provider_label("ag", auth.antigravity, None),
            auth.antigravity,
        ),
    ]
    .into_iter()
    .filter(|(_, state)| *state != AuthState::NotConfigured)
    .collect();

    let full: Vec<&str> = full_specs.iter().map(|(label, _)| label.as_str()).collect();
    let compact: Vec<&str> = compact_specs
        .iter()
        .map(|(label, _)| label.as_str())
        .collect();

    let provider_specs: Vec<&(String, AuthState)> = if rendered_width(&full) <= max_width {
        full_specs.iter().collect()
    } else if rendered_width(&compact) <= max_width {
        compact_specs.iter().collect()
    } else {
        compact_specs.iter().take(4).collect()
    };

    provider_specs
        .into_iter()
        .map(|(label, state)| AuthDotEntry {
            state: map_auth_state(*state),
            label: label.clone(),
        })
        .collect()
}

/// Active-route auth tag for the model line (copy of TUI `header_provider_auth_tag`).
fn header_provider_auth_tag(name: &str, auth: &AuthStatus) -> &'static str {
    let runtime_provider = product_env("RUNTIME_PROVIDER").ok();
    if let Some(provider) = next_code_provider_core::parse_provider_hint(name) {
        match resolve_dual_credential_auth(provider, auth, runtime_provider.as_deref()) {
            Some(resolved) => {
                return match resolved.active {
                    ActiveCredential::OAuth => "oauth",
                    ActiveCredential::ApiKey => "api-key",
                };
            }
            None if matches!(
                provider,
                next_code_provider_core::ActiveProvider::Claude
                    | next_code_provider_core::ActiveProvider::OpenAI
            ) =>
            {
                return "";
            }
            None => {}
        }
    }

    match name {
        "copilot" => {
            if auth.copilot_has_api_token {
                "oauth"
            } else {
                ""
            }
        }
        "openrouter" | "openai-compatible" => "api-key",
        other
            if crate::provider_catalog::resolve_openai_compatible_profile_selection(other)
                .is_some()
                || crate::provider_catalog::openai_compatible_profile_id_for_display_name(other)
                    .is_some() =>
        {
            "api-key"
        }
        _ => "",
    }
}

fn header_provider_label(provider_name: &str, auth: &AuthStatus) -> String {
    let trimmed = provider_name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let name = trimmed.to_lowercase();
    let auth_tag = header_provider_auth_tag(&name, auth);
    if auth_tag.is_empty() {
        name
    } else {
        format!("{auth_tag}:{name}")
    }
}

fn welcome_cwd(remote_working_dir: Option<&str>) -> PathBuf {
    remote_working_dir
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn welcome_skills_line(cwd: &Path) -> Option<String> {
    let registry = crate::skill::SkillRegistry::load_for_working_dir(Some(cwd)).ok()?;
    let names: Vec<String> = registry.list().into_iter().map(|s| s.name.clone()).collect();
    format_skills_line(&names)
}

fn welcome_mcp_line(cwd: &Path) -> String {
    let config = crate::mcp::McpConfig::load_for_dir(Some(cwd));
    let mut names: Vec<String> = config.servers.keys().cloned().collect();
    names.sort();
    format_mcp_line(&names)
}

fn used_session_names(server_info: Option<&crate::registry::ServerInfo>) -> HashSet<String> {
    let mut used = crate::storage::active_session_ids()
        .into_iter()
        .filter_map(|id| crate::id::extract_session_name(&id).map(str::to_string))
        .collect::<HashSet<_>>();
    if let Some(info) = server_info {
        for id in &info.sessions {
            if let Some(name) = crate::id::extract_session_name(id) {
                used.insert(name.to_string());
            }
        }
    }
    used
}

/// Install Face welcome chrome from next-code state (legacy splash field set).
pub(crate) fn install_face_welcome_status(
    resume_session: Option<&str>,
    remote_working_dir: Option<&str>,
) {
    let cwd = welcome_cwd(remote_working_dir);
    let update_bullets = next_code_build_meta::take_unseen_changelog_entries()
        .iter()
        .cloned()
        .collect();

    let server_info = crate::registry::find_server_by_socket_sync(&crate::server::socket_path());
    let auth = AuthStatus::check_fast();

    let client_version_full = next_code_build_meta::VERSION.to_string();
    let server_version_full = server_info
        .as_ref()
        .map(|info| info.version.clone())
        .filter(|v| !v.trim().is_empty());
    let version_mismatch = matches!(
        (&server_version_full, &client_version_full),
        (Some(server), client) if server.trim() != client.trim()
    );
    let include_hash = version_mismatch
        && matches!(
            (&server_version_full, &client_version_full),
            (Some(server), client)
                if compact_version_label(server) == compact_version_label(client)
        );
    let server_version_label = server_version_full
        .as_deref()
        .map(|v| animal_version_label(v, include_hash));
    let client_version_label = animal_version_label(&client_version_full, include_hash);

    let server_line = server_info.as_ref().map(|info| {
        let icon = if info.icon.trim().is_empty() {
            crate::id::server_icon(&info.name).to_string()
        } else {
            info.icon.clone()
        };
        format_server_animal_line(&info.name, &icon, server_version_label.as_deref())
    });

    // Resume → extract name from session id. Fresh → same memorable allocator TUI uses
    // in Session::create (`new_memorable_session_id_avoiding`).
    let client_line = resume_session
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|id| crate::id::extract_session_name(id).map(str::to_string))
        .or_else(|| {
            let used = used_session_names(server_info.as_ref());
            let (_, name) = crate::id::new_memorable_session_id_avoiding(&used);
            Some(name)
        })
        .map(|name| {
            let icon = crate::id::session_icon(&name);
            format_client_animal_line(&name, icon, Some(&client_version_label))
        });

    let mut badge_items: Vec<&str> = Vec::new();
    if server_info.is_some() {
        badge_items.push("client");
    }
    if let Some(badge) = crate::perf::profile().tier.badge() {
        badge_items.push(badge);
    }
    let badge_line = format_badge_line(&badge_items);

    let sessions_line = server_info
        .as_ref()
        .and_then(|info| format_sessions_line(None, info.sessions.len()));

    let cfg = crate::config::Config::load();
    let model = cfg
        .provider
        .default_model
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    let provider = cfg
        .provider
        .default_provider
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    let provider_label = if model.is_empty() {
        String::new()
    } else {
        header_provider_label(provider, &auth)
    };
    let (model_prefix, model_name, model_line) =
        format_model_switch_parts(Some(provider_label.as_str()).filter(|s| !s.is_empty()), model);

    let build_age = next_code_build_meta::binary_age();
    let built_line = build_age.as_deref().and_then(format_built_line);

    install_product_welcome_status(ProductWelcomeStatus {
        model_prefix,
        model_name,
        model_line,
        build_age,
        built_line,
        update_bullets,
        badge_line,
        server_line,
        client_line,
        version_mismatch,
        auth_entries: build_auth_dot_entries(&auth, 120),
        mcp_line: Some(welcome_mcp_line(&cwd)),
        skills_line: welcome_skills_line(&cwd),
        sessions_line,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthState, AuthStatus, ProviderAuth};

    #[test]
    fn auth_dots_include_configured_only() {
        let auth = AuthStatus {
            openrouter: AuthState::Available,
            anthropic: ProviderAuth {
                state: AuthState::Available,
                has_oauth: true,
                oauth_state: AuthState::Available,
                has_api_key: false,
            },
            ..AuthStatus::default()
        };
        let entries = build_auth_dot_entries(&auth, 120);
        let labels: Vec<&str> = entries.iter().map(|e| e.label.as_str()).collect();
        assert!(labels.iter().any(|l| l.contains("openrouter")));
        assert!(labels.iter().any(|l| l.contains("anthropic")));
        assert!(!labels.iter().any(|l| *l == "copilot"));
        assert!(entries.iter().all(|e| e.state == AuthDotState::Available));
    }

    #[test]
    fn auth_dots_empty_when_nothing_configured() {
        assert!(build_auth_dot_entries(&AuthStatus::default(), 120).is_empty());
    }

    #[test]
    fn provider_label_uses_api_key_tag_for_openrouter() {
        let auth = AuthStatus {
            openrouter: AuthState::Available,
            ..AuthStatus::default()
        };
        assert_eq!(
            header_provider_label("openrouter", &auth),
            "api-key:openrouter"
        );
    }
}
