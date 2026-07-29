//! Claude Code–style agent panel under the prompt.
//!
//! Roster of lead + workers with status chips; optional shared team task strip.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use crate::app::agent_roster::{
    color_index_for, AgentPanelState, AgentRosterRow, RosterStatus, TeamTaskItem,
};
use crate::theme::Theme;

/// Identity colors for worker rows (stable by name hash).
const WORKER_COLORS: &[(u8, u8, u8)] = &[
    (110, 180, 220),
    (180, 140, 220),
    (140, 200, 150),
    (220, 170, 110),
    (200, 130, 160),
    (130, 190, 190),
];

/// Desired height for the agent panel given roster + task strip visibility.
pub fn desired_height(
    rows: &[AgentRosterRow],
    show_team_tasks: bool,
    team_tasks: &[TeamTaskItem],
    max_rows: u16,
) -> u16 {
    if rows.len() <= 1 && team_tasks.is_empty() {
        // Lead-only: hide panel until workers or team tasks exist.
        return 0;
    }
    let roster_lines = 1u16; // compact single strip
    let task_lines = if show_team_tasks && !team_tasks.is_empty() {
        1 + team_tasks.len().min(4) as u16
    } else if show_team_tasks {
        1
    } else {
        0
    };
    let chrome = if show_team_tasks { 0 } else { 0 };
    let h = roster_lines + task_lines + chrome;
    h.min(max_rows.max(1))
}

/// Render the agent panel into `area`.
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    theme: &Theme,
    rows: &[AgentRosterRow],
    panel: &AgentPanelState,
    team_tasks: &[TeamTaskItem],
    viewing_label: Option<&str>,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let mut y = area.y;
    render_roster_strip(area.x, y, area.width, buf, theme, rows, panel, viewing_label);
    y = y.saturating_add(1);

    if panel.show_team_tasks && y < area.y.saturating_add(area.height) {
        render_task_strip(
            area.x,
            y,
            area.width,
            area.y.saturating_add(area.height).saturating_sub(y),
            buf,
            theme,
            team_tasks,
            panel.task_selected,
        );
    }
}

fn render_roster_strip(
    x: u16,
    y: u16,
    width: u16,
    buf: &mut Buffer,
    theme: &Theme,
    rows: &[AgentRosterRow],
    panel: &AgentPanelState,
    viewing_label: Option<&str>,
) {
    let mut spans: Vec<Span<'_>> = Vec::new();
    if let Some(name) = viewing_label {
        spans.push(Span::styled(
            format!(" Viewing @{name} · esc return "),
            Style::default()
                .fg(theme.gray)
                .add_modifier(Modifier::ITALIC),
        ));
        spans.push(Span::raw(" "));
    }

    let mut used = spans.iter().map(|s| s.content.width()).sum::<usize>();
    for (idx, row) in rows.iter().enumerate() {
        let selected = panel.selecting && panel.selected_index == idx;
        let bullet = if row.is_lead {
            "●"
        } else {
            status_bullet(row.status)
        };
        let color = if row.is_lead {
            theme.accent_system
        } else {
            let (r, g, b) = WORKER_COLORS[color_index_for(&row.color_key, WORKER_COLORS.len())];
            ratatui::style::Color::Rgb(r, g, b)
        };
        let mut label = format!("{bullet} {}", row.display_name);
        if let Some(ref act) = row.activity {
            if !act.is_empty() && !row.is_lead {
                label.push_str(" · ");
                label.push_str(act);
            }
        }
        if let Some((c, t)) = row.todo_progress {
            label.push_str(&format!(" {c}/{t}"));
        }
        let piece = if selected {
            format!("[{label}]")
        } else {
            format!(" {label} ")
        };
        let piece_w = piece.width();
        if used + piece_w + 1 > width as usize && used > 0 {
            spans.push(Span::styled(
                " …",
                Style::default().fg(theme.gray),
            ));
            break;
        }
        let style = if selected {
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(color)
        };
        spans.push(Span::styled(piece, style));
        used += piece_w;
    }

    if panel.selecting {
        spans.push(Span::styled(
            "  ↑↓ enter  x kill",
            Style::default().fg(theme.gray),
        ));
    }

    let line = Line::from(spans);
    buf.set_line(x, y, &line, width);
}

fn render_task_strip(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    buf: &mut Buffer,
    theme: &Theme,
    tasks: &[TeamTaskItem],
    selected: usize,
) {
    if height == 0 {
        return;
    }
    let pending = tasks.iter().filter(|t| t.is_pending()).count();
    let active = tasks.iter().filter(|t| t.is_in_progress()).count();
    let done = tasks.iter().filter(|t| t.is_completed()).count();
    let header = format!(" Tasks  {active} active · {pending} pending · {done} done  (Ctrl+Shift+T) ");
    buf.set_line(
        x,
        y,
        &Line::from(Span::styled(
            truncate_to_width(&header, width as usize),
            Style::default().fg(theme.gray),
        )),
        width,
    );

    let mut row_y = y.saturating_add(1);
    let max_show = (height.saturating_sub(1) as usize).min(tasks.len()).min(4);
    for (idx, task) in tasks.iter().take(max_show).enumerate() {
        if row_y >= y.saturating_add(height) {
            break;
        }
        let mark = if task.is_completed() {
            "✓"
        } else if task.is_in_progress() {
            "…"
        } else if task.is_claimable() {
            "○"
        } else {
            "·"
        };
        let assignee = task
            .assigned_to
            .as_deref()
            .map(|a| format!(" @{a}"))
            .unwrap_or_default();
        let claim = if task.is_claimable() { " [claim]" } else { "" };
        let text = format!("  {mark} {}{assignee}{claim}", task.content);
        let selected_row = idx == selected;
        let style = if selected_row {
            Style::default()
                .fg(theme.accent_system)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_primary)
        };
        buf.set_line(
            x,
            row_y,
            &Line::from(Span::styled(truncate_to_width(&text, width as usize), style)),
            width,
        );
        row_y = row_y.saturating_add(1);
    }
}

fn status_bullet(status: RosterStatus) -> &'static str {
    match status {
        RosterStatus::Lead => "●",
        RosterStatus::Running | RosterStatus::PendingKill => "◉",
        RosterStatus::Idle => "○",
        RosterStatus::NeedsInput => "◎",
        RosterStatus::Completed => "✓",
        RosterStatus::Failed => "✗",
        RosterStatus::Cancelled => "–",
    }
}

fn truncate_to_width(s: &str, max: usize) -> String {
    if s.width() <= max {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]));
        if w + cw + 1 > max {
            out.push('…');
            break;
        }
        out.push(ch);
        w += cw;
    }
    out
}

/// Render soft swarm transcript lines into `area` (Claude soft-buffer view).
pub fn render_soft_transcript(
    area: Rect,
    buf: &mut Buffer,
    theme: &Theme,
    label: &str,
    lines: &[crate::app::agent_roster::SoftTranscriptLine],
) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let header = format!(" Soft view @{label} · esc return ");
    buf.set_line(
        area.x,
        area.y,
        &Line::from(Span::styled(
            truncate_to_width(&header, area.width as usize),
            Style::default()
                .fg(theme.gray)
                .add_modifier(Modifier::ITALIC),
        )),
        area.width,
    );
    let mut y = area.y.saturating_add(1);
    let max_y = area.y.saturating_add(area.height);
    for line in lines.iter().rev().take(area.height.saturating_sub(1) as usize).collect::<Vec<_>>().into_iter().rev() {
        if y >= max_y {
            break;
        }
        let role = if line.role.eq_ignore_ascii_case("user") {
            "you"
        } else if line.role.eq_ignore_ascii_case("assistant") {
            "agent"
        } else {
            line.role.as_str()
        };
        let text = format!("  {role}: {}", line.content);
        buf.set_line(
            area.x,
            y,
            &Line::from(Span::styled(
                truncate_to_width(&text, area.width as usize),
                Style::default().fg(theme.text_primary),
            )),
            area.width,
        );
        y = y.saturating_add(1);
    }
    if lines.is_empty() && y < max_y {
        buf.set_line(
            area.x,
            y,
            &Line::from(Span::styled(
                "  (no soft transcript yet — messages appear here)",
                Style::default().fg(theme.gray),
            )),
            area.width,
        );
    }
}

/// Widget wrapper for the agent panel.
pub struct AgentPanelWidget<'a> {
    pub rows: &'a [AgentRosterRow],
    pub panel: &'a AgentPanelState,
    pub team_tasks: &'a [TeamTaskItem],
    pub theme: &'a Theme,
    pub viewing_label: Option<&'a str>,
}

impl Widget for AgentPanelWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render(
            area,
            buf,
            self.theme,
            self.rows,
            self.panel,
            self.team_tasks,
            self.viewing_label,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::agent_roster::{RosterSource, RosterStatus};

    fn row(id: &str, lead: bool) -> AgentRosterRow {
        AgentRosterRow {
            id: id.into(),
            display_name: id.into(),
            source: if lead {
                RosterSource::Lead
            } else {
                RosterSource::GrokSubagent
            },
            status: if lead {
                RosterStatus::Lead
            } else {
                RosterStatus::Running
            },
            activity: None,
            color_key: id.into(),
            kill_subagent_id: None,
            can_message: true,
            can_open_transcript: !lead,
            todo_progress: None,
            is_lead: lead,
        }
    }

    #[test]
    fn desired_height_hidden_for_lead_only() {
        let rows = vec![row("lead", true)];
        assert_eq!(desired_height(&rows, false, &[], 10), 0);
    }

    #[test]
    fn desired_height_shows_with_workers() {
        let rows = vec![row("lead", true), row("w1", false)];
        assert!(desired_height(&rows, false, &[], 10) >= 1);
        assert!(desired_height(&rows, true, &[], 10) >= 1);
    }
}
