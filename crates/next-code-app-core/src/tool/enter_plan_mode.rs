//! EnterPlanMode — switch the session into plan-only mode (no confirm dialog).

use super::{Tool, ToolContext, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};

pub struct EnterPlanModeTool;

impl EnterPlanModeTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnterPlanModeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for EnterPlanModeTool {
    fn name(&self) -> &str {
        "EnterPlanMode"
    }

    fn description(&self) -> &str {
        r#"Enter plan mode for complex tasks that need exploration before coding.

In plan mode, mutating tools are blocked except writes to plan.md. Write a clear
plan, then call ExitPlanMode for user approval before implementing."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": super::intent_schema_property()
            }
        })
    }

    fn declared_tier(&self) -> Option<next_code_tool_types::ToolTier> {
        Some(next_code_tool_types::ToolTier::Read)
    }

    async fn execute(&self, _input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        crate::dcg_bridge::enter_plan_mode_for_session(&ctx.session_id);
        Ok(ToolOutput::new(concat!(
            "Entered plan mode. Explore the codebase and design an approach. ",
            "DO NOT write or edit any files except plan.md. When ready, call ",
            "ExitPlanMode to present the plan for approval."
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sets_plan_mode_and_stashes_pre_plan() {
        crate::dcg_bridge::set_mode(crate::dcg_bridge::Mode::AcceptEdits);
        crate::dcg_bridge::set_session_mode(
            "sess-enter-plan",
            crate::dcg_bridge::Mode::AcceptEdits,
        );

        let tool = EnterPlanModeTool::new();
        let ctx = ToolContext {
            session_id: "sess-enter-plan".into(),
            ..Default::default()
        };
        let out = tool.execute(json!({}), ctx).await.unwrap();
        assert!(out.output.contains("Entered plan mode"));
        assert_eq!(
            crate::dcg_bridge::session_mode("sess-enter-plan"),
            Some(crate::dcg_bridge::Mode::Plan)
        );
        assert_eq!(
            crate::dcg_bridge::current_mode(),
            crate::dcg_bridge::Mode::Plan
        );

        let restored = crate::dcg_bridge::leave_plan_mode_for_session("sess-enter-plan");
        assert_eq!(restored, crate::dcg_bridge::Mode::AcceptEdits);
        crate::dcg_bridge::clear_session_mode("sess-enter-plan");
        crate::dcg_bridge::set_mode(crate::dcg_bridge::Mode::Default);
    }
}
