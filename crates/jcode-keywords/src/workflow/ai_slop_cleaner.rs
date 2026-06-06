//! AiSlopCleaner — SlopCleanup workflow handler.
//!
//! Tier 1: Prompt-only. Injects AI code quality improvement instructions.

use super::WorkflowHandler;
use crate::registry::WorkflowKind;

pub struct AiSlopCleanerHandler;

impl WorkflowHandler for AiSlopCleanerHandler {
    fn kind(&self) -> WorkflowKind {
        WorkflowKind::AiSlopCleaner
    }

    fn build_prompt(&self) -> String {
        "# ai-slop-cleaner — AI Slop Cleanup Mode\n\n\
         Detect and fix low-quality AI-generated code.\n\n\
         ## Look For\n\
         1. Redundant comments (restating the code)\n\
         2. Over-abstraction (unnecessary wrappers)\n\
         3. Dead code (unused imports, variables)\n\
         4. Verbose patterns (could be simplified)\n\
         5. Generic names (data, result, temp, helper)\n\
         6. Unnecessary .clone() calls\n\n\
         ## Rules\n\
         - Don't change behavior\n\
         - Preserve public API contracts\n\
         - Keep fixes minimal and focused"
            .to_string()
    }

    // Use trait default: Continue
}
