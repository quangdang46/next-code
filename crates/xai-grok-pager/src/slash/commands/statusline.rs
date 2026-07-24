//! `/statusline` — configure Face prompt statusline segments.
//!
//! Bare `/statusline` opens Settings. Subcommands adjust `[ui.status_line]`
//! live (same persist path as the settings modal).

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};
use xai_grok_shell::agent::config::{StatusLineConfig, StatusLineSegment};

/// Configure prompt statusline chrome.
pub struct StatuslineCommand;

impl SlashCommand for StatuslineCommand {
    fn name(&self) -> &str {
        "statusline"
    }

    fn aliases(&self) -> &[&str] {
        &["status-line", "prompt-status"]
    }

    fn description(&self) -> &str {
        "Configure prompt status line (mode, model, context%, cwd, git)"
    }

    fn usage(&self) -> &str {
        "/statusline [on|off|reset|order <csv>|toggle <segment>|settings]"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            return CommandResult::Action(Action::OpenSettings);
        }

        let mut parts = trimmed.split_whitespace();
        let sub = parts.next().unwrap_or("").to_ascii_lowercase();
        match sub.as_str() {
            "on" | "enable" => CommandResult::Action(Action::SetStatusLineEnabled(true)),
            "off" | "disable" => CommandResult::Action(Action::SetStatusLineEnabled(false)),
            "reset" => CommandResult::Action(Action::ResetStatusLine),
            "settings" | "config" | "prefs" => CommandResult::Action(Action::OpenSettings),
            "order" => {
                let rest = parts.collect::<Vec<_>>().join(" ");
                if rest.is_empty() {
                    return CommandResult::Message(
                        "Usage: /statusline order mode,model,context".into(),
                    );
                }
                let csv = StatusLineConfig::canonicalize_order(&rest);
                CommandResult::Action(Action::SetStatusLineOrder(csv))
            }
            "toggle" => {
                let seg_raw = parts.next().unwrap_or("");
                if seg_raw.is_empty() {
                    return CommandResult::Message(
                        "Usage: /statusline toggle <segment> (mode|model|context|cwd|git)".into(),
                    );
                }
                match StatusLineSegment::parse(seg_raw) {
                    Some(seg) => CommandResult::Action(Action::ToggleStatusLineSegment(seg)),
                    None => CommandResult::Message(format!(
                        "Unknown segment '{seg_raw}'. Try mode, model, context, cwd, or git."
                    )),
                }
            }
            _ => CommandResult::Message(
                "Usage: /statusline [on|off|reset|order <csv>|toggle <segment>|settings]".into(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;
    use crate::app::bundle::BundleState;
    use crate::settings::PagerLocalSnapshot;

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

    fn make_ctx<'a>(models: &'a ModelState) -> CommandExecCtx<'a> {
        CommandExecCtx {
            models,
            session_id: None,
            bundle_state: &DEFAULT_BUNDLE_STATE,
            screen_mode: crate::app::ScreenMode::Inline,
            pager_state: PagerLocalSnapshot {
                multiline_mode: false,
                yolo_mode: false,
                ..PagerLocalSnapshot::default()
            },
        }
    }

    #[test]
    fn empty_args_open_settings() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let result = StatuslineCommand.run(&mut ctx, "");
        assert!(matches!(result, CommandResult::Action(Action::OpenSettings)));
    }

    #[test]
    fn on_off_and_reset_dispatch() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        let cmd = StatuslineCommand;

        assert!(matches!(
            cmd.run(&mut ctx, "on"),
            CommandResult::Action(Action::SetStatusLineEnabled(true))
        ));
        assert!(matches!(
            cmd.run(&mut ctx, "off"),
            CommandResult::Action(Action::SetStatusLineEnabled(false))
        ));
        assert!(matches!(
            cmd.run(&mut ctx, "reset"),
            CommandResult::Action(Action::ResetStatusLine)
        ));
    }

    #[test]
    fn order_canonicalizes() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        match StatuslineCommand.run(&mut ctx, "order context, model") {
            CommandResult::Action(Action::SetStatusLineOrder(csv)) => {
                assert!(csv.starts_with("context,model"));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn toggle_segment() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        assert!(matches!(
            StatuslineCommand.run(&mut ctx, "toggle cwd"),
            CommandResult::Action(Action::ToggleStatusLineSegment(StatusLineSegment::Cwd))
        ));
    }

    #[test]
    fn unknown_sub_shows_usage() {
        let models = ModelState::default();
        let mut ctx = make_ctx(&models);
        assert!(matches!(
            StatuslineCommand.run(&mut ctx, "nope"),
            CommandResult::Message(_)
        ));
    }
}
