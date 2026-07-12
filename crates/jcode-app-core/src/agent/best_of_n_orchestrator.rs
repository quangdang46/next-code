//! Best-of-N orchestration helpers.
//!
//! Primary path today: tool pipeline
//!   best_of_n_edit → propose_* (drafts) → best_of_n_apply
//! via [`select_and_apply_from_store`].
//!
//! Full multi-agent spawn is available as [`run_best_of_n`] when a parent
//! [`Agent`] is in hand (e.g. future keyword / turn-loop hook).

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use jcode_best_of_n::{
    BestOfNConfig, CandidateDiff, CandidateId, CandidateStatus, CandidateStrategy, FileDiff,
    ProposedContentStore, RunId, select_best_candidate,
};

use crate::agent::Agent;
use crate::provider::Provider;
use crate::session::Session;
use crate::tool::{BestOfNOrchestratorHandle, Registry};

/// Result of a best-of-N run for tool/UI reporting.
pub struct BestOfNRunResult {
    pub run_id: String,
    pub winner_index: usize,
    pub candidates: Vec<CandidateDiff>,
    pub selection_reason: String,
    pub files_applied: Vec<String>,
}

/// Select winner from proposals already in the store and apply to disk.
pub fn select_and_apply_from_store(
    run_id: &RunId,
    store: &Arc<ProposedContentStore>,
    config: &BestOfNConfig,
) -> Result<BestOfNRunResult> {
    let all = store.get_all_proposed(run_id);
    if all.is_empty() {
        return Err(anyhow::anyhow!(
            "No proposals found. Use propose_edit / propose_hashline / propose_write first."
        ));
    }

    let mut by_candidate: HashMap<String, Vec<(String, jcode_best_of_n::ProposedEntry)>> =
        HashMap::new();
    for (path, entry) in all {
        by_candidate
            .entry(entry.candidate_id.clone())
            .or_default()
            .push((path, entry));
    }

    let candidates: Vec<CandidateDiff> = by_candidate
        .into_iter()
        .enumerate()
        .map(|(i, (cid, files))| {
            let file_diffs: Vec<FileDiff> = files
                .into_iter()
                .map(|(path, entry)| FileDiff {
                    file_path: path,
                    unified_diff: String::new(),
                    old_content: String::new(),
                    new_content: entry.content,
                    is_new_file: entry.is_new_file,
                })
                .collect();
            let status = if file_diffs.is_empty() {
                CandidateStatus::NoChanges
            } else {
                CandidateStatus::Success
            };
            CandidateDiff {
                candidate_id: CandidateId(cid),
                strategy: CandidateStrategy {
                    label: format!("candidate-{i}"),
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

    let selection = select_best_candidate(&candidates, &config.selector);
    let files_applied = apply_winner(run_id, &candidates, selection.winner_index, store);
    store.clear_run(run_id);

    Ok(BestOfNRunResult {
        run_id: run_id.to_string(),
        winner_index: selection.winner_index,
        candidates,
        selection_reason: selection.reason,
        files_applied,
    })
}

/// Full multi-agent best-of-N (requires parent Agent for provider/registry).
pub async fn run_best_of_n(
    parent: &Agent,
    user_request: &str,
    context_files: &[String],
) -> Result<BestOfNRunResult> {
    let cfg = crate::config::config().best_of_n.clone();
    if !cfg.enabled() {
        return Err(anyhow::anyhow!(
            "Best-of-N is disabled. Set [best_of_n] mode = \"auto\"."
        ));
    }

    let run_id = RunId::new();
    let store = Arc::new(ProposedContentStore::new());
    let strategies =
        jcode_best_of_n::strategies::generate_strategies(cfg.effective_count(), &cfg.temperatures);

    crate::tool::set_best_of_n_handle(BestOfNOrchestratorHandle {
        run_id: run_id.to_string(),
        candidate_id: String::new(),
        config: cfg.clone(),
        store: store.clone(),
    });

    let context_block = load_context_files(context_files).await;
    let mut candidates = Vec::with_capacity(strategies.len());

    for (i, strategy) in strategies.iter().enumerate() {
        let candidate_id = CandidateId::new(i);
        let prompt =
            build_candidate_prompt(&candidate_id, &strategy.label, user_request, &context_block);
        match spawn_and_run_candidate(parent, &run_id, &candidate_id, strategy, &prompt, &store)
            .await
        {
            Ok(c) => candidates.push(c),
            Err(e) => {
                crate::logging::warn(&format!(
                    "[best-of-n] candidate {candidate_id} failed: {e}"
                ));
                candidates.push(CandidateDiff {
                    candidate_id: candidate_id.clone(),
                    strategy: strategy.clone(),
                    status: CandidateStatus::Failed,
                    file_diffs: Vec::new(),
                    total_tokens: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let selection = select_best_candidate(&candidates, &cfg.selector);
    let files_applied = apply_winner(&run_id, &candidates, selection.winner_index, &store);

    crate::tool::clear_best_of_n_handle();
    store.clear_run(&run_id);

    Ok(BestOfNRunResult {
        run_id: run_id.to_string(),
        winner_index: selection.winner_index,
        candidates,
        selection_reason: selection.reason,
        files_applied,
    })
}

async fn spawn_and_run_candidate(
    parent: &Agent,
    run_id: &RunId,
    candidate_id: &CandidateId,
    strategy: &CandidateStrategy,
    prompt: &str,
    store: &Arc<ProposedContentStore>,
) -> Result<CandidateDiff> {
    let (provider, registry, parent_session) = parent.best_of_n_spawn_parts();
    let mut child_session = Session::create(Some(parent_session.id.clone()), None);
    child_session.working_dir = parent_session.working_dir.clone();
    child_session.model = parent_session.model.clone();
    child_session.provider_key = parent_session.provider_key.clone();

    let mut allowed = std::collections::HashSet::new();
    for name in [
        "read",
        "propose_edit",
        "propose_hashline",
        "propose_write",
        "ffs_grep",
        "ffs_outline",
        "ffs_glob",
        "ls",
        "glob",
        "grep",
    ] {
        allowed.insert(name.to_string());
    }

    let mut child =
        Agent::new_with_session(provider, registry, child_session, Some(allowed));
    child.set_best_of_n_context(run_id.to_string(), candidate_id.to_string());

    if let Err(e) = child.run_once_capture_inner(prompt).await {
        crate::logging::warn(&format!(
            "[best-of-n] candidate {candidate_id} turn error: {e}"
        ));
    }

    Ok(candidate_from_store(run_id, candidate_id, strategy, store))
}

fn candidate_from_store(
    run_id: &RunId,
    candidate_id: &CandidateId,
    strategy: &CandidateStrategy,
    store: &Arc<ProposedContentStore>,
) -> CandidateDiff {
    let cid = candidate_id.to_string();
    let file_diffs: Vec<FileDiff> = store
        .get_all_proposed(run_id)
        .into_iter()
        .filter(|(_, entry)| entry.candidate_id == cid)
        .map(|(path, entry)| FileDiff {
            file_path: path,
            unified_diff: String::new(),
            old_content: String::new(),
            new_content: entry.content,
            is_new_file: entry.is_new_file,
        })
        .collect();

    CandidateDiff {
        candidate_id: candidate_id.clone(),
        strategy: strategy.clone(),
        status: if file_diffs.is_empty() {
            CandidateStatus::NoChanges
        } else {
            CandidateStatus::Success
        },
        file_diffs,
        total_tokens: None,
        error: None,
    }
}

fn apply_winner(
    run_id: &RunId,
    candidates: &[CandidateDiff],
    winner_index: usize,
    store: &Arc<ProposedContentStore>,
) -> Vec<String> {
    let Some(winner) = candidates.get(winner_index) else {
        return Vec::new();
    };
    if winner.status != CandidateStatus::Success {
        return Vec::new();
    }

    let cid = winner.candidate_id.to_string();
    let mut files_applied = Vec::new();
    for (path, entry) in store.get_all_proposed(run_id) {
        if entry.candidate_id != cid {
            continue;
        }
        let file_path = std::path::Path::new(&path);
        if let Some(parent) = file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(file_path, &entry.content).is_ok() {
            files_applied.push(path);
        }
    }
    files_applied
}

async fn load_context_files(paths: &[String]) -> String {
    let mut out = String::new();
    for path in paths {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => {
                let clipped = if content.len() > 12_000 {
                    format!("{}…\n[truncated]", &content[..12_000])
                } else {
                    content
                };
                out.push_str(&format!("\n--- {path} ---\n{clipped}\n"));
            }
            Err(_) => out.push_str(&format!("\n--- {path} ---\n[unreadable]\n")),
        }
    }
    out
}

fn build_candidate_prompt(
    candidate_id: &CandidateId,
    strategy_label: &str,
    user_request: &str,
    context_block: &str,
) -> String {
    format!(
        "You are best-of-N implementation candidate {candidate_id}.\n\
         Strategy label: {strategy_label}\n\n\
         ## User request\n\
         {user_request}\n\n\
         ## Context files\n\
         {context_block}\n\n\
         ## Rules\n\
         - Draft ALL needed changes using only: propose_edit, propose_hashline, propose_write.\n\
         - Do NOT use edit/write/apply_patch — those write to disk.\n\
         - Prefer complete, correct, minimal diffs for your strategy.\n\
         - When done drafting, stop (no summary required).\n"
    )
}

impl Agent {
    /// Parts needed to spawn a best-of-N child agent from a parent.
    pub(crate) fn best_of_n_spawn_parts(&self) -> (Arc<dyn Provider>, Registry, Session) {
        (
            Arc::clone(&self.provider),
            self.registry.clone(),
            self.session.clone(),
        )
    }

    /// Convenience: run best-of-N for a request using this agent as parent.
    pub async fn run_best_of_n(
        &self,
        user_request: &str,
        context_files: &[String],
    ) -> Result<BestOfNRunResult> {
        run_best_of_n(self, user_request, context_files).await
    }
}
