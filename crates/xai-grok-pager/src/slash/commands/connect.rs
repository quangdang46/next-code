//! `/connect` — next-code multi-provider login (Face chrome).
//!
//! OpenCode-shaped wizard (see OpenCode `dialog-provider.tsx`):
//! searchable provider picker → optional auth-method step → Face welcome
//! paste/URL flow → model picker (PR #72). Credentials write via daemon
//! `face_auth` / `~/.next-code`. Does **not** start Grok OAuth.

use next_code_provider_metadata::{
    LoginProviderAuthKind, LoginProviderAuthStateKey, LoginProviderDescriptor,
    LoginProviderTarget,
};

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

/// OpenCode TUI `PROVIDER_PRIORITY` order (`dialog-provider.tsx`).
/// Resolved against Face-wired `tui_login_providers()`; missing ids skipped.
const POPULAR_CONNECT_IDS: &[&str] = &[
    "opencode",
    "opencode-go",
    "openai",
    "copilot",
    "claude",
    "gemini",
];

/// OpenCode list-row descriptions (same file, `providerOptions` map).
fn opencode_row_description(id: &str) -> Option<&'static str> {
    match id {
        "opencode" => Some("(Recommended)"),
        "opencode-go" => Some("Low cost subscription for everyone"),
        "openai" => Some("(ChatGPT Plus/Pro or API key)"),
        "claude" | "anthropic" => Some("(OAuth or API key)"),
        "anthropic-api" => Some("(API key)"),
        _ => None,
    }
}

/// Display names matching OpenCode models.dev / TUI (not nextcode CLI labels).
fn opencode_display_name(rep: LoginProviderDescriptor) -> &'static str {
    match rep.id {
        "claude" => "Anthropic",
        "gemini" => "Google",
        "opencode" => "OpenCode Zen",
        "opencode-go" => "OpenCode Go",
        other => {
            // Prefer catalog display when it already matches product naming.
            if other == "copilot" {
                "GitHub Copilot"
            } else {
                rep.display_name
            }
        }
    }
}

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
        None
    }

    fn suggest_args(&self, _ctx: &AppCtx, args_query: &str) -> Option<Vec<ArgItem>> {
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

    if !face_connect_wired(provider) {
        return CommandResult::Error(format!(
            "{} is not available in Face /connect (use CLI `nextcode login {}`).",
            provider.display_name, provider.id
        ));
    }

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

/// Providers Face can actually authenticate (no Bedrock/Azure/AutoImport crash).
fn face_connect_wired(p: LoginProviderDescriptor) -> bool {
    match p.target {
        LoginProviderTarget::Bedrock
        | LoginProviderTarget::Azure
        | LoginProviderTarget::AutoImport => false,
        LoginProviderTarget::Claude
        | LoginProviderTarget::ClaudeApiKey
        | LoginProviderTarget::OpenAi
        | LoginProviderTarget::OpenAiApiKey
        | LoginProviderTarget::OpenRouter
        | LoginProviderTarget::OpenAiCompatible(_)
        | LoginProviderTarget::Cursor
        | LoginProviderTarget::Copilot
        | LoginProviderTarget::Gemini
        | LoginProviderTarget::Antigravity
        | LoginProviderTarget::Google => true,
    }
}

/// Flat searchable list used by OpenConnectPicker and Tab suggest.
pub(crate) fn build_connect_family_items() -> Vec<ArgItem> {
    let providers: Vec<LoginProviderDescriptor> = next_code_provider_metadata::tui_login_providers()
        .into_iter()
        .filter(|p| face_connect_wired(*p))
        .collect();
    let families = group_families(&providers);

    let mut out = Vec::new();
    let mut popular_ids: Vec<&str> = Vec::new();

    out.push(section_header("Popular"));
    for id in POPULAR_CONNECT_IDS {
        let Some(family) = families.iter().find(|f| f.iter().any(|p| p.id == *id)) else {
            continue;
        };
        let rep_id = family_representative(family).id;
        if popular_ids.contains(&rep_id) {
            continue;
        }
        popular_ids.push(rep_id);
        out.push(family_arg_item(family));
    }

    // OpenCode app twin uses "Other"; TUI uses "Providers". Screenshots show Other.
    out.push(section_header("Other"));
    for family in &families {
        let rep_id = family_representative(family).id;
        if popular_ids.contains(&rep_id) {
            continue;
        }
        out.push(family_arg_item(family));
    }
    out
}

fn connect_method_items_for_query(args_query: &str) -> Option<Vec<ArgItem>> {
    if !args_query.ends_with(char::is_whitespace) {
        return None;
    }
    let id = args_query.trim();
    if id.is_empty() {
        return None;
    }
    let provider = next_code_provider_metadata::resolve_login_provider(id)
        .or_else(|| next_code_provider_metadata::resolve_login_provider_loose(id))?;
    if !face_connect_wired(provider) {
        return None;
    }
    let methods = methods_for_key(
        &next_code_provider_metadata::tui_login_providers()
            .into_iter()
            .filter(|p| face_connect_wired(*p))
            .collect::<Vec<_>>(),
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
    // Collapse into a multi-method family only when the same auth_state_key
    // has both a browser path (OAuth/device) and a key/local path — e.g.
    // Anthropic (`claude` + `anthropic-api`). Shared OpenRouterLike API-key
    // rows stay separate (OpenCode lists each provider id).
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
    let display = opencode_display_name(rep).to_string();
    let (insert_text, provider_connect, description) = if multi {
        (
            format!("{} ", rep.id),
            false,
            opencode_row_description(rep.id)
                .map(str::to_string)
                .unwrap_or_else(|| multi_method_hint(methods)),
        )
    } else {
        (
            rep.id.to_string(),
            true,
            opencode_row_description(rep.id)
                .map(str::to_string)
                .unwrap_or_else(|| single_method_hint(rep)),
        )
    };
    ArgItem {
        display: display.clone(),
        match_text: format!(
            "{} {} {}",
            display,
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
        badge: None,
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
    fn suggest_args_has_popular_and_other() {
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
                .any(|i| i.is_section_header && i.display == "Other"),
            "{items:?}"
        );
        assert!(items.iter().any(|i| !i.is_section_header));
    }

    #[test]
    fn popular_matches_opencode_priority() {
        let items = build_connect_family_items();
        let popular: Vec<&str> = items
            .iter()
            .skip_while(|i| !(i.is_section_header && i.display == "Popular"))
            .skip(1)
            .take_while(|i| !i.is_section_header)
            .map(|i| i.insert_text.trim())
            .collect();
        assert!(
            popular.iter().any(|id| *id == "opencode" || *id == "opencode-go"),
            "expected OpenCode Zen/Go in Popular, got {popular:?}"
        );
        assert!(
            popular.iter().any(|id| *id == "openai" || id.starts_with("openai")),
            "{popular:?}"
        );
        assert!(
            !popular.iter().any(|id| *id == "bedrock"),
            "Bedrock must not be Popular"
        );
    }

    #[test]
    fn bedrock_excluded_from_picker() {
        let items = build_connect_family_items();
        assert!(
            !items.iter().any(|i| i.insert_text.trim() == "bedrock"
                || i.display.contains("Bedrock")),
            "{items:?}"
        );
    }

    #[test]
    fn typed_bedrock_errors_without_dispatch() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match ConnectCommand.run(&mut ctx, "bedrock") {
            CommandResult::Error(msg) => {
                assert!(msg.contains("not available in Face"), "{msg}");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn anthropic_family_chains_to_method_picker() {
        let family = build_connect_family_items()
            .into_iter()
            .find(|i| !i.is_section_header && (i.display == "Anthropic" || i.display.contains("Claude")))
            .expect("anthropic family");
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
    fn opencode_go_row_has_subscription_blurb() {
        let go = build_connect_family_items()
            .into_iter()
            .find(|i| i.insert_text.trim() == "opencode-go")
            .expect("opencode-go");
        assert!(
            go.description.contains("Low cost subscription"),
            "{go:?}"
        );
    }

    #[test]
    fn known_provider_dispatches_face_login() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match ConnectCommand.run(&mut ctx, "opencode-go") {
            CommandResult::Action(Action::NextCodeConnect { provider }) => {
                assert_eq!(provider, "opencode-go");
            }
            other => panic!("expected NextCodeConnect, got {other:?}"),
        }
    }
}
