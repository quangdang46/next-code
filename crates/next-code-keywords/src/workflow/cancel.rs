//! Cancel — CancelAll workflow handler.
//!
//! Tier 6: System action. Cancel is handled entirely by `state::update_modes()`
//! which clears all modes before execute() is ever called. These methods are
//! no-ops in the normal flow.

use super::WorkflowHandler;
use crate::registry::WorkflowKind;

pub struct CancelHandler;

impl WorkflowHandler for CancelHandler {
    fn kind(&self) -> WorkflowKind {
        WorkflowKind::Cancel
    }

    fn build_prompt(&self) -> String {
        "# cancelnext — All Modes Cancelled\n\n\
         Returning to normal operation."
            .to_string()
    }

    // Note: execute() and on_turn_complete() are intentionally not overridden.
    // Cancel is handled by state::update_modes() which clears all modes
    // before execute_active_workflows() iterates them. The trait defaults
    // (returning Continue) are correct — this handler is unreachable in
    // the normal flow.
}
