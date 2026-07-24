//! Session-goal tools (`create_goal` / `update_goal` / `get_goal`).
//! Distinct from durable `initiative`.

use super::{Tool, ToolContext, ToolOutput};
use crate::bus::{Bus, BusEvent};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};

fn goal_enabled() -> bool {
    crate::config::config().goal.enabled
}

fn disabled_output() -> ToolOutput {
    ToolOutput::new(
        "Session goals are disabled. Enable them in config (`[goal] enabled = true`)."
            .to_string(),
    )
}

fn publish_updated(
    session_id: &str,
    goal: Option<crate::session_goal::SessionGoal>,
    last_event: Option<&str>,
) {
    Bus::global().publish(BusEvent::SessionGoalUpdated {
        session_id: session_id.to_string(),
        goal,
        last_event: last_event.map(str::to_string),
    });
}

fn format_goal_json(goal: Option<&crate::session_goal::SessionGoal>) -> String {
    serde_json::to_string_pretty(&json!({ "goal": goal })).unwrap_or_else(|_| {
        json!({ "goal": null }).to_string()
    })
}

fn resolve_session_id(explicit: Option<&str>, ctx: &ToolContext) -> Result<String> {
    if let Some(sid) = explicit.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(sid.to_string());
    }
    let sid = ctx.session_id.trim();
    if sid.is_empty() {
        anyhow::bail!("no session_id available");
    }
    Ok(sid.to_string())
}

// --- create_goal -----------------------------------------------------------

pub struct CreateGoalTool;

impl CreateGoalTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct CreateGoalInput {
    objective: String,
    #[serde(default)]
    session_id: Option<String>,
}

#[async_trait]
impl Tool for CreateGoalTool {
    fn name(&self) -> &str {
        "create_goal"
    }

    fn description(&self) -> &str {
        "Create or replace the active goal for the current session. \
         Use when the user asks to set a goal, or when a new high-level objective is needed. \
         The goal persists across turns and is shown in Face chrome. \
         The objective should be concise (under 2000 characters) and describe the desired outcome."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["objective"],
            "properties": {
                "intent": super::intent_schema_property(),
                "objective": {
                    "type": "string",
                    "description": "Concise outcome the session should achieve"
                },
                "session_id": {
                    "type": "string",
                    "description": "Session ID to target (default: current session)"
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        if !goal_enabled() {
            return Ok(disabled_output());
        }
        let params: CreateGoalInput = serde_json::from_value(input)?;
        let session_id = resolve_session_id(params.session_id.as_deref(), &ctx)?;
        let goal = crate::session_goal::set(&session_id, &params.objective)?;
        publish_updated(&session_id, Some(goal.clone()), Some("goal_set"));
        Ok(ToolOutput::new(format_goal_json(Some(&goal))))
    }
}

// --- update_goal -----------------------------------------------------------

pub struct UpdateGoalTool;

impl UpdateGoalTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct UpdateGoalInput {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    objective: Option<String>,
    #[serde(default)]
    session_id: Option<String>,
}

#[async_trait]
impl Tool for UpdateGoalTool {
    fn name(&self) -> &str {
        "update_goal"
    }

    fn description(&self) -> &str {
        "Update the active goal for the current session. \
         Use to pause, resume, complete, or change the objective. \
         When the goal is achieved, call update_goal with status \"complete\"."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": super::intent_schema_property(),
                "status": {
                    "type": "string",
                    "enum": ["active", "paused", "complete"],
                    "description": "New goal status"
                },
                "objective": {
                    "type": "string",
                    "description": "New objective text"
                },
                "session_id": {
                    "type": "string",
                    "description": "Session ID to target (default: current session)"
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        if !goal_enabled() {
            return Ok(disabled_output());
        }
        let params: UpdateGoalInput = serde_json::from_value(input)?;
        let session_id = resolve_session_id(params.session_id.as_deref(), &ctx)?;

        let mut last_event = "goal_updated";
        if let Some(objective) = params.objective.as_deref() {
            crate::session_goal::set(&session_id, objective)?;
            last_event = "goal_set";
        }
        if let Some(status) = params.status.as_deref() {
            match crate::session_goal::SessionGoalStatus::parse(status) {
                Some(crate::session_goal::SessionGoalStatus::Paused) => {
                    crate::session_goal::pause(&session_id)?;
                    last_event = "goal_paused";
                }
                Some(crate::session_goal::SessionGoalStatus::Active) => {
                    crate::session_goal::resume(&session_id)?;
                    last_event = "goal_resumed";
                }
                Some(crate::session_goal::SessionGoalStatus::Complete) => {
                    crate::session_goal::mark_complete(&session_id)?;
                    last_event = "goal_complete";
                }
                None => anyhow::bail!("invalid status: {status}"),
            }
        }

        let goal = crate::session_goal::get(&session_id)?;
        publish_updated(&session_id, goal.clone(), Some(last_event));
        Ok(ToolOutput::new(format_goal_json(goal.as_ref())))
    }
}

// --- get_goal --------------------------------------------------------------

pub struct GetGoalTool;

impl GetGoalTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct GetGoalInput {
    #[serde(default)]
    session_id: Option<String>,
}

#[async_trait]
impl Tool for GetGoalTool {
    fn name(&self) -> &str {
        "get_goal"
    }

    fn description(&self) -> &str {
        "Read the active goal for the current session. \
         Returns the current objective, status, and usage accounting. \
         Returns null if no goal is active."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": super::intent_schema_property(),
                "session_id": {
                    "type": "string",
                    "description": "Session ID to target (default: current session)"
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        if !goal_enabled() {
            return Ok(disabled_output());
        }
        let params: GetGoalInput = serde_json::from_value(input).unwrap_or(GetGoalInput {
            session_id: None,
        });
        let session_id = resolve_session_id(params.session_id.as_deref(), &ctx)?;
        let goal = crate::session_goal::get(&session_id)?;
        Ok(ToolOutput::new(format_goal_json(goal.as_ref())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, timeout};

    fn test_ctx(session_id: &str) -> ToolContext {
        ToolContext {
            session_id: session_id.to_string(),
            message_id: "msg1".to_string(),
            tool_call_id: "tool1".to_string(),
            working_dir: None,
            stdin_request_tx: None,
            ask_user_question_tx: None,
            graceful_shutdown_signal: None,
            execution_mode: crate::tool::ToolExecutionMode::AgentTurn,
            best_of_n_run_id: None,
            best_of_n_candidate_id: None,
        }
    }

    #[tokio::test]
    async fn create_update_get_complete_cycle() {
        let _guard = crate::storage::lock_test_env();
        let temp = tempfile::tempdir().expect("tempdir");
        let prev = std::env::var_os("NEXT_CODE_HOME");
        crate::env::set_var("NEXT_CODE_HOME", temp.path());

        let mut bus_rx = Bus::global().subscribe();
        let ctx = test_ctx("ses_sg_tools");

        let create = CreateGoalTool::new()
            .execute(
                json!({ "objective": "  Ship feature  " }),
                ctx.clone(),
            )
            .await
            .expect("create");
        assert!(create.output.contains("Ship feature"));
        assert!(create.output.contains("\"status\": \"active\"") || create.output.contains("\"status\":\"active\""));

        let event = timeout(Duration::from_millis(500), bus_rx.recv())
            .await
            .expect("bus event")
            .expect("recv");
        assert!(matches!(
            event,
            BusEvent::SessionGoalUpdated {
                session_id,
                goal: Some(_),
                ..
            } if session_id == "ses_sg_tools"
        ));

        let get = GetGoalTool::new()
            .execute(json!({}), ctx.clone())
            .await
            .expect("get");
        assert!(get.output.contains("Ship feature"));

        let complete = UpdateGoalTool::new()
            .execute(json!({ "status": "complete" }), ctx)
            .await
            .expect("complete");
        assert!(
            complete.output.contains("complete"),
            "got {}",
            complete.output
        );

        match prev {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
    }
}
