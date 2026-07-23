//! `/connect` — next-code multi-provider login (Face chrome).
//!
//! OpenCode-shaped wizard (see OpenCode `dialog-provider.tsx`):
//! models.dev provider list → Popular 6 + searchable rest + Other custom →
//! optional auth-method step → Face welcome paste/URL flow → model picker.
//! Credentials write via daemon `face_auth` / `~/.next-code`.

use next_code_provider_metadata::{
    CUSTOM_PROVIDER_SENTINEL, LoginProviderAuthKind, LoginProviderAuthStateKey,
    LoginProviderDescriptor, POPULAR_MODELS_DEV_IDS, face_auth_id_for_models_dev,
    face_auth_id_needs_method_picker, is_valid_custom_provider_id, models_dev_connect_providers,
    normalize_custom_provider_id,
};

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

/// OpenCode list-row descriptions (`providerOptions` map in dialog-provider.tsx).
fn opencode_row_description(models_dev_or_face_id: &str) -> Option<&'static str> {
    match models_dev_or_face_id {
        "opencode" => Some("(Recommended)"),
        "opencode-go" => Some("Low cost subscription for everyone"),
        "openai" => Some("(ChatGPT Plus/Pro or API key)"),
        "claude" | "anthropic" => Some("(OAuth or API key)"),
        "anthropic-api" => Some("(API key)"),
        "github-copilot" | "copilot" => Some("(device code)"),
        "google" | "gemini" => Some("(OAuth or API key)"),
        CUSTOM_PROVIDER_SENTINEL => Some("Custom provider"),
        _ => None,
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

    if trimmed == CUSTOM_PROVIDER_SENTINEL {
        return CommandResult::Action(Action::NextCodeConnect {
            provider: CUSTOM_PROVIDER_SENTINEL.to_string(),
        });
    }

    let resolved = next_code_provider_metadata::resolve_login_provider(trimmed)
        .or_else(|| next_code_provider_metadata::resolve_login_provider_loose(trimmed));
    if let Some(provider) = resolved {
        return CommandResult::Action(Action::NextCodeConnect {
            provider: provider.id.to_string(),
        });
    }

    // models.dev id → Face auth id, or free-text custom id (OpenCode Other).
    let models_dev_id = models_dev_connect_providers()
        .iter()
        .find(|p| p.id.eq_ignore_ascii_case(trimmed))
        .map(|p| p.id.as_str());
    if let Some(md_id) = models_dev_id {
        let face_id = face_auth_id_for_models_dev(md_id);
        return CommandResult::Action(Action::NextCodeConnect {
            provider: face_id.to_string(),
        });
    }

    if let Some(custom) = normalize_custom_provider_id(trimmed)
        && is_valid_custom_provider_id(&custom)
    {
        return CommandResult::Action(Action::NextCodeConnect { provider: custom });
    }

    CommandResult::Error(format!(
        "Unknown provider {trimmed:?}. Try /connect and pick from the list."
    ))
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
/// OpenCode shape: Popular (6) + models.dev rest + synthetic Other (custom).
pub(crate) fn build_connect_family_items() -> Vec<ArgItem> {
    let catalog = models_dev_connect_providers();
    let mut out = Vec::with_capacity(catalog.len() + 8);
    let mut used_models_dev: Vec<&str> = Vec::new();

    out.push(section_header("Popular"));
    for md_id in POPULAR_MODELS_DEV_IDS {
        let row = catalog.iter().find(|p| p.id == *md_id);
        let (name, id) = match row {
            Some(p) => (p.name.as_str(), p.id.as_str()),
            None => continue,
        };
        used_models_dev.push(id);
        out.push(models_dev_arg_item(id, name));
    }

    // OpenCode TUI uses "Providers"; app twin / screenshots use "Other".
    out.push(section_header("Other"));
    let mut rest: Vec<&next_code_provider_metadata::ModelsDevProvider> = catalog
        .iter()
        .filter(|p| !used_models_dev.contains(&p.id.as_str()))
        .collect();
    rest.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    for p in rest {
        out.push(models_dev_arg_item(&p.id, &p.name));
    }

    // Synthetic Other = custom provider id (OpenCode CUSTOM_PROVIDER_OPTION_VALUE).
    out.push(ArgItem {
        display: "Other".into(),
        match_text: "other custom provider".into(),
        insert_text: CUSTOM_PROVIDER_SENTINEL.to_string(),
        description: "Custom provider".into(),
        category: None,
        badge: None,
        is_current: false,
        provider_connect: true,
        is_section_header: false,
    });

    out
}

fn models_dev_arg_item(models_dev_id: &str, name: &str) -> ArgItem {
    let face_id = face_auth_id_for_models_dev(models_dev_id);
    let description = opencode_row_description(models_dev_id)
        .or_else(|| opencode_row_description(face_id))
        .unwrap_or("")
        .to_string();
    let multi = face_auth_id_needs_method_picker(face_id);
    let (insert_text, provider_connect) = if multi {
        (format!("{face_id} "), false)
    } else {
        (face_id.to_string(), true)
    };
    ArgItem {
        display: name.to_string(),
        match_text: format!("{name} {models_dev_id} {face_id} {description}"),
        insert_text,
        description,
        category: None,
        badge: None,
        is_current: false,
        provider_connect,
        is_section_header: false,
    }
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
    let catalog = next_code_provider_metadata::tui_login_providers();
    let mut methods = methods_for_key(&catalog, provider.auth_state_key);
    // OpenCode lists Google as OAuth + API key; next-code stores Gemini API under
    // OpenRouterLike, so merge explicitly for the Face method step.
    if matches!(id, "gemini" | "google") {
        for extra in &catalog {
            if extra.id == "gemini-api" && !methods.iter().any(|m| m.id == extra.id) {
                methods.push(*extra);
            }
        }
    }
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
    fn suggest_args_has_popular_other_and_custom() {
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
        assert!(
            items
                .iter()
                .any(|i| i.insert_text == CUSTOM_PROVIDER_SENTINEL && i.display == "Other"),
            "missing synthetic Other custom row"
        );
        let selectable = items.iter().filter(|i| !i.is_section_header).count();
        assert!(
            selectable >= 150,
            "expected OpenCode-scale list, got {selectable}"
        );
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
        assert_eq!(
            popular,
            vec!["opencode", "opencode-go", "openai", "copilot", "claude", "gemini"]
        );
    }

    #[test]
    fn long_tail_providers_listed() {
        let items = build_connect_family_items();
        for needle in ["cohere", "venice", "poe", "siliconflow", "databricks"] {
            assert!(
                items.iter().any(|i| i.insert_text.trim() == needle || i.match_text.contains(needle)),
                "missing {needle}"
            );
        }
    }

    #[test]
    fn bedrock_listed_under_other() {
        let items = build_connect_family_items();
        let bedrock = items
            .iter()
            .find(|i| i.insert_text.trim() == "bedrock")
            .expect("bedrock in Other");
        assert!(!bedrock.is_section_header);
        assert_eq!(bedrock.display, "Amazon Bedrock");
    }

    #[test]
    fn typed_bedrock_dispatches_face_login() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match ConnectCommand.run(&mut ctx, "bedrock") {
            CommandResult::Action(Action::NextCodeConnect { provider }) => {
                assert_eq!(provider, "bedrock");
            }
            other => panic!("expected NextCodeConnect bedrock, got {other:?}"),
        }
        match ConnectCommand.run(&mut ctx, "amazon-bedrock") {
            CommandResult::Action(Action::NextCodeConnect { provider }) => {
                assert_eq!(provider, "bedrock");
            }
            other => panic!("expected NextCodeConnect bedrock via models.dev id, got {other:?}"),
        }
    }

    #[test]
    fn anthropic_family_chains_to_method_picker() {
        let family = build_connect_family_items()
            .into_iter()
            .find(|i| !i.is_section_header && i.display == "Anthropic")
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

    #[test]
    fn long_tail_models_dev_id_dispatches() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match ConnectCommand.run(&mut ctx, "cohere") {
            CommandResult::Action(Action::NextCodeConnect { provider }) => {
                assert_eq!(provider, "cohere");
            }
            other => panic!("expected NextCodeConnect cohere, got {other:?}"),
        }
    }

    #[test]
    fn custom_sentinel_dispatches() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match ConnectCommand.run(&mut ctx, CUSTOM_PROVIDER_SENTINEL) {
            CommandResult::Action(Action::NextCodeConnect { provider }) => {
                assert_eq!(provider, CUSTOM_PROVIDER_SENTINEL);
            }
            other => panic!("expected custom sentinel, got {other:?}"),
        }
    }
}
