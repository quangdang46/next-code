//! BestOfN — workflow handler.
//!
//! Injects prompt guidance so the model uses the best-of-N tool pipeline.
//! Real orchestration lives in next-code-app-core (`best_of_n_edit` /
//! `propose_*` / `best_of_n_apply` and `Agent::run_best_of_n`).

use super::{WorkflowAction, WorkflowContext, WorkflowHandler};
use crate::registry::WorkflowKind;

pub struct BestOfNHandler;

impl WorkflowHandler for BestOfNHandler {
    fn kind(&self) -> WorkflowKind {
        WorkflowKind::BestOfN
    }

    fn build_prompt(&self) -> String {
        "# $bestofn — Best-of-N Editing Mode ENABLED

MANDATORY: Prefer the best-of-N tool pipeline for non-trivial edits.

## Pipeline (required for multi-approach edits)
1. Call `best_of_n_edit` with the user request (and optional context_files).
2. Draft **N different approaches** using only:
   - `propose_edit`
   - `propose_hashline`
   - `propose_write`
   These stage changes in memory — they do **not** write disk.
3. Call `best_of_n_apply` to select the best staged proposal and write it.

## Strategy diversity
Vary approaches across drafts, e.g.:
- minimal / surgical change
- clearer structure / extract helper
- more defensive / edge-case heavy

## When NOT to use best-of-N
- One-line typo / rename
- Pure Q&A or read-only investigation
- User explicitly wants a single quick edit

## Config
Requires `[best_of_n] mode = \"auto\"` (or `show`). Default is `off`.
Keyword `$bestofn` activates this guidance for the sticky turn budget."
            .to_string()
    }

    fn execute(&self, ctx: &WorkflowContext) -> WorkflowAction {
        // Host tools perform the real work; keyword path only injects guidance.
        let _ = ctx;
        WorkflowAction::Continue
    }
}
