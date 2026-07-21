//! `/connect` — next-code multi-provider login (Face chrome).
//!
//! Uses Face `suggest_args` dropdown with `tui_login_providers()`, then starts
//! the Face welcome auth paste/URL flow (credential write to `~/.next-code`).
//! Does **not** start Grok OAuth and does **not** hand off to a CLI terminal.

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

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
        Some("provider")
    }

    fn suggest_args(&self, _ctx: &AppCtx, _args_query: &str) -> Option<Vec<ArgItem>> {
        Some(provider_arg_items())
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        connect_run(args)
    }
}

/// Shared by `/connect` and embed `/login`.
pub(crate) fn connect_run(args: &str) -> CommandResult {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        let list = provider_arg_items()
            .into_iter()
            .map(|i| format!("  {} — {}", i.insert_text, i.description))
            .collect::<Vec<_>>()
            .join("\n");
        return CommandResult::Message(format!(
            "Connect a provider (Face picker: Tab after /connect ).\n\
             \n\
             {list}\n\
             \n\
             Then run: /connect <provider>\n\
             Face opens the auth URL / paste box — credentials save under ~/.next-code \
             (not Grok OAuth)."
        ));
    }

    let Some(provider) = next_code_provider_metadata::resolve_login_provider(trimmed).or_else(
        || next_code_provider_metadata::resolve_login_provider_loose(trimmed),
    ) else {
        return CommandResult::Error(format!(
            "Unknown provider {trimmed:?}. Try /connect and pick from the dropdown."
        ));
    };

    CommandResult::Action(Action::NextCodeConnect {
        provider: provider.id.to_string(),
    })
}

fn provider_arg_items() -> Vec<ArgItem> {
    next_code_provider_metadata::tui_login_providers()
        .into_iter()
        .map(|p| ArgItem {
            display: p.id.to_string(),
            match_text: p.id.to_string(),
            insert_text: p.id.to_string(),
            description: format!("{} · {}", p.display_name, p.auth_kind.label()),
        })
        .collect()
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
    fn bare_connect_lists_providers() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match ConnectCommand.run(&mut ctx, "") {
            CommandResult::Message(msg) => {
                assert!(msg.contains("/connect <provider>"), "{msg}");
                assert!(!msg.contains("next-code login"), "{msg}");
                assert!(msg.contains("not Grok OAuth"), "{msg}");
            }
            other => panic!("expected Message, got {other:?}"),
        }
    }

    #[test]
    fn suggest_args_non_empty() {
        let models = ModelState::default();
        let cwd = std::path::Path::new(".");
        let ctx = AppCtx {
            models: &models,
            cwd,
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
        };
        let items = ConnectCommand.suggest_args(&ctx, "").expect("items");
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| !i.insert_text.is_empty()));
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
