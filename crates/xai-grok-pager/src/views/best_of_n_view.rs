//! Best-of-N Face UI helpers: progress reduce + scrollback card text.
//!
//! Pick UX reuses [`crate::views::question_view`] via the pager bridge
//! (`x.ai/ask_user_question`). This module owns live candidate-card progress
//! while a run is generating / awaiting pick / applying.

use next_code_best_of_n::{BestOfNPhase, BestOfNProgressPayload, format_progress_cards};
use serde::Deserialize;

use crate::app::agent_view::AgentView;
use crate::scrollback::block::RenderBlock;
use crate::scrollback::blocks::SystemMessageBlock;
use crate::scrollback::entry::EntryId;

/// Live BoN progress chrome attached to an [`AgentView`].
#[derive(Debug, Clone)]
pub struct BestOfNUiState {
    pub run_id: String,
    pub phase: BestOfNPhase,
    pub entry_id: Option<EntryId>,
    pub last_text: String,
}

/// Notification envelope (`sessionId` optional for routing).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BestOfNProgressNotification {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(flatten)]
    pub payload: BestOfNProgressPayload,
}

/// Apply a progress payload: upsert a system scrollback block with candidate cards.
pub fn apply_progress(agent: &mut AgentView, payload: BestOfNProgressPayload) -> bool {
    let text = format_progress_cards(&payload);
    let run_id = payload.run_id.clone();
    let phase = payload.phase;

    if let Some(state) = agent.best_of_n.as_mut()
        && state.run_id == run_id
        && let Some(id) = state.entry_id
        && let Some(entry) = agent.scrollback.get_by_id_mut(id)
        && let RenderBlock::System(sys) = &mut entry.block
    {
        sys.text = text.clone();
        entry.invalidate_cache();
        agent.scrollback.mark_height_dirty(id);
        state.phase = phase;
        state.last_text = text;
        if matches!(phase, BestOfNPhase::Done | BestOfNPhase::Cancelled) {
            agent.best_of_n = None;
        }
        return true;
    }

    let entry_id = agent
        .scrollback
        .push_block(RenderBlock::System(SystemMessageBlock::new(text.clone())));
    if matches!(phase, BestOfNPhase::Done | BestOfNPhase::Cancelled) {
        agent.best_of_n = None;
    } else {
        agent.best_of_n = Some(BestOfNUiState {
            run_id,
            phase,
            entry_id: Some(entry_id),
            last_text: text,
        });
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_code_best_of_n::BestOfNCandidateUi;

    fn payload(phase: BestOfNPhase) -> BestOfNProgressPayload {
        BestOfNProgressPayload {
            run_id: "run-1".into(),
            phase,
            message: "generating".into(),
            completed: 1,
            total: 2,
            candidates: vec![BestOfNCandidateUi {
                index: 0,
                candidate_id: "c0".into(),
                label: "temp-0".into(),
                status: "running".into(),
                file_count: 0,
                files: vec![],
                error: None,
                recommended: false,
            }],
            recommended_index: None,
            selection_reason: None,
        }
    }

    #[test]
    fn format_cards_changes_with_phase() {
        let a = format_progress_cards(&payload(BestOfNPhase::Generating));
        let b = format_progress_cards(&payload(BestOfNPhase::AwaitingPick));
        assert!(a.contains("generating"));
        assert!(b.contains("awaiting_pick"));
        assert!(a.contains("#0 temp-0"));
    }
}
