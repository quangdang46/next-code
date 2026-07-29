//! BestOfNBlock — renders codebuff-style candidate cards for Best-of-N runs.
//!
//! Each candidate is a bordered card with status dot, label, compact file stats,
//! and live progress colouring. The group header shows phase + progress.

use ratatui::prelude::*;

use crate::scrollback::block::BlockContent;
use crate::scrollback::types::{AccentStyle, BlockContext, BlockLine, BlockOutput, Selectable};
use crate::theme::Theme;

use next_code_best_of_n::{BestOfNCandidateUi, BestOfNPhase, BestOfNProgressPayload};

/// Dashed border chars used by codebuff-style proposal cards.
const CORNER_TL: &str = "┌";
const CORNER_TR: &str = "┐";
const CORNER_BL: &str = "└";
const CORNER_BR: &str = "┘";
const H_LINE: &str = "┈";
const V_LINE: &str = "┊";
const V_LINE_LEFT: &str = "┊ ";

/// A Best-of-N run block showing candidate cards.
#[derive(Debug, Clone)]
pub struct BestOfNBlock {
    pub run_id: String,
    pub phase: BestOfNPhase,
    pub message: String,
    pub completed: usize,
    pub total: usize,
    pub candidates: Vec<BestOfNCandidateUi>,
    pub selection_reason: Option<String>,
}

impl BestOfNBlock {
    pub fn new(payload: &BestOfNProgressPayload) -> Self {
        Self {
            run_id: payload.run_id.clone(),
            phase: payload.phase,
            message: payload.message.clone(),
            completed: payload.completed,
            total: payload.total,
            candidates: payload.candidates.clone(),
            selection_reason: payload.selection_reason.clone(),
        }
    }

    pub fn update(&mut self, payload: &BestOfNProgressPayload) {
        self.phase = payload.phase;
        self.message = payload.message.clone();
        self.completed = payload.completed;
        self.total = payload.total;
        self.candidates = payload.candidates.clone();
        self.selection_reason = payload.selection_reason.clone();
    }

    /// Phase indicator string and colour.
    fn phase_style(&self) -> (&'static str, Color) {
        match self.phase {
            BestOfNPhase::Generating => ("generating", Color::Yellow),
            BestOfNPhase::CandidateDone => ("candidates", Color::Green),
            BestOfNPhase::Selecting => ("selecting", Color::Cyan),
            BestOfNPhase::AwaitingPick => ("awaiting pick", Color::Magenta),
            BestOfNPhase::Applying => ("applying", Color::LightBlue),
            BestOfNPhase::Done => ("done", Color::Green),
            BestOfNPhase::Cancelled => ("cancelled", Color::Red),
        }
    }

    /// Status marker (dot/check/cross) and its colour.
    fn status_marker(status: &str) -> (&'static str, Color) {
        match status {
            "success" | "applied" => ("●", Color::Green),
            "running" | "generating" | "pending" => ("●", Color::Yellow),
            "failed" => ("✗", Color::Red),
            "no_changes" => ("○", Color::Gray),
            _ => ("?", Color::DarkGray),
        }
    }

    /// Empty-state label for a card with no files yet.
    fn empty_label(status: &str) -> &'static str {
        match status {
            "running" | "generating" | "pending" => "generating...",
            "success" | "applied" => "no changes",
            "failed" => "failed",
            "no_changes" => "no changes",
            _ => "—",
        }
    }

    /// Render the whole block output.
    fn render_cards(&self, content_width: usize) -> Vec<BlockLine> {
        let mut lines = Vec::new();
        let theme = Theme::current();
        let inner = content_width.saturating_sub(4).max(12); // 2 indent + 2 border pad

        // ── Header ──────────────────────────────────────────────
        let (phase_label, phase_color) = self.phase_style();
        let header = format!(
            "  Best-of-N [{phase}] {done}/{total} — {msg}",
            phase = phase_label,
            done = self.completed,
            total = self.total,
            msg = self.message,
        );
        lines.push(
            BlockLine::styled(
                Line::from(Span::styled(
                    header,
                    Style::default()
                        .fg(phase_color)
                        .add_modifier(Modifier::BOLD),
                )),
            )
            .with_selection_range(Some(0)),
        );

        // Selection reason
        if let Some(reason) = &self.selection_reason {
            lines.push(
                BlockLine::styled(
                    Line::from(Span::styled(
                        format!("  selector: {reason}"),
                        Style::default().fg(theme.gray_dim),
                    )),
                )
                .with_selection_range(Some(0)),
            );
        }

        // ── Candidate cards ─────────────────────────────────────
        for c in &self.candidates {
            let card_w = inner.min(80); // cap card width so side-by-side fits
            let (marker, marker_color) = Self::status_marker(&c.status);
            let rec_prefix = if c.recommended { "★ " } else { "  " };

            // Top border
            let top = format!("  {tl}{h}{tr}", tl = CORNER_TL, h = H_LINE.repeat(card_w), tr = CORNER_TR);
            lines.push(
                BlockLine::styled(
                    Line::from(Span::styled(top, Style::default().fg(theme.gray_dim))),
                )
                .with_selection_range(None),
            );

            // Title row: │ ● #N label [status]
            let title = Line::from(vec![
                Span::styled(V_LINE_LEFT, Style::default().fg(theme.gray_dim)),
                Span::styled(rec_prefix.to_string(), Style::default().fg(Color::Yellow)),
                Span::styled(marker, Style::default().fg(marker_color)),
                Span::styled(
                    format!(" #{idx} {label}  [{status}]", idx = c.index, label = c.label, status = c.status),
                    Style::default().fg(theme.text_primary),
                ),
            ]);
            lines.push(
                BlockLine::styled(title).with_selection_range(Some(0)),
            );

            // File stats or empty-state
            if c.files.is_empty() {
                let empty = Self::empty_label(&c.status);
                lines.push(
                    BlockLine::styled(
                        Line::from(vec![
                            Span::styled(V_LINE_LEFT, Style::default().fg(theme.gray_dim)),
                            Span::styled(format!("  {empty}"), Style::default().fg(theme.gray_dim).add_modifier(Modifier::ITALIC)),
                        ]),
                    )
                    .with_selection_range(None),
                );
            } else {
                for f in &c.files {
                    let file_line = Line::from(vec![
                        Span::styled(V_LINE_LEFT, Style::default().fg(theme.gray_dim)),
                        Span::styled(" M ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::styled(f.clone(), Style::default().fg(theme.text_primary)),
                    ]);
                    lines.push(
                        BlockLine::styled(file_line).with_selection_range(Some(0)),
                    );
                }
            }

            // Error row
            if let Some(err) = &c.error {
                lines.push(
                    BlockLine::styled(
                        Line::from(vec![
                            Span::styled(V_LINE_LEFT, Style::default().fg(theme.gray_dim)),
                            Span::styled(format!("  error: {err}"), Style::default().fg(Color::Red)),
                        ]),
                    )
                    .with_selection_range(Some(0)),
                );
            }

            // Bottom border
            let bottom = format!("  {bl}{h}{br}", bl = CORNER_BL, h = H_LINE.repeat(card_w), br = CORNER_BR);
            lines.push(
                BlockLine::styled(
                    Line::from(Span::styled(bottom, Style::default().fg(theme.gray_dim))),
                )
                .with_selection_range(None),
            );
        }

        lines
    }
}

impl BlockContent for BestOfNBlock {
    fn output(&self, ctx: &BlockContext) -> BlockOutput {
        BlockOutput {
            lines: self.render_cards(ctx.width as usize),
        }
    }

    fn accent(&self, _ctx: &BlockContext) -> Option<AccentStyle> {
        Some(AccentStyle::static_color(Color::Cyan))
    }

    fn has_vpad(&self, _ctx: &BlockContext) -> bool {
        true
    }

    fn has_raw_mode(&self) -> bool {
        false
    }

    fn is_foldable(&self) -> bool {
        true
    }

    fn is_selectable(&self) -> bool {
        true
    }

    fn is_groupable(&self) -> bool {
        true
    }
}
