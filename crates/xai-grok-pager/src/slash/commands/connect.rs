//! `/connect` — next-code multi-provider login (Face chrome).
//!
//! OpenCode-shaped wizard: searchable provider picker → optional auth-method
//! step → Face welcome paste/URL flow. Credentials write via daemon
//! `face_auth` / `~/.next-code` (auth.json on the auth-unify branch).
//! Does **not** start Grok OAuth and does **not** hand off to a CLI terminal.

use next_code_provider_metadata::{
    LoginProviderAuthKind, LoginProviderAuthStateKey, LoginProviderDescriptor,
};

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

/// Popular family anchors (OpenCode-style). Resolved against
/// `tui_login_providers()`; missing ids are skipped.
const POPULAR_CONNECT_IDS: &[&str] = &[
    "claude",
    "openai",
    "gemini",
    "openrouter",
    "xai",
    "copilot",
];

pub struct ConnectCommand;

impl SlashCommand for ConnectCommand {
    fn name(&self) -> &str {
        "connect"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn description(&self) -> &str {
        "Connect a model provider (next-code multi-provider auth)"
    }

    fn usage(&self) -> &str {
        "/connect [provider]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn args_required(&self) -> bool {
        false
    }

    fn arg_placeholder(&self) -> Option<&str> {
        // Picker-first: bare `/connect` opens Connect-a-provider ArgPicker.
        // No inline `provider` ghost — that path stole Enter (trailing space).
        None
    }

    fn suggest_args(&self, _ctx: &AppCtx, args_query: &str) -> Option<Vec<ArgItem>> {
        // Empty args → centered picker (`OpenConnectPicker`), not inline dump.
        if args_query.trim().is_empty() {
            return None;
        }
        Some(suggest_connect_args(args_query))
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        connect_run(args)
    }
}

/// Shared by `/connect` and embed `/login`.
pub(crate) fn connect_run(args: &str) -> CommandResult {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        return CommandResult::Action(Action::OpenConnectPicker);
    }

    let Some(provider) = next_code_provider_metadata::resolve_login_provider(trimmed).or_else(
        || next_code_provider_metadata::resolve_login_provider_loose(trimmed),
    ) else {
        return CommandResult::Error(format!(
            "Unknown provider {trimmed:?}. Try /connect and pick from the list."
        ));
    };

    // Multi-method family passed as the representative id alone → still start
    // that specific catalog entry (typed `/connect claude` = OAuth). Method
    // chooser is picker-only (trailing-space chain).
    CommandResult::Action(Action::NextCodeConnect {
        provider: provider.id.to_string(),
    })
}

pub(crate) fn suggest_connect_args(args_query: &str) -> Vec<ArgItem> {
    let trimmed = args_query.trim();
    if !trimmed.is_empty()
        && let Some(methods) = connect_method_items_for_query(args_query)
    {
        return methods;
    }
    build_connect_family_items()
}

/// Flat searchable list used by OpenConnectPicker and Tab suggest.
pub(crate) fn build_connect_family_items() -> Vec<ArgItem> {
    let providers = next_code_provider_metadata::tui_login_providers();
    let families = group_families(&providers);

    let mut out = Vec::new();
    let mut popular_keys: Vec<LoginProviderAuthStateKey> = Vec::new();

    out.push(section_header("Popular"));
    for id in POPULAR_CONNECT_IDS {
        let Some(family) = families
            .iter()
            .find(|f| f.iter().any(|p| p.id == *id))
        else {
            continue;
        };
        let key = family[0].auth_state_key;
        if popular_keys.contains(&key) {
            continue;
        }
        popular_keys.push(key);
        out.push(family_arg_item(family));
    }
    // Recommended families not already listed under Popular.
    for family in &families {
        if !family.iter().any(|p| p.recommended) {
            continue;
        }
        let key = family[0].auth_state_key;
        if popular_keys.contains(&key) {
            continue;
        }
        popular_keys.push(key);
        out.push(family_arg_item(family));
    }

    out.push(section_header("Providers"));
    for family in &families {
        let key = family[0].auth_state_key;
        if popular_keys.contains(&key) {
            continue;
        }
        out.push(family_arg_item(family));
    }
    out
}

fn connect_method_items_for_query(args_query: &str) -> Option<Vec<ArgItem>> {
    // Chained from a family row: insert_text ends with whitespace.
    if !args_query.ends_with(char::is_whitespace) {
        return None;
    }
    let id = args_query.trim();
    if id.is_empty() {
        return None;
    }
    let provider = next_code_provider_metadata::resolve_login_provider(id)
        .or_else(|| next_code_provider_metadata::resolve_login_provider_loose(id))?;
    let methods = methods_for_key(
        &next_code_provider_metadata::tui_login_providers(),
        provider.auth_state_key,
    );
    if methods.len() < 2 {
        return None;
    }
    Some(build_connect_method_items(&methods))
}

fn build_connect_method_items(methods: &[LoginProviderDescriptor]) -> Vec<ArgItem> {
    methods
        .iter()
        .copied()
        .map(|p| ArgItem {
            display: method_display_name(p),
            match_text: format!("{} {} {}", p.display_name, p.id, p.auth_kind.label()),
            insert_text: p.id.to_string(),
            description: p.menu_detail.to_string(),
            category: Some("Select auth method".into()),
            badge: None,
            is_current: false,
            provider_connect: true,
            is_section_header: false,
        })
        .collect()
}

fn group_families(providers: &[LoginProviderDescriptor]) -> Vec<Vec<LoginProviderDescriptor>> {
    // Only collapse into a multi-method family when the same auth_state_key
    // has both a browser path (OAuth/device) and a key/local path — e.g.
    // Anthropic (`claude` + `anthropic-api`). Shared keys that are all API-key
    // (OpenRouterLike) stay as separate single-method rows.
    let mut fork_keys: Vec<LoginProviderAuthStateKey> = Vec::new();
    let mut seen_keys: Vec<LoginProviderAuthStateKey> = Vec::new();
    for p in providers {
        if seen_keys.contains(&p.auth_state_key) {
            continue;
        }
        seen_keys.push(p.auth_state_key);
        let methods = methods_for_key(providers, p.auth_state_key);
        let has_browser = methods.iter().any(|m| {
            matches!(
                m.auth_kind,
                LoginProviderAuthKind::OAuth | LoginProviderAuthKind::DeviceCode
            )
        });
        let has_key = methods.iter().any(|m| {
            matches!(
                m.auth_kind,
                LoginProviderAuthKind::ApiKey
                    | LoginProviderAuthKind::Hybrid
                    | LoginProviderAuthKind::Local
            )
        });
        if has_browser && has_key && methods.len() > 1 {
            fork_keys.push(p.auth_state_key);
        }
    }

    let mut used_ids: Vec<&str> = Vec::new();
    let mut families = Vec::new();
    for p in providers {
        if used_ids.contains(&p.id) {
            continue;
        }
        if fork_keys.contains(&p.auth_state_key) {
            let methods = methods_for_key(providers, p.auth_state_key);
            for m in &methods {
                used_ids.push(m.id);
            }
            families.push(methods);
        } else {
            used_ids.push(p.id);
            families.push(vec![*p]);
        }
    }
    families
}

fn methods_for_key(
    providers: &[LoginProviderDescriptor],
    key: LoginProviderAuthStateKey,
) -> Vec<LoginProviderDescriptor> {
    providers
        .iter()
        .copied()
        .filter(|p| p.auth_state_key == key)
        .collect()
}

fn family_representative(methods: &[LoginProviderDescriptor]) -> LoginProviderDescriptor {
    methods
        .iter()
        .copied()
        .find(|p| p.recommended)
        .or_else(|| methods.first().copied())
        .expect("non-empty family")
}

fn family_arg_item(methods: &[LoginProviderDescriptor]) -> ArgItem {
    let rep = family_representative(methods);
    let multi = methods.len() > 1;
    let (insert_text, provider_connect, description) = if multi {
        // Trailing space → ArgPicker chains into suggest_args method phase.
        (
            format!("{} ", rep.id),
            false,
            multi_method_hint(methods),
        )
    } else {
        (
            rep.id.to_string(),
            true,
            single_method_hint(rep),
        )
    };
    ArgItem {
        display: rep.display_name.to_string(),
        match_text: format!(
            "{} {} {}",
            rep.display_name,
            methods
                .iter()
                .map(|m| m.id)
                .collect::<Vec<_>>()
                .join(" "),
            description
        ),
        insert_text,
        description,
        category: None,
        badge: if methods.iter().any(|m| m.recommended) {
            Some("Recommended".into())
        } else {
            None
        },
        is_current: false,
        provider_connect,
        is_section_header: false,
    }
}

fn single_method_hint(p: LoginProviderDescriptor) -> String {
    if p.menu_detail.is_empty() {
        p.auth_kind.label().to_string()
    } else {
        format!("{} · {}", p.auth_kind.label(), p.menu_detail)
    }
}

fn multi_method_hint(methods: &[LoginProviderDescriptor]) -> String {
    let kinds: Vec<&str> = methods
        .iter()
        .map(|m| match m.auth_kind {
            LoginProviderAuthKind::OAuth => "OAuth",
            LoginProviderAuthKind::ApiKey => "API key",
            LoginProviderAuthKind::DeviceCode => "device code",
            LoginProviderAuthKind::Cli => "CLI",
            LoginProviderAuthKind::Hybrid => "API key / CLI",
            LoginProviderAuthKind::Local => "local",
        })
        .collect();
    format!("{} · pick method", kinds.join(" or "))
}

fn method_display_name(p: LoginProviderDescriptor) -> String {
    match p.auth_kind {
        LoginProviderAuthKind::OAuth => format!("{} (OAuth)", p.display_name),
        LoginProviderAuthKind::ApiKey => format!("{} (API key)", p.display_name),
        LoginProviderAuthKind::DeviceCode => format!("{} (device code)", p.display_name),
        LoginProviderAuthKind::Cli => format!("{} (CLI)", p.display_name),
        LoginProviderAuthKind::Hybrid => format!("{} (API key / CLI)", p.display_name),
        LoginProviderAuthKind::Local => format!("{} (local)", p.display_name),
    }
}

fn section_header(label: &str) -> ArgItem {
    ArgItem {
        display: label.into(),
        match_text: String::new(),
        insert_text: String::new(),
        description: String::new(),
        category: Some(label.into()),
        badge: None,
        is_current: false,
        provider_connect: false,
        is_section_header: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;
    use crate::slash::command::CommandExecCtx;

    static DEFAULT_BUNDLE_STATE: crate::app::bundle::BundleState = crate::app::bundle::BundleState {
        has_cache: false,
        version: String::new(),
        personas: Vec::new(),
        roles: Vec::new(),
        agents: Vec::new(),
        skills: Vec::new(),
        persona_details: Vec::new(),
        role_details: Vec::new(),
    };

    fn make_ctx<'a>(models: &'a ModelState) -> CommandExecCtx<'a> {
        CommandExecCtx {
            models,
            session_id: None,
            bundle_state: &DEFAULT_BUNDLE_STATE,
            screen_mode: crate::app::ScreenMode::Inline,
            pager_state: crate::settings::PagerLocalSnapshot {
                multiline_mode: false,
                yolo_mode: false,
                ..crate::settings::PagerLocalSnapshot::default()
            },
        }
    }

    #[test]
    fn bare_connect_opens_picker() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match ConnectCommand.run(&mut ctx, "") {
            CommandResult::Action(Action::OpenConnectPicker) => {}
            other => panic!("expected OpenConnectPicker, got {other:?}"),
        }
    }

    #[test]
    fn suggest_args_has_popular_and_providers() {
        let items = build_connect_family_items();
        assert!(
            items
                .iter()
                .any(|i| i.is_section_header && i.display == "Popular"),
            "{items:?}"
        );
        assert!(
            items
                .iter()
                .any(|i| i.is_section_header && i.display == "Providers"),
            "{items:?}"
        );
        assert!(items.iter().any(|i| !i.is_section_header));
    }

    #[test]
    fn anthropic_family_chains_to_method_picker() {
        let family = build_connect_family_items()
            .into_iter()
            .find(|i| !i.is_section_header && i.display.contains("Claude"))
            .expect("claude family");
        assert!(
            family.insert_text.ends_with(' '),
            "multi-method family needs trailing space, got {:?}",
            family.insert_text
        );
        let methods = suggest_connect_args(&family.insert_text);
        assert!(
            methods.iter().any(|i| i.insert_text == "claude"),
            "{methods:?}"
        );
        assert!(
            methods.iter().any(|i| i.insert_text == "anthropic-api"),
            "{methods:?}"
        );
        assert!(methods.iter().all(|i| i.provider_connect || i.is_section_header));
    }

    #[test]
    fn suggest_args_empty_is_none_picker_first() {
        let models = ModelState::default();
        let cwd = std::path::Path::new(".");
        let ctx = AppCtx {
            models: &models,
            cwd,
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
        };
        assert!(
            ConnectCommand.suggest_args(&ctx, "").is_none(),
            "bare /connect must not feed inline arg dropdown"
        );
        assert!(
            ConnectCommand.suggest_args(&ctx, "   ").is_none(),
            "whitespace-only args still picker-first"
        );
        let typed = ConnectCommand
            .suggest_args(&ctx, "cl")
            .expect("typed prefix still suggests");
        assert!(!typed.is_empty());
    }

    #[test]
    fn known_provider_dispatches_face_login() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let providers = next_code_provider_metadata::tui_login_providers();
        let id = providers.first().expect("catalog").id;
        match ConnectCommand.run(&mut ctx, id) {
            CommandResult::Action(Action::NextCodeConnect { provider }) => {
                assert_eq!(provider, id);
            }
            other => panic!("expected NextCodeConnect, got {other:?}"),
        }
    }
}
