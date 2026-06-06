//! Wiki — DocLookup workflow handler.
//!
//! Tier 1: Prompt-only. Injects documentation search instructions.

use super::WorkflowHandler;
use crate::registry::WorkflowKind;

pub struct WikiHandler;

impl WorkflowHandler for WikiHandler {
    fn kind(&self) -> WorkflowKind {
        WorkflowKind::Wiki
    }

    fn build_prompt(&self) -> String {
        "# $wiki — Documentation Lookup Mode\n\n\
         Search and synthesize documentation.\n\n\
         ## Search Strategy\n\
         1. Local docs: README, AGENTS.md, docs/\n\
         2. Code docs: docstrings, comments\n\
         3. Web docs: official documentation\n\
         4. Cross-reference multiple sources\n\n\
         ## Output\n\
         - Direct answer\n\
         - file:line references for local sources\n\
         - URLs for web sources"
            .to_string()
    }

    // Use trait default: Continue
}
