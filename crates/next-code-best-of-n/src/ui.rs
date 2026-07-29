//! Face / ACP UI payload types for Best-of-N runs.
//!
//! Shared between the daemon orchestrator, wire protocol, and Face pager so
//! candidate cards / pick requests stay in sync.
//!
//! Visual/copy patterns follow Codebuff CLI:
//! - `cli/src/components/blocks/implementor-row.tsx` (ImplementorCard)
//! - `cli/src/utils/implementor-helpers.ts` (`getMultiPromptPreview`, file ± stats)
//! - `cli/src/utils/agent-helpers.ts` (status text)

use serde::{Deserialize, Serialize};

use crate::types::{CandidateDiff, CandidateStatus, FileDiff};

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

/// Per-file compact stats (Codebuff `FileStats` / CompactFileRow).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BestOfNFileStat {
    pub path: String,
    /// `A` (added), `M` (modified), `D` (deleted).
    pub change_type: String,
    pub lines_added: usize,
    pub lines_removed: usize,
}

impl BestOfNFileStat {
    pub fn from_file_diff(f: &FileDiff) -> Self {
        let (mut lines_added, mut lines_removed) = parse_diff_stats(&f.unified_diff);
        // Fallback when unified_diff is empty but content differs.
        if lines_added == 0 && lines_removed == 0 && f.has_changes() {
            if f.is_new_file || f.old_content.is_empty() {
                lines_added = f.new_content.lines().count().max(1);
            } else if f.new_content.is_empty() {
                lines_removed = f.old_content.lines().count().max(1);
            } else {
                lines_added = f.new_content.lines().count();
                lines_removed = f.old_content.lines().count();
            }
        }
        let change_type = if f.is_new_file || f.old_content.is_empty() {
            "A"
        } else if f.new_content.is_empty() {
            "D"
        } else {
            "M"
        };
        Self {
            path: f.file_path.clone(),
            change_type: change_type.to_string(),
            lines_added,
            lines_removed,
        }
    }

    /// Compact `+N/-M` suffix for pick descriptions.
    pub fn plus_minus(&self) -> String {
        format!("+{}/-{}", self.lines_added, self.lines_removed)
    }
}

/// Parse unified-diff style `+/-` line counts (Codebuff `parseDiffStats`).
pub fn parse_diff_stats(diff: &str) -> (usize, usize) {
    let mut added = 0usize;
    let mut removed = 0usize;
    for line in diff.lines() {
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if line.starts_with('+') {
            added += 1;
        } else if line.starts_with('-') {
            removed += 1;
        }
    }
    (added, removed)
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
    /// Path list (legacy / simple previews). Prefer [`Self::file_stats`].
    #[serde(default)]
    pub files: Vec<String>,
    /// Codebuff-style per-file ± stats.
    #[serde(default)]
    pub file_stats: Vec<BestOfNFileStat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// True when this is the deterministic selector's recommendation.
    #[serde(default)]
    pub recommended: bool,
}

impl BestOfNCandidateUi {
    /// Running placeholder before a candidate finishes (Codebuff empty card).
    pub fn pending(index: usize, candidate_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            index,
            candidate_id: candidate_id.into(),
            label: label.into(),
            status: "running".to_string(),
            file_count: 0,
            files: Vec::new(),
            file_stats: Vec::new(),
            error: None,
            recommended: false,
        }
    }

    pub fn from_candidate(index: usize, c: &CandidateDiff, recommended: bool) -> Self {
        let status = match c.status {
            CandidateStatus::Success => "success",
            CandidateStatus::NoChanges => "no_changes",
            CandidateStatus::Failed => "failed",
        };
        let file_stats: Vec<BestOfNFileStat> = c
            .file_diffs
            .iter()
            .filter(|f| f.has_changes() || f.is_new_file)
            .map(BestOfNFileStat::from_file_diff)
            .collect();
        let files: Vec<String> = file_stats.iter().map(|f| f.path.clone()).collect();
        Self {
            index,
            candidate_id: c.candidate_id.to_string(),
            label: c.strategy.label.clone(),
            status: status.to_string(),
            file_count: files.len(),
            files,
            file_stats,
            error: c.error.clone(),
            recommended,
        }
    }

    /// Codebuff-like display name: `Proposal #N` (1-based).
    pub fn display_name(&self) -> String {
        format!("Proposal #{}", self.index + 1)
    }

    /// Status chrome matching Codebuff `getAgentStatusInfo` text form.
    ///
    /// Running → `● running`; complete → `completed ✓`; failed → `✗ failed`.
    pub fn status_text(&self) -> (&'static str, &'static str) {
        match self.status.as_str() {
            "running" | "generating" | "pending" => ("● running", "running"),
            "success" | "applied" | "complete" | "completed" => ("completed ✓", "complete"),
            "failed" => ("✗ failed", "failed"),
            "cancelled" => ("⊘ cancelled", "cancelled"),
            "no_changes" => ("completed ✓", "complete"),
            _ => ("○ waiting", "waiting"),
        }
    }

    /// Empty-state label when the card has no file stats yet.
    pub fn empty_label(&self) -> &'static str {
        match self.status.as_str() {
            "running" | "generating" | "pending" => "generating...",
            "failed" => "failed",
            "cancelled" => "cancelled",
            "success" | "applied" | "complete" | "completed" | "no_changes" => "no changes",
            _ => "—",
        }
    }

    /// One-line summary for AskUserQuestion-style option descriptions.
    pub fn option_description(&self) -> String {
        let mut parts = Vec::new();
        let (status_text, _) = self.status_text();
        parts.push(status_text.to_string());
        if self.file_count == 0 {
            parts.push(self.empty_label().to_string());
        } else {
            let stats = if self.file_stats.is_empty() {
                self.files
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                self.file_stats
                    .iter()
                    .take(3)
                    .map(|f| format!("{} {} {}", f.change_type, f.path, f.plus_minus()))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let more = if self.file_count > 3 {
                format!(" (+{} more)", self.file_count - 3)
            } else {
                String::new()
            };
            parts.push(format!("{} file(s): {stats}{more}", self.file_count));
        }
        if let Some(err) = &self.error {
            parts.push(format!("error: {err}"));
        }
        parts.join(" · ")
    }

    pub fn option_label(&self) -> String {
        let mut label = self.display_name();
        if !self.label.is_empty() && self.label != "default" {
            label.push_str(&format!(" ({})", self.label));
        }
        if self.recommended {
            label.push_str(" ★ Recommended");
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

/// Codebuff `getMultiPromptPreview`-shaped progress copy.
///
/// Examples: `Generating 3 proposals...`, `2/3 proposals complete...`,
/// `3 proposals complete • Selecting best...`, `Applying selected changes...`.
pub fn progress_preview(
    phase: BestOfNPhase,
    completed: usize,
    total: usize,
    failed: usize,
    selection_reason: Option<&str>,
) -> String {
    match phase {
        BestOfNPhase::Generating => {
            if completed == 0 {
                format!("Generating {total} proposals...")
            } else if failed > 0 {
                format!("{completed}/{total} complete, {failed} failed...")
            } else {
                format!("{completed}/{total} proposals complete...")
            }
        }
        BestOfNPhase::CandidateDone => {
            if completed >= total {
                if failed > 0 {
                    format!("{}/{total} proposals complete ({failed} failed)", completed - failed)
                } else {
                    format!("{total} proposals complete")
                }
            } else if failed > 0 {
                format!("{completed}/{total} complete, {failed} failed...")
            } else {
                format!("{completed}/{total} proposals complete...")
            }
        }
        BestOfNPhase::Selecting => {
            format!("{total} proposals complete • Selecting best...")
        }
        BestOfNPhase::AwaitingPick => {
            format!("{total} proposals complete • Pick a winner")
        }
        BestOfNPhase::Applying => "Applying selected changes...".to_string(),
        BestOfNPhase::Done => {
            if let Some(reason) = selection_reason.filter(|r| !r.is_empty()) {
                let formatted = {
                    let mut c = reason.chars();
                    match c.next() {
                        Some(first) => {
                            first.to_uppercase().collect::<String>() + c.as_str()
                        }
                        None => String::new(),
                    }
                };
                format!("{total} proposals evaluated\n{formatted}")
            } else {
                format!("{total} proposals evaluated")
            }
        }
        BestOfNPhase::Cancelled => "Best-of-N cancelled — no files applied.".to_string(),
    }
}

/// Format progress payload as scrollback candidate cards (Codebuff-like rows).
pub fn format_progress_cards(payload: &BestOfNProgressPayload) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Best-of-N — {message}",
        message = payload.message,
    ));
    if let Some(reason) = &payload.selection_reason {
        lines.push(format!("  selector: {reason}"));
    }
    for c in &payload.candidates {
        let mark = if c.recommended { "★ " } else { "  " };
        let (status_text, _) = c.status_text();
        let files = if c.file_count == 0 {
            c.empty_label().to_string()
        } else if !c.file_stats.is_empty() {
            c.file_stats
                .iter()
                .take(3)
                .map(|f| format!("{} {} {}", f.change_type, f.path, f.plus_minus()))
                .collect::<Vec<_>>()
                .join(", ")
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
            "{mark}{name}  {status}  {files}{err}",
            name = c.display_name(),
            status = status_text,
        ));
    }
    lines.join("\n")
}

/// Map an AskUserQuestion-style accepted option label back to a candidate index.
///
/// Labels are produced by [`BestOfNCandidateUi::option_label`].
pub fn index_from_option_label(label: &str, candidates: &[BestOfNCandidateUi]) -> Option<usize> {
    let trimmed = label.trim();
    for c in candidates {
        if c.option_label() == trimmed
            || trimmed == c.display_name()
            || trimmed.starts_with(&format!("Proposal #{}", c.index + 1))
            || trimmed.starts_with(&format!("#{} ", c.index))
        {
            return Some(c.index);
        }
    }
    // Fallback: "Proposal #N" or leading `#N`
    if let Some(rest) = trimmed.strip_prefix("Proposal #") {
        let num: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
        if let Ok(n) = num.parse::<usize>() {
            if n >= 1 {
                return Some(n - 1);
            }
        }
    }
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
                unified_diff: "@@\n-old\n+new\n+line2\n".into(),
                old_content: "old\n".into(),
                new_content: "new\nline2\n".into(),
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
        assert_eq!(ui.file_stats[0].lines_added, 2);
        assert_eq!(ui.file_stats[0].lines_removed, 1);
        assert_eq!(ui.file_stats[0].change_type, "M");
        assert!(ui.option_label().contains("Recommended"));
        assert!(ui.option_label().contains("Proposal #1"));
        assert!(ui.option_description().contains("src/f0.rs"));
        assert_eq!(ui.status_text().0, "completed ✓");
    }

    #[test]
    fn pending_shows_generating_empty() {
        let ui = BestOfNCandidateUi::pending(2, "c2", "temp-2");
        assert_eq!(ui.empty_label(), "generating...");
        assert_eq!(ui.status_text().0, "● running");
        assert_eq!(ui.display_name(), "Proposal #3");
    }

    #[test]
    fn progress_preview_matches_codebuff_copy() {
        assert_eq!(
            progress_preview(BestOfNPhase::Generating, 0, 3, 0, None),
            "Generating 3 proposals..."
        );
        assert_eq!(
            progress_preview(BestOfNPhase::CandidateDone, 2, 3, 0, None),
            "2/3 proposals complete..."
        );
        assert_eq!(
            progress_preview(BestOfNPhase::Selecting, 3, 3, 0, None),
            "3 proposals complete • Selecting best..."
        );
        assert_eq!(
            progress_preview(BestOfNPhase::Applying, 3, 3, 0, None),
            "Applying selected changes..."
        );
        assert!(
            progress_preview(BestOfNPhase::Done, 3, 3, 0, Some("focused edit"))
                .starts_with("3 proposals evaluated")
        );
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
            message: progress_preview(BestOfNPhase::Generating, 0, 1, 0, None),
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
        let text = format_progress_cards(&payload);
        assert!(text.contains("Generating 1 proposals"));
        assert!(text.contains("Proposal #1"));
        assert!(text.contains("completed ✓"));
    }

    #[test]
    fn index_from_option_label_reads_proposal() {
        let c = BestOfNCandidateUi::from_candidate(
            3,
            &sample_candidate(3, CandidateStatus::Success),
            true,
        );
        assert_eq!(
            index_from_option_label(&c.option_label(), std::slice::from_ref(&c)),
            Some(3)
        );
        assert_eq!(
            index_from_option_label("Proposal #4", std::slice::from_ref(&c)),
            Some(3)
        );
    }

    #[test]
    fn parse_diff_stats_skips_headers() {
        let diff = "--- a/f\n+++ b/f\n@@ -1 +1 @@\n-old\n+new\n";
        assert_eq!(parse_diff_stats(diff), (1, 1));
    }
}
