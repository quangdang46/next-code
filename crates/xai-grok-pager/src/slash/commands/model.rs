//! `/model` (alias `/m`) — switch model + (optionally) reasoning effort.
//!
//! Bare `/model` opens the Select-model palette (searchable list with provider
//! groups, current checkmark, Popular providers, `ctrl+a` view-all). Typed
//! `/model <name> [effort]` still works; reasoning models chain into an effort
//! sub-menu when selected from the palette.

use agent_client_protocol as acp;
use xai_grok_shell::sampling::types::supports_reasoning_effort_meta;

use crate::acp::model_state::ModelState;
use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};
use crate::slash::commands::effort_levels::build_effort_arg_items;

/// Popular provider ids shown under "Popular providers" (OpenCode-style).
/// Ids match `next_code_provider_metadata::tui_login_providers()`.
const POPULAR_PROVIDER_IDS: &[&str] = &[
    "claude",
    "openai",
    "gemini",
    "openrouter",
    "xai",
    "copilot",
];

/// Switch the active model (and optionally its reasoning effort).
pub struct ModelCommand;

impl SlashCommand for ModelCommand {
    fn name(&self) -> &str {
        "model"
    }

    fn aliases(&self) -> &[&str] {
        &["m"]
    }

    fn description(&self) -> &str {
        "Switch the active model"
    }

    fn session_scoped(&self) -> bool {
        true
    }

    fn offered_when_session_less(&self) -> bool {
        // The dashboard offers `/model` to pick the model for the next
        // spawned agent (intercepted in `dispatch_dashboard_dispatch_slash`).
        true
    }

    fn usage(&self) -> &str {
        "/model [name] [effort]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn args_required(&self) -> bool {
        // Empty args open the Select-model palette (OpenCode-style).
        false
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("<model> [effort]")
    }

    fn suggest_args(&self, ctx: &AppCtx, args_query: &str) -> Option<Vec<ArgItem>> {
        if ctx.models.is_empty() {
            // Still offer Popular providers so the user can connect.
            return Some(build_popular_provider_items(true));
        }

        // Effort phase if input is "<reasoning-model> ", else model phase.
        if let Some(model_id) = detect_effort_phase(ctx.models, args_query) {
            return Some(build_effort_items(ctx.models, &model_id));
        }
        Some(build_model_items(ctx.models))
    }

    fn run(&self, ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            return CommandResult::Action(Action::OpenModelPicker);
        }

        // Prefer an exact full-string catalog match first. Model display names
        // often contain spaces ("Grok 4.5"); if we split on the last token
        // first, a shorter catalog entry ("Grok") would steal the prefix and
        // treat "4.5" as an effort level.
        if let Some(id) = ctx.models.resolve_by_name_or_id(trimmed) {
            return CommandResult::Action(Action::SetDefaultModel(id));
        }

        // Trailing effort token + reasoning model → session-scoped switch
        // (not persisted as default). Resolve via the shared gate so a rejected
        // level (e.g. `none` on grok-4.5) surfaces the effort error with the
        // model's offered ids — not "Unknown model: … none".
        if let Some((prefix, token)) = split_trailing_token(trimmed)
            && let Some(id) = resolve_model(ctx.models, prefix)
            && ctx
                .models
                .available
                .get(&id)
                .map(supports_reasoning_effort)
                .unwrap_or(false)
        {
            return match ctx.models.resolve_effort_for_model(&id, token) {
                Ok(effort) => CommandResult::Action(Action::SwitchModel {
                    model_id: id,
                    effort: Some(effort),
                }),
                Err(err) => CommandResult::Error(err.message()),
            };
        }

        CommandResult::Error(format!("Unknown model: {trimmed}"))
    }
}

/// Look up a model by case-insensitive display name OR model id match.
fn resolve_model(models: &ModelState, name: &str) -> Option<acp::ModelId> {
    models.resolve_by_name_or_id(name)
}

fn supports_reasoning_effort(info: &acp::ModelInfo) -> bool {
    supports_reasoning_effort_meta(info.meta.as_ref())
}

/// Split `args` into `(prefix, last_token)` on the final whitespace run.
/// Returns `None` when there is no interior whitespace to split on. The token is
/// resolved to an effort against the picked model's options by the caller.
fn split_trailing_token(args: &str) -> Option<(&str, &str)> {
    let (prefix, last) = args.rsplit_once(char::is_whitespace)?;
    let prefix = prefix.trim_end();
    if prefix.is_empty() || last.is_empty() {
        return None;
    }
    Some((prefix, last))
}

/// Returns the matched model id when `args_query` is `"<reasoning-model> ..."`.
/// Longest-name-first to disambiguate names that share a prefix.
fn detect_effort_phase(models: &ModelState, args_query: &str) -> Option<acp::ModelId> {
    let mut candidates: Vec<(&acp::ModelId, &str)> = models
        .available
        .iter()
        .filter(|(_, info)| supports_reasoning_effort(info))
        .map(|(id, info)| (id, info.name.as_str()))
        .collect();
    candidates.sort_by_key(|(_, name)| std::cmp::Reverse(name.len()));

    for (id, name) in candidates {
        if args_query.len() > name.len()
            && args_query.is_char_boundary(name.len())
            && args_query[..name.len()].eq_ignore_ascii_case(name)
            && args_query[name.len()..].starts_with(char::is_whitespace)
        {
            return Some(id.clone());
        }
    }
    None
}

/// Infer a human provider label for grouping in the Select-model palette.
pub(crate) fn provider_label_for_model(id: &str, info: &acp::ModelInfo) -> String {
    if let Some(name) = info
        .meta
        .as_ref()
        .and_then(|m| m.get("providerName"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return name.to_string();
    }

    let id_l = id.to_ascii_lowercase();
    if id_l.starts_with("claude") || id_l.contains("anthropic") {
        "Anthropic".into()
    } else if id_l.starts_with("gpt-")
        || id_l.starts_with("o1")
        || id_l.starts_with("o3")
        || id_l.starts_with("o4")
        || id_l.starts_with("chatgpt")
    {
        "OpenAI".into()
    } else if id_l.starts_with("gemini") {
        "Gemini".into()
    } else if id_l.starts_with("grok") || id_l.contains("xai") {
        "xAI".into()
    } else if id_l.contains('/') || id_l.contains('@') {
        "OpenRouter".into()
    } else if id_l.starts_with("deepseek") {
        "DeepSeek".into()
    } else if id_l.starts_with("mistral") || id_l.starts_with("codestral") {
        "Mistral".into()
    } else {
        "Models".into()
    }
}

fn model_looks_free(id: &str, info: &acp::ModelInfo) -> bool {
    let id_l = id.to_ascii_lowercase();
    if id_l.contains(":free") || id_l.ends_with("-free") {
        return true;
    }
    if info.name.to_ascii_lowercase().contains(" free") {
        return true;
    }
    info.meta
        .as_ref()
        .and_then(|m| m.get("cost"))
        .and_then(|c| c.get("input"))
        .and_then(|v| v.as_f64())
        == Some(0.0)
}

fn popular_provider_rank(label: &str) -> usize {
    let key = label.to_ascii_lowercase();
    const ORDER: &[&str] = &[
        "anthropic",
        "openai",
        "gemini",
        "google",
        "openrouter",
        "xai",
        "copilot",
        "github-copilot",
        "models",
    ];
    ORDER
        .iter()
        .position(|p| key.contains(p))
        .unwrap_or(ORDER.len())
}

/// One row per logical model, grouped by provider, plus Popular providers.
/// Reasoning models get a trailing space in `insert_text` so the prompt widget
/// / palette chains into the effort sub-menu.
fn build_model_items(models: &ModelState) -> Vec<ArgItem> {
    let current_id = models.current.as_ref();
    let mut rows: Vec<(String, ArgItem)> = Vec::with_capacity(models.available.len());

    for (id, info) in &models.available {
        let is_current = current_id == Some(id);
        let supports = supports_reasoning_effort(info);
        let category = provider_label_for_model(id.0.as_ref(), info);
        let badge = if model_looks_free(id.0.as_ref(), info) {
            Some("Free".into())
        } else {
            None
        };

        // Trailing space on reasoning models: signals "more input
        // expected" so Enter advances to effort phase instead of submitting.
        let insert_text = if supports {
            format!("{} ", info.name)
        } else {
            info.name.clone()
        };

        rows.push((
            category.clone(),
            ArgItem {
                display: info.name.clone(),
                match_text: format!("{} {}", category, info.name),
                insert_text,
                description: category.clone(),
                category: Some(category),
                badge,
                is_current,
                provider_connect: false,
                is_section_header: false,
            },
        ));
    }

    rows.sort_by(|a, b| {
        popular_provider_rank(&a.0)
            .cmp(&popular_provider_rank(&b.0))
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.display.to_ascii_lowercase().cmp(&b.1.display.to_ascii_lowercase()))
    });

    let mut items: Vec<ArgItem> = Vec::new();
    let mut last_cat: Option<String> = None;
    for (cat, item) in rows {
        if last_cat.as_deref() != Some(cat.as_str()) {
            items.push(ArgItem {
                display: cat.clone(),
                match_text: String::new(),
                insert_text: String::new(),
                description: String::new(),
                category: Some(cat.clone()),
                badge: None,
                is_current: false,
                provider_connect: false,
                is_section_header: true,
            });
            last_cat = Some(cat);
        }
        items.push(item);
    }
    let popular = build_popular_provider_items(false);
    if !popular.is_empty() {
        items.push(ArgItem {
            display: "Popular providers".into(),
            match_text: String::new(),
            insert_text: String::new(),
            description: String::new(),
            category: Some("Popular providers".into()),
            badge: None,
            is_current: false,
            provider_connect: false,
            is_section_header: true,
        });
        // Strip per-row categories so we don't double-header.
        for mut row in popular {
            row.category = None;
            items.push(row);
        }
    }
    items
}

fn build_popular_provider_items(only_section: bool) -> Vec<ArgItem> {
    let providers = next_code_provider_metadata::tui_login_providers();
    let mut out = Vec::new();
    if only_section {
        out.push(ArgItem {
            display: "Popular providers".into(),
            match_text: String::new(),
            insert_text: String::new(),
            description: String::new(),
            category: Some("Popular providers".into()),
            badge: None,
            is_current: false,
            provider_connect: false,
            is_section_header: true,
        });
    }
    for id in POPULAR_PROVIDER_IDS {
        let Some(p) = providers.iter().find(|d| &d.id == id) else {
            continue;
        };
        out.push(ArgItem {
            display: p.display_name.to_string(),
            match_text: format!("Popular providers {} {}", p.display_name, p.id),
            insert_text: p.id.to_string(),
            description: format!("{} · connect", p.auth_kind.label()),
            category: if only_section {
                None
            } else {
                Some("Popular providers".into())
            },
            badge: None,
            is_current: false,
            provider_connect: true,
            is_section_header: false,
        });
        if only_section && out.iter().filter(|i| i.provider_connect).count() >= 6 {
            break;
        }
    }
    if !out.iter().any(|i| i.provider_connect) {
        for p in providers.into_iter().filter(|d| d.recommended).take(6) {
            out.push(ArgItem {
                display: p.display_name.to_string(),
                match_text: format!("Popular providers {} {}", p.display_name, p.id),
                insert_text: p.id.to_string(),
                description: format!("{} · connect", p.auth_kind.label()),
                category: None,
                badge: None,
                is_current: false,
                provider_connect: true,
                is_section_header: false,
            });
        }
    }
    out
}

/// Flat list of all login providers for `ctrl+a` "view all".
pub(crate) fn build_all_provider_connect_items() -> Vec<ArgItem> {
    let mut out = vec![ArgItem {
        display: "All providers".into(),
        match_text: String::new(),
        insert_text: String::new(),
        description: String::new(),
        category: Some("All providers".into()),
        badge: None,
        is_current: false,
        provider_connect: false,
        is_section_header: true,
    }];
    out.extend(
        next_code_provider_metadata::tui_login_providers()
            .into_iter()
            .map(|p| ArgItem {
                display: p.display_name.to_string(),
                match_text: format!("{} {}", p.display_name, p.id),
                insert_text: p.id.to_string(),
                description: format!("{} · {}", p.id, p.auth_kind.label()),
                category: None,
                badge: None,
                is_current: false,
                provider_connect: true,
                is_section_header: false,
            }),
    );
    out
}

/// One row per effort level for the `/model` chained effort phase.
/// `insert_text` is `"ModelName high"` so selecting a row completes both tokens.
fn build_effort_items(models: &ModelState, model_id: &acp::ModelId) -> Vec<ArgItem> {
    let info = match models.available.get(model_id) {
        Some(info) => info,
        None => return Vec::new(),
    };
    let model_name = info.name.clone();
    let is_current_model = models.current.as_ref() == Some(model_id);
    let options = models.reasoning_effort_options_for(model_id);
    build_effort_arg_items(
        &options,
        models.reasoning_effort,
        is_current_model,
        |option| format!("{model_name} {}", option.id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use xai_grok_shell::sampling::types::ReasoningEffort;

    fn model_with_reasoning(id: &str, name: &str) -> (acp::ModelId, acp::ModelInfo) {
        let id = acp::ModelId::new(Arc::from(id));
        let mut meta = serde_json::Map::new();
        meta.insert(
            "supportsReasoningEffort".into(),
            serde_json::Value::Bool(true),
        );
        let info = acp::ModelInfo::new(id.clone(), name.to_string())
            .meta(serde_json::Value::Object(meta).as_object().cloned());
        (id, info)
    }

    fn plain_model(id: &str, name: &str) -> (acp::ModelId, acp::ModelInfo) {
        let id = acp::ModelId::new(Arc::from(id));
        let info = acp::ModelInfo::new(id.clone(), name.to_string());
        (id, info)
    }

    static EMPTY_BUNDLE: crate::app::bundle::BundleState = crate::app::bundle::BundleState {
        has_cache: false,
        version: String::new(),
        personas: Vec::new(),
        roles: Vec::new(),
        agents: Vec::new(),
        skills: Vec::new(),
        persona_details: Vec::new(),
        role_details: Vec::new(),
    };

    fn dummy_exec_ctx(models: &ModelState) -> CommandExecCtx<'_> {
        CommandExecCtx {
            models,
            session_id: None,
            bundle_state: &EMPTY_BUNDLE,
            screen_mode: crate::app::ScreenMode::Inline,
            pager_state: crate::settings::PagerLocalSnapshot {
                multiline_mode: false,
                yolo_mode: false,
                ..crate::settings::PagerLocalSnapshot::default()
            },
        }
    }

    #[test]
    fn split_trailing_token_splits_on_final_whitespace() {
        assert_eq!(
            split_trailing_token("Reasoning X high"),
            Some(("Reasoning X", "high"))
        );
        assert_eq!(
            split_trailing_token("reasoning-x  xhigh"),
            Some(("reasoning-x", "xhigh"))
        );
        // No interior whitespace → nothing to split off.
        assert!(split_trailing_token("reasoning-x-pro").is_none());
    }

    #[test]
    fn empty_query_returns_models_plus_popular_providers() {
        let mut state = ModelState::default();
        let (rid, rinfo) = model_with_reasoning("reasoning-x", "Reasoning X");
        let (pid, pinfo) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(rid, rinfo);
        state.available.insert(pid, pinfo);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
        };
        let items = cmd.suggest_args(&ctx, "").unwrap();
        assert!(
            items.iter().any(|i| i.match_text.contains("Reasoning X")),
            "expected model rows"
        );
        assert!(
            items.iter().any(|i| i.provider_connect),
            "expected Popular providers connect rows"
        );

        let reasoning = items
            .iter()
            .find(|i| i.display == "Reasoning X")
            .unwrap();
        assert_eq!(reasoning.insert_text, "Reasoning X ");
        assert!(!reasoning.display.contains("(current)"));

        let plain = items.iter().find(|i| i.display == "Grok 4.5").unwrap();
        assert_eq!(plain.insert_text, "Grok 4.5");
    }

    #[test]
    fn bare_model_opens_picker() {
        let state = ModelState::default();
        let mut ctx = dummy_exec_ctx(&state);
        assert!(matches!(
            ModelCommand.run(&mut ctx, ""),
            CommandResult::Action(Action::OpenModelPicker)
        ));
    }

    #[test]
    fn trailing_space_after_reasoning_model_enters_effort_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
        };
        // Args query has a trailing space -> effort phase. Items come out
        // ordered xhigh -> low (strongest first) per EFFORT_LEVELS.
        let items = cmd.suggest_args(&ctx, "Reasoning X ").unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].insert_text, "Reasoning X xhigh");
        assert_eq!(items[1].insert_text, "Reasoning X high");
        assert_eq!(items[2].insert_text, "Reasoning X medium");
        assert_eq!(items[3].insert_text, "Reasoning X low");
        // Display is just the level so the user sees a clean column.
        assert_eq!(items[0].display, "xhigh");
        // match_text carries the sort-key prefix that forces the matcher's
        // alphabetical tiebreak to render rows in EFFORT_LEVELS order.
        assert!(items[0].match_text.starts_with("a "));
        assert!(items[3].match_text.starts_with("d "));
    }

    #[test]
    fn partial_effort_query_still_in_effort_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
        };
        // Still in effort phase; matcher upstream narrows to high / xhigh.
        let items = cmd.suggest_args(&ctx, "Reasoning X h").unwrap();
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn partial_model_query_stays_in_model_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
        };
        // No trailing space, user is still typing the model name.
        let items = cmd.suggest_args(&ctx, "Reason").unwrap();
        let models: Vec<_> = items
            .iter()
            .filter(|i| !i.provider_connect && !i.is_section_header)
            .collect();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].insert_text, "Reasoning X ");
    }

    #[test]
    fn run_parses_model_plus_effort_when_supported() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Reasoning X xhigh");
        match result {
            CommandResult::Action(Action::SwitchModel { model_id, effort }) => {
                assert_eq!(model_id.0.as_ref(), "reasoning-x");
                assert_eq!(effort, Some(ReasoningEffort::Xhigh));
            }
            other => panic!("expected SwitchModel with effort, got {other:?}"),
        }
    }

    #[test]
    fn run_rejects_unoffered_effort_with_effort_error_not_unknown_model() {
        // Regression: previously `resolve_effort_token_for` returned None and
        // the handler fell through to `Unknown model: Reasoning X none`.
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Reasoning X none");
        match result {
            CommandResult::Error(msg) => {
                assert!(
                    msg.contains("unknown effort level 'none'"),
                    "expected effort error, got {msg}"
                );
                assert!(
                    msg.contains("use one of:"),
                    "expected offered levels in message, got {msg}"
                );
                assert!(
                    !msg.to_lowercase().contains("unknown model"),
                    "must not misreport as unknown model: {msg}"
                );
                let offered = msg.split_once("; ").map(|(_, r)| r).unwrap_or("");
                assert!(
                    !offered.contains("none"),
                    "must not list none as offered: {msg}"
                );
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn run_prefers_full_multi_word_model_name_over_prefix_plus_effort() {
        // Catalog has both "Grok" (reasoning) and "Grok 4.5". `/model Grok 4.5`
        // must select the full name, not treat "4.5" as an effort on "Grok".
        let mut state = ModelState::default();
        let (short_id, short_info) = model_with_reasoning("grok", "Grok");
        let (long_id, long_info) = model_with_reasoning("grok-4.5", "Grok 4.5");
        state.available.insert(short_id, short_info);
        state.available.insert(long_id.clone(), long_info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, long_id);
            }
            other => panic!("expected SetDefaultModel(Grok 4.5), got {other:?}"),
        }
    }

    #[test]
    fn run_rejects_effort_for_non_reasoning_model() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5 high");
        // Falls through to "is the whole string a model name?" — which
        // it isn't, so we get an Unknown error.
        assert!(matches!(result, CommandResult::Error(_)));
    }

    /// The bare `/model <name>` form dispatches
    /// `Action::SetDefaultModel(<ModelId>)` instead of the legacy
    /// `Action::SwitchModel { effort: None }`. The dispatcher routes
    /// the typed setter through both `Effect::SwitchModel`
    /// (session-level mutation) AND `Effect::PersistSetting`
    /// (next-session default).
    ///
    /// The payload is the typed `acp::ModelId` (resolved at the slash
    /// boundary), not a String.
    #[test]
    fn run_bare_model_name_dispatches_set_default_model() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id.clone(), info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, id);
            }
            other => panic!("expected Action::SetDefaultModel(<id>), got {other:?}"),
        }
    }

    /// Case-insensitive matching against the catalog: `/model grok 4.5`
    /// resolves to the same `ModelId` as `/model Grok 4.5`.
    #[test]
    fn run_set_default_model_resolves_case_insensitively() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id.clone(), info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, id);
            }
            other => panic!("expected Action::SetDefaultModel(<id>), got {other:?}"),
        }
    }

    #[test]
    fn provider_label_uses_meta_then_heuristics() {
        let (_, mut info) = plain_model("custom-1", "Custom");
        let mut meta = serde_json::Map::new();
        meta.insert(
            "providerName".into(),
            serde_json::Value::String("Acme".into()),
        );
        info = info.meta(Some(meta));
        assert_eq!(provider_label_for_model("custom-1", &info), "Acme");
        let (_, grok) = plain_model("grok-4.5", "Grok 4.5");
        assert_eq!(provider_label_for_model("grok-4.5", &grok), "xAI");
    }
}
