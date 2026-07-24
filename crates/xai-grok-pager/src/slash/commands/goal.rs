//! `/goal` — persistent session objective (oh-my-openagent parity).
//!
//! Argument grammar matches OMO `parseGoalCommand`:
//! - `` `/goal` `` / `` `/goal status` `` → show
//! - `` `/goal pause` `` / `` `/goal resume` `` / `` `/goal clear` ``
//! - anything else → set objective and start pursuing it

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

/// Parsed `/goal` arguments — mirrors OMO `ParsedGoalCommand`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedGoalCommand {
    Show,
    Clear,
    Pause,
    Resume,
    SetObjective(String),
}

/// Parse raw `/goal` args with OMO semantics.
pub fn parse_goal_command(raw_args: &str) -> ParsedGoalCommand {
    let trimmed = raw_args.trim();
    if trimmed.is_empty() {
        return ParsedGoalCommand::Show;
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "pause" => ParsedGoalCommand::Pause,
        "resume" => ParsedGoalCommand::Resume,
        "clear" => ParsedGoalCommand::Clear,
        "status" => ParsedGoalCommand::Show,
        _ => ParsedGoalCommand::SetObjective(trimmed.to_string()),
    }
}

pub struct GoalCommand;

impl SlashCommand for GoalCommand {
    fn name(&self) -> &str {
        "goal"
    }

    fn aliases(&self) -> &[&str] {
        &["mission"]
    }

    fn description(&self) -> &str {
        "Set or manage a persistent session goal"
    }

    fn session_scoped(&self) -> bool {
        true
    }

    fn usage(&self) -> &str {
        "/goal [pause|resume|clear|status|<objective>]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("[pause|resume|clear|status|<objective>]")
    }

    fn suggest_args(&self, _ctx: &AppCtx, _args_query: &str) -> Option<Vec<ArgItem>> {
        Some(vec![
            ArgItem {
                display: "status".into(),
                match_text: "status".into(),
                insert_text: "status".into(),
                description: "Show the active goal".into(),
                ..Default::default()
            },
            ArgItem {
                display: "pause".into(),
                match_text: "pause".into(),
                insert_text: "pause".into(),
                description: "Pause goal pursuit".into(),
                ..Default::default()
            },
            ArgItem {
                display: "resume".into(),
                match_text: "resume".into(),
                insert_text: "resume".into(),
                description: "Resume a paused goal".into(),
                ..Default::default()
            },
            ArgItem {
                display: "clear".into(),
                match_text: "clear".into(),
                insert_text: "clear".into(),
                description: "Clear the active goal".into(),
                ..Default::default()
            },
        ])
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        match parse_goal_command(args) {
            ParsedGoalCommand::Show => CommandResult::Action(Action::GoalShow),
            ParsedGoalCommand::Pause => CommandResult::Action(Action::GoalPause),
            ParsedGoalCommand::Resume => CommandResult::Action(Action::GoalResume),
            ParsedGoalCommand::Clear => CommandResult::Action(Action::GoalClear),
            ParsedGoalCommand::SetObjective(objective) => {
                CommandResult::Action(Action::GoalSet { objective })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_and_status_are_show() {
        assert_eq!(parse_goal_command(""), ParsedGoalCommand::Show);
        assert_eq!(parse_goal_command("  "), ParsedGoalCommand::Show);
        assert_eq!(parse_goal_command("status"), ParsedGoalCommand::Show);
        assert_eq!(parse_goal_command("STATUS"), ParsedGoalCommand::Show);
    }

    #[test]
    fn parse_control_verbs() {
        assert_eq!(parse_goal_command("pause"), ParsedGoalCommand::Pause);
        assert_eq!(parse_goal_command("Resume"), ParsedGoalCommand::Resume);
        assert_eq!(parse_goal_command("clear"), ParsedGoalCommand::Clear);
    }

    #[test]
    fn parse_objective_is_raw_text() {
        assert_eq!(
            parse_goal_command("ship the /goal slash"),
            ParsedGoalCommand::SetObjective("ship the /goal slash".into())
        );
    }

    #[test]
    fn run_dispatches_actions() {
        let cmd = GoalCommand;
        let models = crate::acp::model_state::ModelState::default();
        let bundle = crate::app::bundle::BundleState::default();
        let mut ctx = CommandExecCtx {
            models: &models,
            session_id: None,
            bundle_state: &bundle,
            screen_mode: crate::app::ScreenMode::Inline,
            pager_state: crate::settings::PagerLocalSnapshot::default(),
        };
        assert!(matches!(
            cmd.run(&mut ctx, ""),
            CommandResult::Action(Action::GoalShow)
        ));
        assert!(matches!(
            cmd.run(&mut ctx, "pause"),
            CommandResult::Action(Action::GoalPause)
        ));
        assert!(matches!(
            cmd.run(&mut ctx, "fix auth"),
            CommandResult::Action(Action::GoalSet { objective }) if objective == "fix auth"
        ));
    }
}
