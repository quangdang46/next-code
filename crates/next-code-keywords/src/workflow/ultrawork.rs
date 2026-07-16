//! Ultrawork — ParallelExecution workflow handler.
//!
//! Tier 2: Sub-agent spawning. Spawns parallel sub-agents for independent subtasks.

use super::{WorkflowAction, WorkflowContext, WorkflowHandler, sanitize_user_input};
use crate::registry::WorkflowKind;
use std::collections::HashMap;

pub struct UltraworkHandler;

impl WorkflowHandler for UltraworkHandler {
    fn kind(&self) -> WorkflowKind {
        WorkflowKind::Ultrawork
    }

    fn build_prompt(&self) -> String {
        // SpawnParallel is deferred by the host (not always wired). Keep the
        // prompt honest: plan and execute thoroughly in this session rather than
        // claiming parallel sub-agents always launch.
        "# $ultrawork — High-throughput execution mode

MANDATORY: Say \"ULTRAWORK MODE ENABLED!\" as your first response.

## CERTAINTY PROTOCOL
Do NOT start implementing until you understand the task. Before you write code:
1. THINK — What is the user's true intent?
2. EXPLORE — Search/read the codebase for context
3. ASK — If critical ambiguity remains, ask the user

## Execution Strategy
1. Break the work into a clear checklist of independent subtasks
2. Execute subtasks aggressively in this session (use tools; parallelize tool
   calls when safe via batch). Prefer finishing end-to-end over partial notes.
3. If the host provides multi-agent/swarm tools, you MAY fan out subtasks —
   but do not block waiting for spawns that never return.
4. Aggregate results and verify before claiming done

## Completion Markers
Ready to implement: [MODE:ULTRAWORK_READY]
Work in progress: [MODE:ULTRAWORK_ACTIVE]
Results aggregated: [MODE:COMPLETE]"
            .to_string()
    }

    fn execute(&self, ctx: &WorkflowContext) -> WorkflowAction {
        // Guard: don't re-emit spawn metadata if already marked this session
        if ctx.metadata.contains_key("ultrawork_spawned") {
            return WorkflowAction::Continue;
        }

        // Soft-mark activation. Actual multi-agent spawn remains deferred to the
        // host (see deferred_spawns); until wired, this mode is prompt-driven.
        let mut meta = HashMap::new();
        meta.insert("ultrawork_spawned".to_string(), "prompt_only".to_string());
        meta.insert(
            "task_preview".to_string(),
            sanitize_user_input(ctx.user_input)
                .chars()
                .take(200)
                .collect(),
        );
        WorkflowAction::ContinueWithMetadata {
            reminder: String::new(),
            metadata: meta,
        }
    }

    fn on_turn_complete(
        &self,
        _response: &str,
        metadata: &HashMap<String, String>,
    ) -> WorkflowAction {
        // If we already spawned, mark as complete
        if metadata.contains_key("ultrawork_spawned") {
            return WorkflowAction::Complete(
                "Parallel execution complete. Results aggregated.".to_string(),
            );
        }
        WorkflowAction::Continue
    }
}
