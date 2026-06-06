//! Analyze — DeepAnalysis workflow handler.
//!
//! Tier 1: Prompt-only. Injects structured analysis instructions.

use super::WorkflowHandler;
use crate::registry::WorkflowKind;

pub struct AnalyzeHandler;

impl WorkflowHandler for AnalyzeHandler {
    fn kind(&self) -> WorkflowKind {
        WorkflowKind::Analyze
    }

    fn build_prompt(&self) -> String {
        "# $analyze — Deep Analysis Mode\n\n\
         Perform structured, thorough analysis.\n\n\
         ## Strategy\n\
         1. Map architecture and dependencies\n\
         2. Identify patterns and anti-patterns\n\
         3. Assess complexity and quality\n\
         4. Generate ranked recommendations\n\n\
         ## Output\n\
         - Summary paragraph\n\
         - Findings with severity (Critical/High/Medium/Low)\n\
         - file:line references\n\
         - Top 3 priority actions"
            .to_string()
    }

    // Use trait default: Continue
}
