//! `/keybindings` — create/open `~/.next-code/keybindings.json` (Claude-style remap).

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Open or reload Face user keybindings.
pub struct KeybindingsCommand;

impl SlashCommand for KeybindingsCommand {
    fn name(&self) -> &str {
        "keybindings"
    }

    fn aliases(&self) -> &[&str] {
        &["keys-file", "remap-keys"]
    }

    fn description(&self) -> &str {
        "Edit Face keybindings file (~/.next-code/keybindings.json)"
    }

    fn usage(&self) -> &str {
        "/keybindings [reload]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let trimmed = args.trim();
        if trimmed.eq_ignore_ascii_case("reload") || trimmed.eq_ignore_ascii_case("--reload") {
            return CommandResult::Action(Action::ReloadKeybindings);
        }

        match crate::actions::ensure_keybindings_file() {
            Ok((path, _created)) => CommandResult::Action(Action::SuspendForEditor {
                path,
                refresh_agents_modal: None,
                reload_keybindings: true,
            }),
            Err(e) => CommandResult::Message(format!(
                "Could not create/open {}: {e}",
                crate::actions::keybindings_display_path()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;

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
    fn reload_arg_dispatches_reload() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let result = KeybindingsCommand.run(&mut ctx, "reload");
        assert!(matches!(result, CommandResult::Action(Action::ReloadKeybindings)));
    }

    #[test]
    fn bare_command_name() {
        assert_eq!(KeybindingsCommand.name(), "keybindings");
    }
}
