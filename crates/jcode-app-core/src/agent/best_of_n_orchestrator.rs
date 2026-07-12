//! Best-of-N orchestrator — spawns parallel candidates, collects proposals, selects winner.
//!
//! MVP: The tool-based approach (best_of_n_edit → propose_* → best_of_n_apply) is the
//! primary entry point. This module provides the orchestrator logic for future use when
//! Agent gains a `run_single_turn` method for sub-agent spawning.

use std::sync::Arc;

use anyhow::Result;
use jcode_best_of_n::{
    BestOfNConfig, CandidateDiff, CandidateId, CandidateStatus, CandidateStrategy,
    FileDiff, ProposedContentStore, RunId, SelectionResult,
    select_best_candidate,
};

/// Result of a best-of-N run.
pub struct BestOfNRunResult {
    pub run_id: String,
    pub winner_index: usize,
    pub candidates: Vec<CandidateDiff>,
    pub selection_reason: String,
    pub files_applied: Vec<String>,
}

/// Select the best candidate from a list and apply its edits.
pub fn select_and_apply(
    run_id: &RunId,
    candidates: Vec<CandidateDiff>,
    config: &BestOfNConfig,
    store: &Arc<ProposedContentStore>,
) -> (BestOfNRunResult, Vec<String>) {
    let selection = select_best_candidate(&candidates, &config.selector);
    let files_applied = apply_winner(run_id, &candidates, &selection, store);

    let selection_reason = selection.reason.clone();

    (BestOfNRunResult {
        run_id: run_id.to_string(),
        winner_index: selection.winner_index,
        candidates,
        selection_reason,
        files_applied: files_applied.clone(),
    }, files_applied)
}

/// Apply the winning candidate's proposals to disk.
fn apply_winner(
    run_id: &RunId,
    candidates: &[CandidateDiff],
    selection: &SelectionResult,
    store: &Arc<ProposedContentStore>,
) -> Vec<String> {
    if selection.winner_index >= candidates.len() {
        return Vec::new();
    }

    let winner = &candidates[selection.winner_index];
    let mut files_applied = Vec::new();

    let all_proposed = store.get_all_proposed(run_id);

    for (path, entry) in all_proposed {
        if entry.candidate_id != winner.candidate_id.to_string() {
            continue;
        }

        let file_path = std::path::Path::new(&path);

        if let Some(parent) = file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(()) = std::fs::write(file_path, &entry.content) {
            files_applied.push(path);
        }
    }

    files_applied
}
