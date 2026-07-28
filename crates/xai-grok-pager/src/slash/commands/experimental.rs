//! `/experimental` (alias `/experiment`) — toggle experimental feature flags.
//!
//! Opens a Codex-style checklist of `Stage::Experimental` flags. Space toggles;
//! Enter persists `[experiments]` in config.toml.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Open the experimental features checklist.
pub struct ExperimentalCommand;

impl SlashCommand for ExperimentalCommand {
    fn name(&self) -> &str {
        "experimental"
    }

    fn aliases(&self) -> &[&str] {
        &["experiment"]
    }

    fn description(&self) -> &str {
        "Toggle experimental features"
    }

    fn usage(&self) -> &str {
        "/experimental"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        CommandResult::Action(Action::OpenExperimental)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;
    use crate::slash::commands::builtin_commands;
    use crate::slash::registry::CommandRegistry;

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
    fn run_opens_experimental() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let result = ExperimentalCommand.run(&mut ctx, "");
        assert!(matches!(
            result,
            CommandResult::Action(Action::OpenExperimental)
        ));
    }

    #[test]
    fn registered_under_canonical_and_alias() {
        let reg = CommandRegistry::new(builtin_commands());
        assert!(
            reg.get("experimental").is_some(),
            "/experimental should be registered"
        );
        assert!(
            reg.get("experiment").is_some(),
            "/experiment alias should resolve"
        );
    }
}
