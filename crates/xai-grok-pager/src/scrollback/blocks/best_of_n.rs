//! BestOfNBlock — renders Codebuff-style ImplementorCard candidates for BoN runs.
//!
//! Reference: `.tmp/codebuff/cli/src/components/blocks/implementor-row.tsx`
//! - Rounded dashed borders (`PROPOSAL_BORDER_CHARS`: ╭╮╰╯ ┈┊)
//! - Header: bold display name + dim status (`● running` / `completed ✓`)
//! - File rows: change type + path + green/red ± bars
//! - Empty states: `generating...` / `no changes` / `failed` / `cancelled`
//! - Masonry-ish 2-column layout when the viewport is wide enough

use ratatui::prelude::*;

use crate::scrollback::block::BlockContent;
use crate::scrollback::types::{AccentStyle, BlockContext, BlockLine, BlockOutput};
use crate::theme::Theme;

use next_code_best_of_n::{BestOfNCandidateUi, BestOfNFileStat, BestOfNPhase, BestOfNProgressPayload};

/// Codebuff `PROPOSAL_BORDER_CHARS` (rounded + dashed).
const CORNER_TL: &str = "╭";
const CORNER_TR: &str = "╮";
const CORNER_BL: &str = "╰";
const CORNER_BR: &str = "╯";
const H_LINE: &str = "┈";
const V_LINE: &str = "┊";

/// Fixed ± bar segment width (Codebuff `STATS_BAR_WIDTH`).
const STATS_BAR_WIDTH: usize = 5;

/// Minimum card inner width.
const MIN_CARD_INNER: usize = 16;

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

    fn phase_color(&self) -> Color {
        match self.phase {
            BestOfNPhase::Generating | BestOfNPhase::CandidateDone => Color::Yellow,
            BestOfNPhase::Selecting => Color::Cyan,
            BestOfNPhase::AwaitingPick => Color::Magenta,
            BestOfNPhase::Applying => Color::LightBlue,
            BestOfNPhase::Done => Color::Green,
            BestOfNPhase::Cancelled => Color::Red,
        }
    }

    fn status_color(kind: &str) -> Color {
        match kind {
            "running" => Color::Yellow,
            "complete" => Color::Green,
            "failed" | "cancelled" => Color::Red,
            _ => Color::DarkGray,
        }
    }

    /// Render one ImplementorCard as a list of full-width (or card-width) lines.
    fn render_card(c: &BestOfNCandidateUi, card_w: usize, theme: &Theme) -> Vec<Line<'static>> {
        let inner = card_w.saturating_sub(2).max(MIN_CARD_INNER);
        let border_fg = if matches!(
            c.status.as_str(),
            "success" | "applied" | "complete" | "completed" | "no_changes"
        ) {
            theme.gray_dim
        } else {
            Color::Cyan
        };

        let mut out = Vec::new();

        // Top border
        out.push(Line::from(Span::styled(
            format!(
                "{tl}{h}{tr}",
                tl = CORNER_TL,
                h = H_LINE.repeat(inner),
                tr = CORNER_TR
            ),
            Style::default().fg(border_fg),
        )));

        // Header: ★? Proposal #N   ● running
        let (status_text, status_kind) = c.status_text();
        let rec = if c.recommended { "★ " } else { "" };
        let name = c.display_name();
        let mut header_spans = vec![
            Span::styled(format!("{V_LINE} "), Style::default().fg(border_fg)),
            Span::styled(
                format!("{rec}{name}"),
                Style::default()
                    .fg(theme.text_primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                status_text.to_string(),
                Style::default()
                    .fg(Self::status_color(status_kind))
                    .add_modifier(Modifier::DIM),
            ),
        ];
        // Pad remaining width with spaces then close border (best-effort).
        let used = 2 + rec.chars().count() + name.chars().count() + 1 + status_text.chars().count();
        if used < inner {
            header_spans.push(Span::raw(" ".repeat(inner - used)));
        }
        header_spans.push(Span::styled(V_LINE.to_string(), Style::default().fg(border_fg)));
        out.push(Line::from(header_spans));

        // Strategy subtitle (Codebuff initialPrompt italic)
        if !c.label.is_empty() && c.label != "default" {
            let strategy = truncate_ellipsis(&c.label, inner.saturating_sub(2));
            let used = 2 + strategy.chars().count();
            let mut spans = vec![
                Span::styled(format!("{V_LINE} "), Style::default().fg(border_fg)),
                Span::styled(
                    strategy,
                    Style::default()
                        .fg(theme.gray_dim)
                        .add_modifier(Modifier::ITALIC),
                ),
            ];
            if used < inner {
                spans.push(Span::raw(" ".repeat(inner - used)));
            }
            spans.push(Span::styled(V_LINE.to_string(), Style::default().fg(border_fg)));
            out.push(Line::from(spans));
        }

        let stats = if !c.file_stats.is_empty() {
            c.file_stats.clone()
        } else {
            c.files
                .iter()
                .map(|p| BestOfNFileStat {
                    path: p.clone(),
                    change_type: "M".into(),
                    lines_added: 0,
                    lines_removed: 0,
                })
                .collect()
        };

        if stats.is_empty() {
            let empty = c.empty_label();
            let used = 2 + empty.chars().count();
            let mut spans = vec![
                Span::styled(format!("{V_LINE} "), Style::default().fg(border_fg)),
                Span::styled(
                    empty.to_string(),
                    Style::default()
                        .fg(theme.gray_dim)
                        .add_modifier(Modifier::ITALIC),
                ),
            ];
            if used < inner {
                spans.push(Span::raw(" ".repeat(inner - used)));
            }
            spans.push(Span::styled(V_LINE.to_string(), Style::default().fg(border_fg)));
            out.push(Line::from(spans));
        } else {
            let max_added_w = stats
                .iter()
                .map(|f| format!("+{}", f.lines_added).len())
                .max()
                .unwrap_or(2)
                .max(2);
            let max_removed_w = stats
                .iter()
                .map(|f| format!("-{}", f.lines_removed).len())
                .max()
                .unwrap_or(2)
                .max(2);

            for f in &stats {
                out.push(Self::file_row(
                    f,
                    inner,
                    max_added_w,
                    max_removed_w,
                    border_fg,
                    theme,
                ));
            }
        }

        if let Some(err) = &c.error {
            let msg = truncate_ellipsis(&format!("error: {err}"), inner.saturating_sub(2));
            let used = 2 + msg.chars().count();
            let mut spans = vec![
                Span::styled(format!("{V_LINE} "), Style::default().fg(border_fg)),
                Span::styled(msg, Style::default().fg(Color::Red)),
            ];
            if used < inner {
                spans.push(Span::raw(" ".repeat(inner - used)));
            }
            spans.push(Span::styled(V_LINE.to_string(), Style::default().fg(border_fg)));
            out.push(Line::from(spans));
        }

        // Bottom border
        out.push(Line::from(Span::styled(
            format!(
                "{bl}{h}{br}",
                bl = CORNER_BL,
                h = H_LINE.repeat(inner),
                br = CORNER_BR
            ),
            Style::default().fg(border_fg),
        )));

        out
    }

    /// Codebuff CompactFileRow: `M path   [+N ][-M ]` with green/red bars.
    fn file_row(
        f: &BestOfNFileStat,
        inner: usize,
        max_added_w: usize,
        max_removed_w: usize,
        border_fg: Color,
        theme: &Theme,
    ) -> Line<'static> {
        let added_section_w = STATS_BAR_WIDTH + max_added_w;
        let removed_section_w = STATS_BAR_WIDTH + max_removed_w;
        let bar_w = added_section_w + removed_section_w;
        // Layout: "┊ " + change(1) + " " + path + " " + bars + "┊"
        let fixed = 2 + 1 + 1 + 1 + bar_w + 1;
        let path_w = inner.saturating_sub(fixed).max(6);
        let path = truncate_ellipsis(&f.path, path_w);

        let added_str = format!("+{}", f.lines_added);
        let removed_str = format!("-{}", f.lines_removed);
        let added_content = format!("{} ", added_str);
        let added_content = format!("{added_content:>width$}", width = added_section_w);
        let removed_content = format!(" {removed_str}");
        let removed_content = format!("{removed_content:<width$}", width = removed_section_w);

        let change_color = match f.change_type.as_str() {
            "A" => Color::Green,
            "D" => Color::Red,
            _ => Color::Green,
        };

        Line::from(vec![
            Span::styled(format!("{V_LINE} "), Style::default().fg(border_fg)),
            Span::styled(
                f.change_type.clone(),
                Style::default()
                    .fg(change_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(path, Style::default().fg(theme.text_primary)),
            Span::raw(" "),
            Span::styled(
                added_content,
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(58, 90, 58)),
            ),
            Span::styled(
                removed_content,
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(90, 58, 58)),
            ),
            Span::styled(V_LINE.to_string(), Style::default().fg(border_fg)),
        ])
    }

    /// Pad a card's lines to `height` with blank bordered rows.
    fn pad_card(mut lines: Vec<Line<'static>>, height: usize, card_w: usize, dim: Color) -> Vec<Line<'static>> {
        let inner = card_w.saturating_sub(2).max(MIN_CARD_INNER);
        while lines.len() < height {
            // Insert blank body rows before the last (bottom border) if present.
            let blank = Line::from(vec![
                Span::styled(format!("{V_LINE} "), Style::default().fg(dim)),
                Span::raw(" ".repeat(inner.saturating_sub(1))),
                Span::styled(V_LINE.to_string(), Style::default().fg(dim)),
            ]);
            if lines.len() >= 2 {
                lines.insert(lines.len() - 1, blank);
            } else {
                lines.push(blank);
            }
        }
        lines
    }

    fn render_cards(&self, content_width: usize) -> Vec<BlockLine> {
        let mut lines = Vec::new();
        let theme = Theme::current();
        let available = content_width.saturating_sub(2).max(20);

        // ── Header (Codebuff multi-prompt preview copy) ─────────
        let header = format!("  {}", self.message);
        lines.push(
            BlockLine::styled(Line::from(Span::styled(
                header,
                Style::default()
                    .fg(self.phase_color())
                    .add_modifier(Modifier::BOLD),
            )))
            .with_selection_range(Some(0)),
        );

        if let Some(reason) = &self.selection_reason {
            let formatted = {
                let mut chars = reason.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            };
            lines.push(
                BlockLine::styled(Line::from(Span::styled(
                    format!("  {formatted}"),
                    Style::default().fg(theme.gray_dim),
                )))
                .with_selection_range(Some(0)),
            );
        }

        if self.candidates.is_empty() {
            lines.push(
                BlockLine::styled(Line::from(Span::styled(
                    "  generating...".to_string(),
                    Style::default()
                        .fg(theme.gray_dim)
                        .add_modifier(Modifier::ITALIC),
                )))
                .with_selection_range(None),
            );
            return lines;
        }

        // Masonry: 2 columns when wide enough (Codebuff ImplementorGroup).
        let cols = if available >= 72 && self.candidates.len() >= 2 {
            2
        } else {
            1
        };
        let gap = if cols > 1 { 1 } else { 0 };
        let card_w = ((available - gap * (cols - 1)) / cols).max(MIN_CARD_INNER + 2);

        for chunk in self.candidates.chunks(cols) {
            let mut card_lines: Vec<Vec<Line<'static>>> = chunk
                .iter()
                .map(|c| Self::render_card(c, card_w, &theme))
                .collect();
            let max_h = card_lines.iter().map(|c| c.len()).max().unwrap_or(0);
            for card in &mut card_lines {
                *card = Self::pad_card(std::mem::take(card), max_h, card_w, theme.gray_dim);
            }

            for row in 0..max_h {
                let mut spans: Vec<Span<'static>> = vec![Span::raw("  ")];
                for (i, card) in card_lines.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::raw(" ".repeat(gap)));
                    }
                    if let Some(line) = card.get(row) {
                        spans.extend(line.spans.iter().cloned());
                    }
                }
                lines.push(
                    BlockLine::styled(Line::from(spans)).with_selection_range(Some(0)),
                );
            }
            // Small gap between masonry rows
            lines.push(
                BlockLine::styled(Line::from(Span::raw("  "))).with_selection_range(None),
            );
        }

        lines
    }
}

fn truncate_ellipsis(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    if max <= 1 {
        return "…".to_string();
    }
    let keep = max - 1;
    s.chars().take(keep).collect::<String>() + "…"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrollback::types::DisplayMode;
    use next_code_best_of_n::{BestOfNFileStat, BestOfNPhase, BestOfNProgressPayload};

    fn make_ctx(width: u16) -> BlockContext {
        BlockContext {
            width,
            mode: DisplayMode::Collapsed,
            is_running: false,
            raw: false,
            max_lines: None,
            appearance: Default::default(),
            is_selected: false,
            cwd: None,
        }
    }

    fn sample_payload() -> BestOfNProgressPayload {
        BestOfNProgressPayload {
            run_id: "r1".into(),
            phase: BestOfNPhase::CandidateDone,
            message: "2/2 proposals complete...".into(),
            completed: 2,
            total: 2,
            candidates: vec![
                BestOfNCandidateUi {
                    index: 0,
                    candidate_id: "c0".into(),
                    label: "temp-0".into(),
                    status: "success".into(),
                    file_count: 1,
                    files: vec!["src/a.rs".into()],
                    file_stats: vec![BestOfNFileStat {
                        path: "src/a.rs".into(),
                        change_type: "M".into(),
                        lines_added: 3,
                        lines_removed: 1,
                    }],
                    error: None,
                    recommended: true,
                },
                BestOfNCandidateUi::pending(1, "c1", "temp-1"),
            ],
            recommended_index: Some(0),
            selection_reason: Some("focused edit".into()),
        }
    }

    fn lines_text(out: &BlockOutput) -> String {
        out.lines
            .iter()
            .map(|l| {
                l.content
                    .spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn render_uses_rounded_borders_and_proposal_names() {
        let block = BestOfNBlock::new(&sample_payload());
        let out = block.output(&make_ctx(100));
        let text = lines_text(&out);
        assert!(text.contains('╭'), "rounded TL: {text}");
        assert!(text.contains("Proposal #1"), "{text}");
        assert!(
            text.contains("completed ✓") || text.contains("● running"),
            "{text}"
        );
        assert!(text.contains("+3"), "{text}");
    }

    #[test]
    fn narrow_width_stacks_single_column() {
        let block = BestOfNBlock::new(&sample_payload());
        let out = block.output(&make_ctx(40));
        assert!(out.lines.len() > 4);
    }
}
