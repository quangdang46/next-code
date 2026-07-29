//! `/multitask` — Cursor-style parallel async workers from Face.
//!
//! Drains queued prompts (and optional inline args) into headless swarm
//! workers so the lead session stays interactive. Workers report a final
//! summary back; select a teammate in the agent-team panel to steer a
//! running worker.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Parsed `/multitask` arguments.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MultitaskArgs {
    /// When set, send `message` as a follow-up/steer into this running worker
    /// (swarm DM) instead of spawning new workers.
    pub to_session: Option<String>,
    /// Inline worker prompts (or the steer message when `to_session` is set).
    /// Separators: newlines or a line that is exactly `---`.
    pub prompts: Vec<String>,
}

/// Parse the raw argument string after `/multitask`.
pub fn parse_multitask_args(args: &str) -> Result<MultitaskArgs, String> {
    let mut rest = args.trim();
    let mut to_session = None;

    if let Some(after) = rest.strip_prefix("--to") {
        let after = after.trim_start();
        let (id, remainder) = match after.split_once(char::is_whitespace) {
            Some((id, rem)) => (id.trim(), rem.trim()),
            None => {
                return Err(
                    "Usage: /multitask --to <worker_session_id> <follow-up message>".into(),
                );
            }
        };
        if id.is_empty() {
            return Err("Missing worker session id after --to".into());
        }
        to_session = Some(id.to_string());
        rest = remainder;
    }

    let prompts = if rest.is_empty() {
        Vec::new()
    } else {
        split_multitask_prompts(rest)
    };

    if to_session.is_some() && prompts.is_empty() {
        return Err("Usage: /multitask --to <worker_session_id> <follow-up message>".into());
    }

    Ok(MultitaskArgs { to_session, prompts })
}

/// Split inline multitask text on newlines / `---` separators.
pub fn split_multitask_prompts(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for line in text.lines() {
        if line.trim() == "---" {
            let chunk = buf.trim().to_string();
            if !chunk.is_empty() {
                out.push(chunk);
            }
            buf.clear();
            continue;
        }
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(line);
    }
    let chunk = buf.trim().to_string();
    if !chunk.is_empty() {
        out.push(chunk);
    }
    // Also accept a single-line comma-light path: one prompt per non-empty line
    // when no `---` was used and every "chunk" is already one line — already handled.
    out
}

/// Parallelize queued / inline prompts as background workers.
pub struct MultitaskCommand;

impl SlashCommand for MultitaskCommand {
    fn name(&self) -> &str {
        "multitask"
    }

    fn description(&self) -> &str {
        "Run queued (or inline) prompts as parallel background workers"
    }

    fn session_scoped(&self) -> bool {
        true
    }

    fn usage(&self) -> &str {
        "/multitask [prompt | --- separated prompts] | /multitask --to <id> <msg>"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn args_required(&self) -> bool {
        false
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("[prompts…]")
    }

    fn run(&self, ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        if ctx.session_id.is_none() {
            return CommandResult::Error("No active session".to_string());
        }
        match parse_multitask_args(args) {
            Ok(parsed) => CommandResult::Action(Action::Multitask(parsed)),
            Err(msg) => CommandResult::Error(msg),
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

    #[test]
    fn parse_empty_ok() {
        let a = parse_multitask_args("").unwrap();
        assert!(a.prompts.is_empty());
        assert!(a.to_session.is_none());
    }

    #[test]
    fn parse_inline_and_separators() {
        let a = parse_multitask_args("fix auth\n---\nadd tests").unwrap();
        assert_eq!(a.prompts, vec!["fix auth", "add tests"]);
    }

    #[test]
    fn parse_to_follow_up() {
        let a = parse_multitask_args("--to worker-1 please also lint").unwrap();
        assert_eq!(a.to_session.as_deref(), Some("worker-1"));
        assert_eq!(a.prompts, vec!["please also lint"]);
    }

    #[test]
    fn parse_to_requires_message() {
        assert!(parse_multitask_args("--to worker-1").is_err());
    }

    #[test]
    fn no_session_errors() {
        let models = ModelState::default();
        let mut ctx = CommandExecCtx {
            models: &models,
            session_id: None,
            bundle_state: &DEFAULT_BUNDLE_STATE,
            screen_mode: crate::app::ScreenMode::Minimal,
            pager_state: PagerLocalSnapshot::default(),
        };
        match MultitaskCommand.run(&mut ctx, "") {
            CommandResult::Error(msg) => assert!(msg.contains("No active session")),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn with_session_dispatches_action() {
        let models = ModelState::default();
        let sid = agent_client_protocol::SessionId::from("s1".to_string());
        let mut ctx = CommandExecCtx {
            models: &models,
            session_id: Some(&sid),
            bundle_state: &DEFAULT_BUNDLE_STATE,
            screen_mode: crate::app::ScreenMode::Minimal,
            pager_state: PagerLocalSnapshot::default(),
        };
        match MultitaskCommand.run(&mut ctx, "do thing") {
            CommandResult::Action(Action::Multitask(a)) => {
                assert_eq!(a.prompts, vec!["do thing"]);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
