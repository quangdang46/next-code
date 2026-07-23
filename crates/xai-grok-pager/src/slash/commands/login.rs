//! `/login` — authenticate.
//!
//! In the nextcode embed: alias of `/connect` (multi-provider Face picker),
//! **not** Grok OAuth. Stock grok-bin still uses `Action::Login`.

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

pub struct LoginCommand;

impl SlashCommand for LoginCommand {
    fn name(&self) -> &str {
        "login"
    }

    fn description(&self) -> &str {
        if crate::product_welcome::is_nextcode_embed() {
            "Connect a model provider (alias of /connect)"
        } else {
            "Log in or re-authenticate with your account"
        }
    }

    fn usage(&self) -> &str {
        if crate::product_welcome::is_nextcode_embed() {
            "/login [provider]"
        } else {
            "/login"
        }
    }

    fn takes_args(&self) -> bool {
        crate::product_welcome::is_nextcode_embed()
    }

    fn arg_placeholder(&self) -> Option<&str> {
        // Embed `/login` is picker-first (same as `/connect`); no inline ghost.
        None
    }

    fn suggest_args(&self, ctx: &AppCtx, args_query: &str) -> Option<Vec<ArgItem>> {
        if crate::product_welcome::is_nextcode_embed() {
            super::connect::ConnectCommand.suggest_args(ctx, args_query)
        } else {
            None
        }
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        if crate::product_welcome::is_nextcode_embed() {
            return super::connect::connect_run(args);
        }
        CommandResult::Action(Action::Login)
    }
}
