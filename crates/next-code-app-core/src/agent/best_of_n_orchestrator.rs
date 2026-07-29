//! Best-of-N orchestration helpers.
//!
//! Primary path today: tool pipeline
//!   best_of_n_edit → propose_* (drafts) → best_of_n_apply
//! via [`select_and_apply_from_store`].
//!
//! Full multi-agent spawn is available as [`run_best_of_n`] when a parent
//! [`Agent`] is in hand (e.g. keyword / turn-loop hook).

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use next_code_best_of_n::{
    BestOfNCandidateUi, BestOfNConfig, BestOfNMode, BestOfNPhase, BestOfNPickExtResponse,
    BestOfNProgressPayload, CandidateDiff, CandidateId, CandidateStatus, CandidateStrategy,
    FileDiff, ProposedContentStore, RunId, select_best_candidate,
};

use crate::agent::Agent;
use crate::protocol::ServerEvent;
use crate::provider::Provider;
use crate::session::Session;
use crate::tool::{BestOfNOrchestratorHandle, BestOfNPickInputRequest, Registry};

/// Result of a best-of-N run for tool/UI reporting.
pub struct BestOfNRunResult {
    pub run_id: String,
    pub winner_index: usize,
    pub candidates: Vec<CandidateDiff>,
    pub selection_reason: String,
    pub files_applied: Vec<String>,
    /// True when the user cancelled in `mode=show` (no files applied).
    pub cancelled: bool,
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

    let mut by_candidate: HashMap<String, Vec<(String, next_code_best_of_n::ProposedEntry)>> =
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
        cancelled: false,
    })
}

/// Full multi-agent best-of-N (requires parent Agent for provider/registry).
pub async fn run_best_of_n(
    parent: &Agent,
    user_request: &str,
    context_files: &[String],
) -> Result<BestOfNRunResult> {
    run_best_of_n_with_progress(parent, user_request, context_files, &None).await
}

/// Like [`run_best_of_n`] but emits structured progress via an optional event sender.
///
/// When config `mode=show` and the parent has a Face pick channel, waits for the
/// user to select a winner (or cancel) before applying.
pub async fn run_best_of_n_with_progress(
    parent: &Agent,
    user_request: &str,
    context_files: &[String],
    event_tx: &Option<tokio::sync::mpsc::UnboundedSender<ServerEvent>>,
) -> Result<BestOfNRunResult> {
    let cfg = crate::config::config().best_of_n.clone();
    if !cfg.enabled() {
        return Err(anyhow::anyhow!(
            "Best-of-N is disabled. Set [best_of_n] mode = \"auto\" or \"show\"."
        ));
    }

    let (parent_provider, parent_registry, parent_session) = parent.best_of_n_spawn_parts();

    let run_id = RunId::new();
    let store = Arc::new(ProposedContentStore::new());
    let strategies =
        next_code_best_of_n::strategies::generate_strategies(cfg.effective_count(), &cfg.temperatures);
    let total = strategies.len();

    crate::tool::set_best_of_n_handle(BestOfNOrchestratorHandle {
        run_id: run_id.to_string(),
        candidate_id: String::new(),
        config: cfg.clone(),
        store: store.clone(),
    });

    let context_block = load_context_files(context_files).await;
    emit_bon_progress(
        event_tx,
        BestOfNProgressPayload {
            run_id: run_id.to_string(),
            phase: BestOfNPhase::Generating,
            message: format!("Generating {total} candidates…"),
            completed: 0,
            total,
            candidates: (0..total)
                .map(|i| BestOfNCandidateUi {
                    index: i,
                    candidate_id: CandidateId::new(i).to_string(),
                    label: strategies[i].label.clone(),
                    status: "running".to_string(),
                    file_count: 0,
                    files: Vec::new(),
                    error: None,
                    recommended: false,
                })
                .collect(),
            recommended_index: None,
            selection_reason: None,
        },
    );
    crate::logging::info(&format!(
        "[best-of-n] spawning {}/{} candidates for run {}",
        total,
        cfg.effective_count(),
        run_id
    ));

    let mut handles = Vec::with_capacity(strategies.len());
    for (i, strategy) in strategies.iter().enumerate() {
        let candidate_id = CandidateId::new(i);
        let prompt =
            build_candidate_prompt(&candidate_id, &strategy.label, user_request, &context_block);
        let strategy = strategy.clone();
        let rid = run_id.clone();
        let st = store.clone();
        let pv = parent_provider.clone();
        let rg = parent_registry.clone();
        let ps = parent_session.clone();

        handles.push(tokio::spawn(async move {
            match tokio::time::timeout(
                std::time::Duration::from_secs(120),
                spawn_and_run_candidate(
                    &rid,
                    &candidate_id,
                    &strategy,
                    &prompt,
                    &st,
                    &pv,
                    &rg,
                    &ps,
                ),
            )
            .await
            {
                Ok(Ok(c)) => c,
                Ok(Err(e)) => {
                    crate::logging::warn(&format!(
                        "[best-of-n] candidate {candidate_id} error: {e}"
                    ));
                    CandidateDiff {
                        candidate_id: candidate_id.clone(),
                        strategy,
                        status: CandidateStatus::Failed,
                        file_diffs: Vec::new(),
                        total_tokens: None,
                        error: Some(e.to_string()),
                    }
                }
                Err(_) => {
                    crate::logging::warn(&format!(
                        "[best-of-n] candidate {candidate_id} timed out (120s)"
                    ));
                    CandidateDiff {
                        candidate_id: candidate_id.clone(),
                        strategy,
                        status: CandidateStatus::Failed,
                        file_diffs: Vec::new(),
                        total_tokens: None,
                        error: Some("candidate timed out after 120s".to_string()),
                    }
                }
            }
        }));
    }

    let mut candidates = Vec::with_capacity(handles.len());
    for (i, handle) in handles.into_iter().enumerate() {
        crate::logging::info(&format!("[best-of-n] candidate {}/{} done", i + 1, total));
        match handle.await {
            Ok(c) => candidates.push(c),
            Err(e) => {
                crate::logging::warn(&format!("[best-of-n] candidate {i} panicked: {e}"));
                candidates.push(CandidateDiff {
                    candidate_id: CandidateId::new(i),
                    strategy: strategies[i].clone(),
                    status: CandidateStatus::Failed,
                    file_diffs: Vec::new(),
                    total_tokens: None,
                    error: Some(format!("candidate panicked: {e}")),
                });
            }
        }
        let ui_rows = candidates
            .iter()
            .enumerate()
            .map(|(idx, c)| BestOfNCandidateUi::from_candidate(idx, c, false))
            .chain((candidates.len()..total).map(|idx| BestOfNCandidateUi {
                index: idx,
                candidate_id: CandidateId::new(idx).to_string(),
                label: strategies[idx].label.clone(),
                status: "running".to_string(),
                file_count: 0,
                files: Vec::new(),
                error: None,
                recommended: false,
            }))
            .collect();
        emit_bon_progress(
            event_tx,
            BestOfNProgressPayload {
                run_id: run_id.to_string(),
                phase: BestOfNPhase::CandidateDone,
                message: format!("Candidate {}/{} complete…", i + 1, total),
                completed: i + 1,
                total,
                candidates: ui_rows,
                recommended_index: None,
                selection_reason: None,
            },
        );
    }

    emit_bon_progress(
        event_tx,
        BestOfNProgressPayload {
            run_id: run_id.to_string(),
            phase: BestOfNPhase::Selecting,
            message: "Selecting best candidate…".to_string(),
            completed: total,
            total,
            candidates: candidates
                .iter()
                .enumerate()
                .map(|(idx, c)| BestOfNCandidateUi::from_candidate(idx, c, false))
                .collect(),
            recommended_index: None,
            selection_reason: None,
        },
    );

    let selection = select_best_candidate(&candidates, &cfg.selector);
    let mut ui_candidates: Vec<BestOfNCandidateUi> = candidates
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            BestOfNCandidateUi::from_candidate(idx, c, idx == selection.winner_index)
        })
        .collect();

    let (winner_index, cancelled) = if matches!(cfg.mode, BestOfNMode::Show) {
        emit_bon_progress(
            event_tx,
            BestOfNProgressPayload {
                run_id: run_id.to_string(),
                phase: BestOfNPhase::AwaitingPick,
                message: format!(
                    "Recommended #{} — pick a winner or cancel",
                    selection.winner_index
                ),
                completed: total,
                total,
                candidates: ui_candidates.clone(),
                recommended_index: Some(selection.winner_index),
                selection_reason: Some(selection.reason.clone()),
            },
        );
        match await_user_pick(
            parent,
            &run_id,
            selection.winner_index,
            &selection.reason,
            &ui_candidates,
            event_tx,
        )
        .await
        {
            Ok(Some(idx)) => (idx, false),
            Ok(None) => {
                crate::tool::clear_best_of_n_handle();
                store.clear_run(&run_id);
                emit_bon_progress(
                    event_tx,
                    BestOfNProgressPayload {
                        run_id: run_id.to_string(),
                        phase: BestOfNPhase::Cancelled,
                        message: "Best-of-N cancelled — no files applied.".to_string(),
                        completed: total,
                        total,
                        candidates: ui_candidates,
                        recommended_index: Some(selection.winner_index),
                        selection_reason: Some(selection.reason.clone()),
                    },
                );
                return Ok(BestOfNRunResult {
                    run_id: run_id.to_string(),
                    winner_index: selection.winner_index,
                    candidates,
                    selection_reason: selection.reason,
                    files_applied: Vec::new(),
                    cancelled: true,
                });
            }
            Err(e) => {
                crate::logging::warn(&format!(
                    "[best-of-n] pick UI unavailable ({e}); falling back to auto recommendation"
                ));
                (selection.winner_index, false)
            }
        }
    } else {
        (selection.winner_index, false)
    };

    for row in &mut ui_candidates {
        row.recommended = row.index == winner_index;
    }

    emit_bon_progress(
        event_tx,
        BestOfNProgressPayload {
            run_id: run_id.to_string(),
            phase: BestOfNPhase::Applying,
            message: format!("Applying candidate #{winner_index}…"),
            completed: total,
            total,
            candidates: ui_candidates.clone(),
            recommended_index: Some(winner_index),
            selection_reason: Some(selection.reason.clone()),
        },
    );

    let files_applied = apply_winner(&run_id, &candidates, winner_index, &store);

    crate::tool::clear_best_of_n_handle();
    store.clear_run(&run_id);

    emit_bon_progress(
        event_tx,
        BestOfNProgressPayload {
            run_id: run_id.to_string(),
            phase: BestOfNPhase::Done,
            message: "Best-of-N done.".to_string(),
            completed: total,
            total,
            candidates: ui_candidates,
            recommended_index: Some(winner_index),
            selection_reason: Some(selection.reason.clone()),
        },
    );

    Ok(BestOfNRunResult {
        run_id: run_id.to_string(),
        winner_index,
        candidates,
        selection_reason: selection.reason,
        files_applied,
        cancelled,
    })
}

/// Wait for Face pick. Returns `Ok(Some(index))`, `Ok(None)` on cancel,
/// or `Err` when no Face bridge is wired.
async fn await_user_pick(
    parent: &Agent,
    run_id: &RunId,
    recommended_index: usize,
    selection_reason: &str,
    candidates: &[BestOfNCandidateUi],
    event_tx: &Option<tokio::sync::mpsc::UnboundedSender<ServerEvent>>,
) -> Result<Option<usize>> {
    let Some(tx) = parent.best_of_n_pick_tx.clone() else {
        return Err(anyhow::anyhow!("best_of_n_pick_tx not wired"));
    };

    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let request_id = format!("bon-pick-{run_id}");
    let candidates_json = serde_json::to_value(candidates)?;
    if tx
        .send(BestOfNPickInputRequest {
            request_id: request_id.clone(),
            session_id: parent.session.id.clone(),
            run_id: run_id.to_string(),
            tool_call_id: request_id,
            recommended_index,
            selection_reason: selection_reason.to_string(),
            candidates: candidates_json,
            response_tx,
        })
        .is_err()
    {
        return Err(anyhow::anyhow!("best_of_n pick channel closed"));
    }

    let _ = event_tx; // progress already emitted; pick goes via best_of_n_pick_tx only

    let response_value = response_rx
        .await
        .map_err(|_| anyhow::anyhow!("best_of_n pick response dropped"))?
        .map_err(|e| anyhow::anyhow!("best_of_n pick failed: {e}"))?;

    let ext: BestOfNPickExtResponse = serde_json::from_value(response_value)
        .map_err(|e| anyhow::anyhow!("invalid best_of_n pick response: {e}"))?;

    match ext {
        BestOfNPickExtResponse::Selected { index } => {
            if index >= candidates.len() {
                return Err(anyhow::anyhow!("pick index {index} out of range"));
            }
            Ok(Some(index))
        }
        BestOfNPickExtResponse::Cancelled => Ok(None),
    }
}

async fn spawn_and_run_candidate(
    run_id: &RunId,
    candidate_id: &CandidateId,
    strategy: &CandidateStrategy,
    prompt: &str,
    store: &Arc<ProposedContentStore>,
    provider: &Arc<dyn Provider>,
    registry: &Registry,
    parent_session: &Session,
) -> Result<CandidateDiff> {
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

    let mut child = Agent::new_with_session(
        Arc::clone(provider),
        registry.clone(),
        child_session,
        Some(allowed),
    );
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
        crate::agent::best_of_n_orchestrator::run_best_of_n(self, user_request, context_files).await
    }
}

fn emit_bon_progress(
    event_tx: &Option<tokio::sync::mpsc::UnboundedSender<ServerEvent>>,
    payload: BestOfNProgressPayload,
) {
    // Always log a human line for headless / TUI text consumers.
    crate::logging::info(&format!("[best-of-n] {}", payload.message));
    let Some(tx) = event_tx else {
        return;
    };
    let json = serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null);
    let _ = tx.send(ServerEvent::BestOfNProgress { payload: json });
    // Dual-write a short TextDelta so older clients still see activity.
    let _ = tx.send(ServerEvent::TextDelta {
        text: format!("[best-of-n] {}\n", payload.message),
    });
}

/// Format a user-facing summary for session history.
pub fn format_run_summary(result: &BestOfNRunResult) -> String {
    if result.cancelled {
        return format!(
            "BEST-OF-N CANCELLED\n\
             Candidates: {}\n\
             Recommended (not applied): #{}\n\
             Reason: {}\n",
            result.candidates.len(),
            result.winner_index,
            result.selection_reason,
        );
    }
    let winner = result
        .candidates
        .get(result.winner_index)
        .map(|c| c.strategy.label.as_str())
        .unwrap_or("unknown");
    let files = if result.files_applied.is_empty() {
        "  (none)".to_string()
    } else {
        result
            .files_applied
            .iter()
            .map(|f| format!("  - {f}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "BEST-OF-N MODE ENABLED!\n\
         Candidates: {}\n\
         Winner: #{} ({})\n\
         Reason: {}\n\
         Files applied:\n{}",
        result.candidates.len(),
        result.winner_index,
        winner,
        result.selection_reason,
        files,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_code_best_of_n::BestOfNPickExtResponse;

    #[test]
    fn format_run_summary_cancelled() {
        let summary = format_run_summary(&BestOfNRunResult {
            run_id: "r1".into(),
            winner_index: 1,
            candidates: Vec::new(),
            selection_reason: "focused".into(),
            files_applied: Vec::new(),
            cancelled: true,
        });
        assert!(summary.contains("CANCELLED"));
        assert!(summary.contains("not applied"));
    }

    #[test]
    fn pick_ext_selected_parses() {
        let v = serde_json::json!({ "outcome": "selected", "index": 2 });
        let ext: BestOfNPickExtResponse = serde_json::from_value(v).unwrap();
        assert_eq!(ext, BestOfNPickExtResponse::Selected { index: 2 });
    }
}
