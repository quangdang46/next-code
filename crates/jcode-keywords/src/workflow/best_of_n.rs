//! BestOfN — workflow handler.
//!
//! Injects prompt guidance for best-of-N editing mode.
//! The actual orchestrator logic lives in jcode-app-core.

use super::{WorkflowAction, WorkflowContext, WorkflowHandler};
use crate::registry::WorkflowKind;

pub struct BestOfNHandler;

impl WorkflowHandler for BestOfNHandler {
    fn kind(&self) -> WorkflowKind {
        WorkflowKind::BestOfN
    }

    fn build_prompt(&self) -> String {
        "# $bestofn — Best-of-N Editing Mode

When best-of-N mode is active, the agent should use the `best_of_n_edit` tool
instead of direct `edit` calls for non-trivial changes. This spawns parallel
candidates and picks the best implementation.

## When to Use
- Multi-file edits with multiple valid approaches
- Complex refactors where alternatives exist
- Tasks where code quality matters more than speed

## When NOT to Use
- Simple one-line fixes
- Trivial renames or formatting
- Read-only tasks

The `best_of_n_edit` tool handles the orchestration internally."
            .to_string()
    }

    fn execute(&self, ctx: &WorkflowContext) -> WorkflowAction {
        // Best-of-N doesn't spawn agents from the keyword system.
        // The model calls the `best_of_n_edit` tool directly.
        let _ = ctx;
        WorkflowAction::Continue
    }
}
