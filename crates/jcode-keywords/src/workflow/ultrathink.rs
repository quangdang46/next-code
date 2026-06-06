//! Ultrathink — ExtendedThinking workflow handler.
//!
//! Tier 1: Prompt-only. Injects deep reasoning instructions into system prompt.

use super::WorkflowHandler;
use crate::registry::WorkflowKind;

pub struct UltrathinkHandler;

impl WorkflowHandler for UltrathinkHandler {
    fn kind(&self) -> WorkflowKind {
        WorkflowKind::Ultrathink
    }

    fn build_prompt(&self) -> String {
        "# $ultrathink — Extended Thinking Mode\n\n\
         Reason deeply and thoroughly about the problem.\n\n\
         ## Strategy\n\
         1. Break the problem into components\n\
         2. Consider edge cases and boundary conditions\n\
         3. Evaluate trade-offs between approaches\n\
         4. Consider alternatives and implications\n\
         5. Provide thorough analysis with reasoning chain"
            .to_string()
    }

    // Use trait default: Continue (no-op execute, no-op on_turn_complete)
    // Defer to turn-limit expiration in state::update_modes
}
