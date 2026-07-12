//! Best-of-N edit tools — orchestrates draft-then-select workflow.
//!
//! MVP approach: instead of spawning sub-agents (which needs provider wiring),
//! the model calls `best_of_n_edit` to set up context, then uses propose_*
//! tools directly, then calls `best_of_n_apply` to select + apply the best.

use super::{Tool, ToolContext, ToolOutput, get_best_of_n_handle, clear_best_of_n_handle};
use anyhow::Result;
use async_trait::async_trait;
use jcode_best_of_n::{CandidateDiff, CandidateId, CandidateStatus, CandidateStrategy,
    FileDiff, ProposedContentStore, RunId};
use std::sync::Arc;
use serde::Deserialize;
use serde_json::{Value, json};

// ─── best_of_n_edit: set up context + instruct model ───────────────

pub struct BestOfNEditTool;

impl BestOfNEditTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct BestOfNEditInput {
    request: String,
    #[serde(default)]
    context_files: Vec<String>,
}

#[async_trait]
impl Tool for BestOfNEditTool {
    fn name(&self) -> &str { "best_of_n_edit" }

    fn description(&self) -> &str {
        "Start a best-of-N editing session. Sets up the store so propose_* tools \
         can draft changes. After calling this, use propose_edit/propose_hashline \
         N times with different approaches, then call best_of_n_apply to select \
         and apply the best one."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["request"],
            "properties": {
                "request": {
                    "type": "string",
                    "description": "The edit request to process."
                },
                "context_files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Files to read as context."
                }
            }
        })
    }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput> {
        let params: BestOfNEditInput = serde_json::from_value(input)?;
        let cfg = crate::config::config().best_of_n.clone();

        if !cfg.enabled() {
            return Err(anyhow::anyhow!(
                "Best-of-N is disabled. Set [best_of_n] mode = \"auto\" in config."
            ));
        }

        let run_id = RunId::new();
        let store = Arc::new(ProposedContentStore::new());

        // Register global handle
        let handle = super::BestOfNOrchestratorHandle {
            run_id: run_id.to_string(),
            candidate_id: String::new(),
            config: cfg.clone(),
            store: store.clone(),
        };
        super::set_best_of_n_handle(handle);

        let count = cfg.effective_count();
        Ok(ToolOutput::new(format!(
            "Best-of-N session started.\n\
             Run ID: {}\n\
             Candidates: {}\n\n\
             ## Next Steps\n\
             1. Read any needed files first\n\
             2. Use `propose_edit` or `propose_hashline` {} times with different approaches\n\
                (each call drafts changes in memory — nothing is written to disk)\n\
             3. Call `best_of_n_apply` to select the best and apply it\n\n\
             Request: {}",
            run_id, count, count, params.request,
        )))
    }
}

// ─── best_of_n_apply: select + apply ───────────────

pub struct BestOfNApplyTool;

impl BestOfNApplyTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct BestOfNApplyInput {
    /// Optional description of what each candidate approach was.
    #[serde(default)]
    descriptions: Vec<String>,
}

#[async_trait]
impl Tool for BestOfNApplyTool {
    fn name(&self) -> &str { "best_of_n_apply" }

    fn description(&self) -> &str {
        "Finish a best-of-N session: selects the best proposed changes and applies \
         them to disk. Call this after using propose_edit/propose_hashline multiple times."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "descriptions": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional descriptions of each candidate's approach."
                }
            }
        })
    }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput> {
        let _params: BestOfNApplyInput = serde_json::from_value(input)?;

        let handle = get_best_of_n_handle()
            .ok_or_else(|| anyhow::anyhow!("No active best-of-N session. Call best_of_n_edit first."))?;

        let run_id = RunId(handle.run_id.clone());
        let store = handle.store.clone();
        let config = handle.config.clone();

        // Collect all proposals
        let all_proposed = store.get_all_proposed(&run_id);

        if all_proposed.is_empty() {
            clear_best_of_n_handle();
            return Err(anyhow::anyhow!(
                "No proposals found. Use propose_edit/propose_hashline to draft changes first."
            ));
        }

        // Group by candidate_id
        let mut candidate_map: std::collections::HashMap<String, Vec<(String, jcode_best_of_n::ProposedEntry)>> =
            std::collections::HashMap::new();
        for (path, entry) in all_proposed {
            candidate_map
                .entry(entry.candidate_id.clone())
                .or_default()
                .push((path, entry));
        }

        // Build CandidateDiff for each candidate
        let candidates: Vec<CandidateDiff> = candidate_map
            .iter()
            .enumerate()
            .map(|(i, (cid, files))| {
                let file_diffs: Vec<FileDiff> = files
                    .iter()
                    .map(|(path, entry)| FileDiff {
                        file_path: path.clone(),
                        unified_diff: String::new(),
                        old_content: String::new(),
                        new_content: entry.content.clone(),
                        is_new_file: entry.is_new_file,
                    })
                    .collect();

                let status = if file_diffs.iter().any(|d| !d.new_content.is_empty()) {
                    CandidateStatus::Success
                } else {
                    CandidateStatus::NoChanges
                };

                CandidateDiff {
                    candidate_id: CandidateId(cid.clone()),
                    strategy: CandidateStrategy {
                        label: format!("candidate-{}", i),
                        temperature: 0.5,
                        model: None,
                    },
                    status,
                    file_diffs,
                    total_tokens: None,
                    error: None,
                }
            })
            .collect();

        // Select winner
        let selection = jcode_best_of_n::select_best_candidate(&candidates, &config.selector);

        // Apply winner
        let mut files_applied = Vec::new();
        if let Some(winner) = candidates.get(selection.winner_index) {
            for diff in &winner.file_diffs {
                let path = std::path::Path::new(&diff.file_path);
                if let Some(parent) = path.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                if let Ok(()) = tokio::fs::write(path, &diff.new_content).await {
                    files_applied.push(diff.file_path.clone());
                }
            }
        }

        // Cleanup
        clear_best_of_n_handle();

        let files_list = if files_applied.is_empty() {
            "No files changed".to_string()
        } else {
            files_applied.iter().map(|f| format!("  - {}", f)).collect::<Vec<_>>().join("\n")
        };

        Ok(ToolOutput::new(format!(
            "Best-of-N complete.\n\
             Candidates: {}\n\
             Winner: {} (strategy: {})\n\
             Reason: {}\n\
             Files applied:\n{}",
            candidates.len(),
            selection.winner_index,
            candidates.get(selection.winner_index)
                .map(|c| c.strategy.label.as_str())
                .unwrap_or("unknown"),
            selection.reason,
            files_list,
        )))
    }
}
