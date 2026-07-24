//! Face / ACP UI payload types for Best-of-N runs.
//!
//! Shared between the daemon orchestrator, wire protocol, and Face pager so
//! candidate cards / pick requests stay in sync.

use serde::{Deserialize, Serialize};

use crate::types::{CandidateDiff, CandidateStatus};

/// High-level phase of a Best-of-N run (Face progress chrome).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BestOfNPhase {
    /// Fan-out started; candidates still running.
    Generating,
    /// One or more candidates finished (partial or all).
    CandidateDone,
    /// Deterministic selector ran (auto recommendation ready).
    Selecting,
    /// `mode=show`: waiting for user pick / cancel.
    AwaitingPick,
    /// Applying the chosen winner to disk.
    Applying,
    /// Run finished successfully.
    Done,
    /// User cancelled or run aborted without apply.
    Cancelled,
}

impl BestOfNPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Generating => "generating",
            Self::CandidateDone => "candidate_done",
            Self::Selecting => "selecting",
            Self::AwaitingPick => "awaiting_pick",
            Self::Applying => "applying",
            Self::Done => "done",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Compact per-candidate row for Face cards / pick options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BestOfNCandidateUi {
    pub index: usize,
    pub candidate_id: String,
    pub label: String,
    pub status: String,
    pub file_count: usize,
    pub files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// True when this is the deterministic selector's recommendation.
    #[serde(default)]
    pub recommended: bool,
}

impl BestOfNCandidateUi {
    pub fn from_candidate(index: usize, c: &CandidateDiff, recommended: bool) -> Self {
        let status = match c.status {
            CandidateStatus::Success => "success",
            CandidateStatus::NoChanges => "no_changes",
            CandidateStatus::Failed => "failed",
        };
        let files: Vec<String> = c
            .file_diffs
            .iter()
            .map(|f| f.file_path.clone())
            .collect();
        Self {
            index,
            candidate_id: c.candidate_id.to_string(),
            label: c.strategy.label.clone(),
            status: status.to_string(),
            file_count: files.len(),
            files,
            error: c.error.clone(),
            recommended,
        }
    }

    /// One-line summary for AskUserQuestion-style option descriptions.
    pub fn option_description(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("status={}", self.status));
        if self.file_count == 0 {
            parts.push("no file changes".to_string());
        } else {
            let preview = self
                .files
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let more = if self.files.len() > 3 {
                format!(" (+{} more)", self.files.len() - 3)
            } else {
                String::new()
            };
            parts.push(format!("{} file(s): {preview}{more}", self.file_count));
        }
        if let Some(err) = &self.error {
            parts.push(format!("error: {err}"));
        }
        parts.join(" · ")
    }

    pub fn option_label(&self) -> String {
        let mut label = format!("#{} {}", self.index, self.label);
        if self.recommended {
            label.push_str(" (Recommended)");
        }
        label
    }
}

/// Progress payload emitted during a BoN run (and as pick-request body).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BestOfNProgressPayload {
    pub run_id: String,
    pub phase: BestOfNPhase,
    pub message: String,
    pub completed: usize,
    pub total: usize,
    pub candidates: Vec<BestOfNCandidateUi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_reason: Option<String>,
}

/// ACP `x.ai/best_of_n/pick` request (daemon → Face).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BestOfNPickExtRequest {
    pub session_id: String,
    pub run_id: String,
    pub tool_call_id: String,
    pub recommended_index: usize,
    pub selection_reason: String,
    pub candidates: Vec<BestOfNCandidateUi>,
}

/// ACP `x.ai/best_of_n/pick` response (Face → daemon).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BestOfNPickExtResponse {
    Selected { index: usize },
    Cancelled,
}

/// Format progress payload as scrollback candidate cards (Codebuff-like rows).
pub fn format_progress_cards(payload: &BestOfNProgressPayload) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Best-of-N [{phase}] {completed}/{total} — {message}",
        phase = payload.phase.as_str(),
        completed = payload.completed,
        total = payload.total,
        message = payload.message,
    ));
    if let Some(reason) = &payload.selection_reason {
        lines.push(format!("  selector: {reason}"));
    }
    for c in &payload.candidates {
        let mark = if c.recommended { "★ " } else { "  " };
        let files = if c.file_count == 0 {
            "no file changes".to_string()
        } else {
            let preview = c
                .files
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let more = if c.files.len() > 3 {
                format!(" (+{} more)", c.files.len() - 3)
            } else {
                String::new()
            };
            format!("{} file(s): {preview}{more}", c.file_count)
        };
        let err = c
            .error
            .as_ref()
            .map(|e| format!(" · error: {e}"))
            .unwrap_or_default();
        lines.push(format!(
            "{mark}#{idx} {label}  [{status}]  {files}{err}",
            idx = c.index,
            label = c.label,
            status = c.status,
        ));
    }
    lines.join("\n")
}

/// Map an AskUserQuestion-style accepted option label back to a candidate index.
///
/// Labels are produced by [`BestOfNCandidateUi::option_label`] (`#N …`).
pub fn index_from_option_label(label: &str, candidates: &[BestOfNCandidateUi]) -> Option<usize> {
    let trimmed = label.trim();
    for c in candidates {
        if c.option_label() == trimmed || trimmed.starts_with(&format!("#{} ", c.index)) {
            return Some(c.index);
        }
    }
    // Fallback: parse leading `#N`
    let rest = trimmed.strip_prefix('#')?;
    let num: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    num.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CandidateId, CandidateStrategy, FileDiff};

    fn sample_candidate(i: usize, status: CandidateStatus) -> CandidateDiff {
        CandidateDiff {
            candidate_id: CandidateId::new(i),
            strategy: CandidateStrategy {
                label: format!("temp-{i}"),
                temperature: 0.5,
                model: None,
            },
            status,
            file_diffs: vec![FileDiff {
                file_path: format!("src/f{i}.rs"),
                unified_diff: String::new(),
                old_content: String::new(),
                new_content: "x".into(),
                is_new_file: false,
            }],
            total_tokens: None,
            error: None,
        }
    }

    #[test]
    fn candidate_ui_marks_recommended_and_files() {
        let c = sample_candidate(0, CandidateStatus::Success);
        let ui = BestOfNCandidateUi::from_candidate(0, &c, true);
        assert!(ui.recommended);
        assert_eq!(ui.file_count, 1);
        assert!(ui.option_label().contains("Recommended"));
        assert!(ui.option_description().contains("src/f0.rs"));
    }

    #[test]
    fn pick_response_round_trips() {
        let selected = BestOfNPickExtResponse::Selected { index: 2 };
        let json = serde_json::to_string(&selected).unwrap();
        let back: BestOfNPickExtResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back, BestOfNPickExtResponse::Selected { index: 2 });

        let cancelled = BestOfNPickExtResponse::Cancelled;
        let json = serde_json::to_string(&cancelled).unwrap();
        let back: BestOfNPickExtResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back, BestOfNPickExtResponse::Cancelled);
    }

    #[test]
    fn format_progress_cards_lists_candidates() {
        let payload = BestOfNProgressPayload {
            run_id: "r".into(),
            phase: BestOfNPhase::Generating,
            message: "go".into(),
            completed: 0,
            total: 1,
            candidates: vec![BestOfNCandidateUi::from_candidate(
                0,
                &sample_candidate(0, CandidateStatus::Success),
                false,
            )],
            recommended_index: None,
            selection_reason: None,
        };
        // Override status via from_candidate — sample is Success.
        let text = format_progress_cards(&payload);
        assert!(text.contains("generating"));
        assert!(text.contains("#0"));
    }

    #[test]
    fn index_from_option_label_reads_hash() {
        let c = BestOfNCandidateUi::from_candidate(
            3,
            &sample_candidate(3, CandidateStatus::Success),
            true,
        );
        assert_eq!(
            index_from_option_label(&c.option_label(), std::slice::from_ref(&c)),
            Some(3)
        );
    }
}
