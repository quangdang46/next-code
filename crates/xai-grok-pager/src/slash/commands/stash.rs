//! `/stash` — open the stash dialog to browse and restore previously stashed
//! prompt drafts.
//!
//! The stash is a persistent, bounded collection of prompt texts (with optional
//! images) that were automatically saved when the user typed while a turn was
//! running. `/stash` opens a picker modal showing the most recent entries first;
//! Enter restores the selected text into the composer, `d` deletes an entry.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Open the stash dialog.
pub struct StashCommand;

impl SlashCommand for StashCommand {
    fn name(&self) -> &str {
        "stash"
    }

    fn description(&self) -> &str {
        "Browse and restore stashed prompt drafts"
    }

    fn session_scoped(&self) -> bool {
        false
    }

    fn usage(&self) -> &str {
        "/stash"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        CommandResult::Action(Action::ShowStash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;
    use crate::app::actions::Action;
    use crate::app::bundle::BundleState;
    use crate::settings::PagerLocalSnapshot;
    use crate::slash::command::{CommandExecCtx, CommandResult};

    static DEFAULT_BUNDLE_STATE: BundleState = BundleState {
        has_cache: false,
        version: String::new(),
        personas: Vec::new(),
        roles: Vec::new(),
        agents: Vec::new(),
        skills: Vec::new(),
        persona_details: Vec::new(),
        role_details: Vec::new(),
    };

    #[test]
    fn stash_returns_show_stash_action() {
        let models = ModelState::default();
        let mut ctx = CommandExecCtx {
            models: &models,
            session_id: None,
            bundle_state: &DEFAULT_BUNDLE_STATE,
            screen_mode: crate::app::ScreenMode::Inline,
            pager_state: PagerLocalSnapshot::default(),
        };
        let result = StashCommand.run(&mut ctx, "");
        assert!(matches!(result, CommandResult::Action(Action::ShowStash)));
    }

    #[test]
    fn stash_is_visible_in_dropdown() {
        assert_eq!(StashCommand.name(), "stash");
        assert_eq!(StashCommand.description(), "Browse and restore stashed prompt drafts");
        assert!(!StashCommand.session_scoped());
    }
}
