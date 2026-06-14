use super::{RunningItem, RunningItemStatus};
use crate::tui::color_support::rgb;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;
use std::time::Duration;

pub(super) fn draw_running_items(
    frame: &mut Frame,
    app: &dyn super::TuiState,
    area: Rect,
) {
    let items_state = app.running_items();
    if !items_state.visible || items_state.items.is_empty() {
        return;
    }
    if area.height == 0 {
        return;
    }

    let inner_w = area.width as usize;

    // Header line
    let header_style = Style::default().fg(rgb(100, 100, 110));
    let hint = " ↑/↓ to select · Enter to view ";
    let header = format!(
        "{:width$}{}",
        "  ⏺ main",
        hint,
        width = inner_w.saturating_sub(hint.chars().count())
    );

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled(header, header_style)));

    let selected = items_state.selected;
    let max_visible = area.height.saturating_sub(1) as usize;

    // Show items with scroll offset if needed
    let scroll_offset = if items_state.items.len() > max_visible {
        selected.saturating_sub(max_visible.saturating_sub(1) / 2)
    } else {
        0
    };

    for (_display_idx, idx) in (scroll_offset..items_state.items.len())
        .take(max_visible)
        .enumerate()
    {
        let item = &items_state.items[idx];
        let is_selected = idx == selected;

        let (icon, icon_color) = item_icon_and_color(item);

        let mut spans: Vec<Span<'static>> = Vec::new();

        // Selection indicator
        if is_selected {
            spans.push(Span::styled(
                "❯ ",
                Style::default().fg(rgb(80, 160, 255)).bold(),
            ));
        } else {
            spans.push(Span::styled("  ", Style::default()));
        }

        // Status icon
        spans.push(Span::styled(
            format!("{} ", icon),
            Style::default().fg(icon_color),
        ));

        // Label
        let label_style = if is_selected {
            Style::default().fg(rgb(220, 220, 230)).bold()
        } else {
            Style::default().fg(rgb(180, 180, 190))
        };
        spans.push(Span::styled(item.label.clone(), label_style));

        // Detail text (truncated)
        if let Some(detail) = &item.detail {
            let available = inner_w.saturating_sub(UnicodeWidthStr::width(item.label.as_str()) + 6);
            if available > 4 {
                let truncated = truncate_to_width(detail, available);
                spans.push(Span::styled(
                    format!(" {}", truncated),
                    Style::default().fg(rgb(120, 120, 130)),
                ));
            }
        }

        // Elapsed time (right-aligned)
        if let Some(elapsed) = item.elapsed {
            let elapsed_str = format_elapsed(elapsed);
            let line_w: usize = spans.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
            let padding = inner_w.saturating_sub(line_w + elapsed_str.chars().count());
            if padding > 1 {
                spans.push(Span::raw(" ".repeat(padding)));
            }
            spans.push(Span::styled(
                elapsed_str,
                Style::default().fg(rgb(120, 120, 130)),
            ));
        }

        lines.push(Line::from(spans));
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, area);
}

fn item_icon_and_color(item: &RunningItem) -> (&'static str, Color) {
    match item.status {
        RunningItemStatus::Running => ("◯", rgb(80, 220, 100)),
        RunningItemStatus::Completed => ("✓", rgb(100, 180, 100)),
        RunningItemStatus::Failed => ("✗", rgb(255, 100, 100)),
        RunningItemStatus::Stopped => ("■", rgb(200, 180, 80)),
    }
}

fn format_elapsed(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
    let w = UnicodeWidthStr::width(s);
    if w <= max_width {
        s.to_string()
    } else {
        let mut out = String::with_capacity(max_width);
        for c in s.chars() {
            if UnicodeWidthStr::width(out.as_str()) + UnicodeWidthStr::width(c.to_string().as_str()) > max_width.saturating_sub(1) {
                break;
            }
            out.push(c);
        }
        out.push('…');
        out
    }
}
