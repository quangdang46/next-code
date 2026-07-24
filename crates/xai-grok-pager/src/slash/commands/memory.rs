//! `/memory` -- browse/edit typed memory + notepad (Claude-style).

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Open the typed memory browser, or toggle memory on/off.
pub struct MemoryCommand;

impl SlashCommand for MemoryCommand {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "Browse and edit typed memory / notepad"
    }

    fn usage(&self) -> &str {
        "/memory [on|off]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("[on|off]")
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        match args.trim().to_ascii_lowercase().as_str() {
            "" => CommandResult::Action(Action::OpenMemoryModal),
            // Forward enable/disable to the agent session (experimental memory gate).
            "on" | "off" => CommandResult::Action(Action::SendPrompt(format!(
                "/memory {}",
                args.trim().to_ascii_lowercase()
            ))),
            other => CommandResult::Error(format!(
                "Unknown /memory argument `{other}`. Use `/memory`, `/memory on`, or `/memory off`."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;

    static DEFAULT_BUNDLE_STATE: crate::app::bundle::BundleState =
        crate::app::bundle::BundleState {
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
    fn empty_opens_modal() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        assert!(matches!(
            MemoryCommand.run(&mut ctx, ""),
            CommandResult::Action(Action::OpenMemoryModal)
        ));
    }

    #[test]
    fn on_off_forward_prompt() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match MemoryCommand.run(&mut ctx, "off") {
            CommandResult::Action(Action::SendPrompt(text)) => {
                assert_eq!(text, "/memory off");
            }
            other => panic!("expected SendPrompt, got {other:?}"),
        }
    }

    #[test]
    fn unknown_arg_errors() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        assert!(matches!(
            MemoryCommand.run(&mut ctx, "dream"),
            CommandResult::Error(_)
        ));
    }
}
