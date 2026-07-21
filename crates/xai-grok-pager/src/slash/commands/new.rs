//! `/new` (alias `/clear`) -- create a new session.
//!
//! **Alias hazard (PR10 / Face vs next-code TUI):** Face `/clear` is an alias of
//! `/new` (fresh session). Legacy next-code TUI `/clear` meant clear-history in
//! the current session. Embed users who type `/clear` get Face semantics
//! (new session). Prefer `/new` in docs; do not silently restore TUI clear-history.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Start a new agent session, clearing the current conversation.
pub struct NewCommand;

impl SlashCommand for NewCommand {
    fn name(&self) -> &str {
        "new"
    }

    fn aliases(&self) -> &[&str] {
        &["clear"]
    }

    fn description(&self) -> &str {
        "Start a new session"
    }

    fn usage(&self) -> &str {
        "/new"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        CommandResult::Action(Action::NewSession)
    }
}
