//! ExitPlanMode — present the plan for user approve / revise / abandon via Face ACP.

use super::{ExitPlanModeInputRequest, Tool, ToolContext, ToolOutput};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use xai_grok_tools::implementations::grok_build::exit_plan_mode::ExitPlanModeExtResponse;

const CANCEL_STAY_IN_PLAN: &str = concat!(
    "The user cancelled exiting plan mode and wants to keep refining the plan. ",
    "Stay in plan mode, incorporate any feedback, and call ExitPlanMode again when ready."
);

const APPROVED_MSG: &str = concat!(
    "The user approved the plan. You have left plan mode and may now make ",
    "mutating changes. Follow the approved plan as the contract for implementation."
);

const ABANDONED_MSG: &str = concat!(
    "The user abandoned the plan without approving it. You have left plan mode. ",
    "Do not implement the abandoned plan; wait for new instructions."
);

pub struct ExitPlanModeTool;

impl ExitPlanModeTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExitPlanModeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct ExitPlanModeInput {
    #[serde(default)]
    plan_content: Option<String>,
}

#[async_trait]
impl Tool for ExitPlanModeTool {
    fn name(&self) -> &str {
        "ExitPlanMode"
    }

    fn description(&self) -> &str {
        r#"Exit plan mode and ask the user to approve, revise, or abandon the plan.

Call this after writing plan.md when the plan is ready for review. Mutating tools
remain blocked until the user approves."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": super::intent_schema_property(),
                "plan_content": {
                    "type": "string",
                    "description": "Optional plan body to show in the approval UI. Prefer the on-disk plan.md when omitted."
                }
            }
        })
    }

    fn declared_tier(&self) -> Option<next_code_tool_types::ToolTier> {
        Some(next_code_tool_types::ToolTier::Read)
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: ExitPlanModeInput = serde_json::from_value(input).unwrap_or(ExitPlanModeInput {
            plan_content: None,
        });

        let plan_content = params
            .plan_content
            .filter(|s| !s.trim().is_empty())
            .or_else(|| read_plan_md(&ctx));

        let Some(tx) = ctx.exit_plan_mode_tx.as_ref() else {
            return Ok(ToolOutput::new(CANCEL_STAY_IN_PLAN));
        };

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        let request_id = format!("exit-plan-{}", ctx.tool_call_id);
        if tx
            .send(ExitPlanModeInputRequest {
                request_id,
                session_id: ctx.session_id.clone(),
                tool_call_id: ctx.tool_call_id.clone(),
                plan_content,
                response_tx,
            })
            .is_err()
        {
            return Err(anyhow!(
                "ExitPlanMode session ended unexpectedly (bridge channel closed)"
            ));
        }

        let response_value = match response_rx.await {
            Ok(Ok(v)) => v,
            Ok(Err(msg)) => {
                return Err(anyhow!("Failed to reach the client for plan approval: {msg}"));
            }
            Err(_closed) => {
                return Err(anyhow!(
                    "ExitPlanMode session ended unexpectedly (client may have disconnected)"
                ));
            }
        };

        let ext: ExitPlanModeExtResponse = serde_json::from_value(response_value)
            .map_err(|e| anyhow!("Client returned an invalid ExitPlanMode response: {e}"))?;

        let message = match ext.outcome.as_str() {
            "approved" => {
                crate::dcg_bridge::leave_plan_mode_for_session(&ctx.session_id);
                APPROVED_MSG.to_string()
            }
            "abandoned" => {
                crate::dcg_bridge::leave_plan_mode_for_session(&ctx.session_id);
                ABANDONED_MSG.to_string()
            }
            "cancelled" => match ext.feedback.filter(|f| !f.trim().is_empty()) {
                Some(feedback) => {
                    format!("{CANCEL_STAY_IN_PLAN}\n\nUser feedback:\n{feedback}")
                }
                None => CANCEL_STAY_IN_PLAN.to_string(),
            },
            other => {
                return Err(anyhow!("Unknown ExitPlanMode outcome: {other}"));
            }
        };

        Ok(ToolOutput::new(message))
    }
}

fn read_plan_md(ctx: &ToolContext) -> Option<String> {
    let base = ctx
        .working_dir
        .clone()
        .or_else(|| std::env::current_dir().ok())?;
    let path = base.join("plan.md");
    std::fs::read_to_string(path)
        .ok()
        .filter(|s| !s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    #[tokio::test]
    async fn approved_restores_pre_plan_mode() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ExitPlanModeInputRequest>();
        let tool = ExitPlanModeTool::new();
        let handle = tokio::spawn(async move {
            let req = rx.recv().await.expect("request");
            let resp = ExitPlanModeExtResponse {
                outcome: "approved".into(),
                feedback: None,
            };
            let _ = req
                .response_tx
                .send(Ok(serde_json::to_value(resp).unwrap()));
        });

        crate::dcg_bridge::set_mode(crate::dcg_bridge::Mode::AcceptEdits);
        crate::dcg_bridge::enter_plan_mode_for_session("sess-exit-ok");

        let ctx = ToolContext {
            session_id: "sess-exit-ok".into(),
            tool_call_id: "tc-1".into(),
            exit_plan_mode_tx: Some(tx),
            ..Default::default()
        };
        let out = tool.execute(json!({}), ctx).await.unwrap();
        assert!(out.output.contains("approved"));
        assert_eq!(
            crate::dcg_bridge::session_mode("sess-exit-ok"),
            Some(crate::dcg_bridge::Mode::AcceptEdits)
        );
        handle.await.unwrap();
        crate::dcg_bridge::clear_session_mode("sess-exit-ok");
        crate::dcg_bridge::set_mode(crate::dcg_bridge::Mode::Default);
    }

    #[tokio::test]
    async fn cancelled_stays_in_plan_with_feedback() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ExitPlanModeInputRequest>();
        let tool = ExitPlanModeTool::new();
        let handle = tokio::spawn(async move {
            let req = rx.recv().await.expect("request");
            let resp = ExitPlanModeExtResponse {
                outcome: "cancelled".into(),
                feedback: Some("Add rollback steps".into()),
            };
            let _ = req
                .response_tx
                .send(Ok(serde_json::to_value(resp).unwrap()));
        });

        crate::dcg_bridge::enter_plan_mode_for_session("sess-exit-cancel");

        let ctx = ToolContext {
            session_id: "sess-exit-cancel".into(),
            tool_call_id: "tc-2".into(),
            exit_plan_mode_tx: Some(tx),
            ..Default::default()
        };
        let out = tool.execute(json!({}), ctx).await.unwrap();
        assert!(out.output.contains("Stay in plan mode"));
        assert!(out.output.contains("Add rollback steps"));
        assert_eq!(
            crate::dcg_bridge::session_mode("sess-exit-cancel"),
            Some(crate::dcg_bridge::Mode::Plan)
        );
        handle.await.unwrap();
        crate::dcg_bridge::clear_session_mode("sess-exit-cancel");
        crate::dcg_bridge::set_mode(crate::dcg_bridge::Mode::Default);
    }

    #[tokio::test]
    async fn abandoned_leaves_plan() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ExitPlanModeInputRequest>();
        let tool = ExitPlanModeTool::new();
        let handle = tokio::spawn(async move {
            let req = rx.recv().await.expect("request");
            let resp = ExitPlanModeExtResponse {
                outcome: "abandoned".into(),
                feedback: None,
            };
            let _ = req
                .response_tx
                .send(Ok(serde_json::to_value(resp).unwrap()));
        });

        // Reset global mode so parallel tests that call set_mode_from_str
        // cannot poison the pre-plan stash (restore must be Default).
        crate::dcg_bridge::set_mode(crate::dcg_bridge::Mode::Default);
        crate::dcg_bridge::clear_session_mode("sess-exit-abandon");
        crate::dcg_bridge::enter_plan_mode_for_session("sess-exit-abandon");

        let ctx = ToolContext {
            session_id: "sess-exit-abandon".into(),
            tool_call_id: "tc-3".into(),
            exit_plan_mode_tx: Some(tx),
            ..Default::default()
        };
        let out = tool.execute(json!({}), ctx).await.unwrap();
        assert!(out.output.contains("abandoned"));
        assert_eq!(
            crate::dcg_bridge::session_mode("sess-exit-abandon"),
            Some(crate::dcg_bridge::Mode::Default)
        );
        handle.await.unwrap();
        crate::dcg_bridge::clear_session_mode("sess-exit-abandon");
    }

    #[tokio::test]
    async fn missing_bridge_fail_closed_stay_in_plan() {
        let tool = ExitPlanModeTool::new();
        let out = tool
            .execute(json!({}), ToolContext::default())
            .await
            .unwrap();
        assert!(out.output.contains("Stay in plan mode") || out.output.contains("cancelled"));
    }

    #[tokio::test]
    async fn loads_plan_md_when_content_omitted() {
        let dir = tempfile::tempdir().unwrap();
        let plan_path = dir.path().join("plan.md");
        {
            let mut f = std::fs::File::create(&plan_path).unwrap();
            writeln!(f, "# Ship it").unwrap();
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ExitPlanModeInputRequest>();
        let tool = ExitPlanModeTool::new();
        let handle = tokio::spawn(async move {
            let req = rx.recv().await.expect("request");
            assert!(
                req.plan_content
                    .as_deref()
                    .is_some_and(|c| c.contains("Ship it")),
                "expected plan.md body, got {:?}",
                req.plan_content
            );
            let resp = ExitPlanModeExtResponse {
                outcome: "cancelled".into(),
                feedback: None,
            };
            let _ = req
                .response_tx
                .send(Ok(serde_json::to_value(resp).unwrap()));
        });

        let ctx = ToolContext {
            session_id: "sess-exit-planfile".into(),
            tool_call_id: "tc-4".into(),
            working_dir: Some(dir.path().to_path_buf()),
            exit_plan_mode_tx: Some(tx),
            ..Default::default()
        };
        let _ = tool.execute(json!({}), ctx).await.unwrap();
        handle.await.unwrap();
    }
}
