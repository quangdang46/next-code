//! Best-of-N tools.
//!
//! Manual pipeline (works today without sub-agent spawn):
//! 1. `best_of_n_edit` — open a run + install store handle
//! 2. `propose_edit` / `propose_hashline` / `propose_write` — draft N approaches
//! 3. `best_of_n_apply` — select best + write to disk
//!
//! Full auto-spawn orchestrator lives in `agent::best_of_n_orchestrator` and is
//! callable when a parent `Agent` is available (e.g. future keyword hook).

use super::{Tool, ToolContext, ToolOutput, clear_best_of_n_handle, get_best_of_n_handle};
use anyhow::Result;
use async_trait::async_trait;
use jcode_best_of_n::{ProposedContentStore, RunId};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

// ─── best_of_n_edit ───────────────────────────────────────────────

pub struct BestOfNEditTool;

impl BestOfNEditTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BestOfNEditInput {
    request: String,
    #[serde(default)]
    context_files: Vec<String>,
}

#[async_trait]
impl Tool for BestOfNEditTool {
    fn name(&self) -> &str {
        "best_of_n_edit"
    }

    fn description(&self) -> &str {
        "Start a best-of-N editing session. Installs a draft store so propose_* tools \
         can stage changes without writing disk. After drafting N approaches with \
         propose_edit/propose_hashline/propose_write, call best_of_n_apply to select \
         and apply the best. Requires [best_of_n] mode = auto|show."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["request"],
            "properties": {
                "request": {
                    "type": "string",
                    "description": "The edit request / task description."
                },
                "context_files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional files to read first as context."
                }
            }
        })
    }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput> {
        let params: BestOfNEditInput = serde_json::from_value(input)?;
        let cfg = crate::config::config().best_of_n.clone();
        if !cfg.enabled() {
            return Err(anyhow::anyhow!(
                "Best-of-N is disabled. Set [best_of_n] mode = \"auto\" in config.toml \
                 (or JCODE_BEST_OF_N_MODE=auto)."
            ));
        }

        let run_id = RunId::new();
        let store = Arc::new(ProposedContentStore::new());
        let count = cfg.effective_count();

        super::set_best_of_n_handle(super::BestOfNOrchestratorHandle {
            run_id: run_id.to_string(),
            candidate_id: String::new(),
            config: cfg,
            store,
        });

        let ctx_hint = if params.context_files.is_empty() {
            String::new()
        } else {
            format!(
                "\nSuggested context files:\n{}",
                params
                    .context_files
                    .iter()
                    .map(|f| format!("  - {f}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        Ok(ToolOutput::new(format!(
            "Best-of-N session started (run {run_id}).\n\
             Target candidates: {count}\n\
             Request: {}\n\
             {ctx_hint}\n\n\
             ## Next steps\n\
             1. Read any needed files.\n\
             2. Draft {count} different approaches with propose_edit / propose_hashline / propose_write.\n\
                (Each draft is staged in memory — nothing hits disk yet.)\n\
             3. Call best_of_n_apply to pick the best and write it.\n\
             Tip: vary strategy across drafts (minimal change vs modular refactor, etc.).",
            params.request,
        )))
    }
}

// ─── best_of_n_apply ──────────────────────────────────────────────

pub struct BestOfNApplyTool;

impl BestOfNApplyTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize, Default)]
struct BestOfNApplyInput {}

#[async_trait]
impl Tool for BestOfNApplyTool {
    fn name(&self) -> &str {
        "best_of_n_apply"
    }

    fn description(&self) -> &str {
        "Finish a best-of-N session: select the best staged proposal and apply it to disk. \
         Call after best_of_n_edit + one or more propose_* drafts."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput> {
        let _: BestOfNApplyInput = serde_json::from_value(input).unwrap_or_default();

        let handle = get_best_of_n_handle().ok_or_else(|| {
            anyhow::anyhow!("No active best-of-N session. Call best_of_n_edit first.")
        })?;

        let run_id = RunId(handle.run_id.clone());
        let result = crate::agent::best_of_n_orchestrator::select_and_apply_from_store(
            &run_id,
            &handle.store,
            &handle.config,
        )?;

        clear_best_of_n_handle();

        let files_list = if result.files_applied.is_empty() {
            "  (none)".to_string()
        } else {
            result
                .files_applied
                .iter()
                .map(|f| format!("  - {f}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let winner_label = result
            .candidates
            .get(result.winner_index)
            .map(|c| c.strategy.label.as_str())
            .unwrap_or("unknown");

        Ok(ToolOutput::new(format!(
            "Best-of-N apply complete.\n\
             Candidates: {}\n\
             Winner: #{} ({})\n\
             Reason: {}\n\
             Files applied:\n{}",
            result.candidates.len(),
            result.winner_index,
            winner_label,
            result.selection_reason,
            files_list,
        )))
    }
}
