//! Face reconnect banner — paste-port of origin TUI connection card
//! (`next-code-tui` `render_connection_system_message`).
//!
//! Origin stores a system message like
//! `⚡ Connection lost - retrying (attempt N, Xs) - <detail> · resume: …`
//! and renders an orange rounded card. Face keeps structured state and paints
//! the same card in the agent banner slot while reconnect is in progress.

use std::time::Instant;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

use super::info_floats::rgb;

/// Live reconnect status shown above the agent scrollback.
#[derive(Debug, Clone)]
pub struct ReconnectBanner {
    pub attempt: u32,
    pub started_at: Instant,
    pub detail: String,
    /// e.g. `next-code --resume giraffe` (without the "Resume " label).
    pub resume_hint: Option<String>,
}

impl ReconnectBanner {
    pub fn retrying(attempt: u32, detail: impl Into<String>, resume_hint: Option<String>) -> Self {
        Self {
            attempt: attempt.max(1),
            started_at: Instant::now(),
            detail: detail.into(),
            resume_hint,
        }
    }

    pub fn with_attempt(mut self, attempt: u32) -> Self {
        self.attempt = attempt.max(1);
        self
    }

    pub fn elapsed_label(&self) -> String {
        let secs = self.started_at.elapsed().as_secs();
        if secs < 60 {
            format!("{secs}s")
        } else {
            format!("{}m {}s", secs / 60, secs % 60)
        }
    }

    pub fn status_line(&self) -> String {
        format!(
            "Retrying · attempt {} · {}",
            self.attempt,
            self.elapsed_label()
        )
    }
}

/// Rows reserved for the reconnect card (rounded border + up to 3 body lines).
pub const RECONNECT_BANNER_HEIGHT: u16 = 5;

fn truncate_line(input: &str, width: usize) -> String {
    if input.chars().count() <= width {
        return input.to_string();
    }
    if width <= 1 {
        return "…".to_string();
    }
    let mut out: String = input.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Paint the orange `⚡ reconnecting` card into `area`.
pub fn render(area: Rect, buf: &mut Buffer, banner: &ReconnectBanner) {
    if area.width < 8 || area.height == 0 {
        return;
    }

    Clear.render(area, buf);

    let border = rgb(255, 193, 94);
    let status_fg = rgb(255, 220, 140);
    let label_fg = rgb(140, 150, 165);
    let body_fg = rgb(225, 232, 245);
    let hint_fg = rgb(170, 200, 255);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            " ⚡ reconnecting ",
            Style::default().fg(border).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let inner_w = inner.width as usize;
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(3);
    lines.push(Line::from(Span::styled(
        truncate_line(&banner.status_line(), inner_w),
        Style::default().fg(status_fg).add_modifier(Modifier::BOLD),
    )));

    if !banner.detail.trim().is_empty() && lines.len() < inner.height as usize {
        let detail = truncate_line(&banner.detail.replace('\n', " "), inner_w.saturating_sub(7));
        lines.push(Line::from(vec![
            Span::styled("Detail ", Style::default().fg(label_fg)),
            Span::styled(detail, Style::default().fg(body_fg)),
        ]));
    }

    if let Some(hint) = banner
        .resume_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        && lines.len() < inner.height as usize
    {
        let hint = truncate_line(hint, inner_w.saturating_sub(7));
        lines.push(Line::from(vec![
            Span::styled("Resume ", Style::default().fg(label_fg)),
            Span::styled(hint, Style::default().fg(hint_fg)),
        ]));
    }

    // Keep title glyph width stable on narrow terminals (origin width_stable).
    let _ = "reconnecting".width();

    Paragraph::new(lines).render(inner, buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_paints_reconnecting_title_and_detail() {
        let banner = ReconnectBanner::retrying(
            1,
            "server closed the connection",
            Some("next-code --resume giraffe".into()),
        );
        let area = Rect::new(0, 0, 60, RECONNECT_BANNER_HEIGHT);
        let mut buf = Buffer::empty(area);
        render(area, &mut buf, &banner);

        let mut plain = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                plain.push(buf[(x, y)].symbol().chars().next().unwrap_or(' '));
            }
            plain.push('\n');
        }
        assert!(
            plain.contains("reconnecting"),
            "missing title in:\n{plain}"
        );
        assert!(
            plain.contains("Retrying"),
            "missing status in:\n{plain}"
        );
        assert!(
            plain.contains("server closed"),
            "missing detail in:\n{plain}"
        );
        assert!(
            plain.contains("giraffe"),
            "missing resume hint in:\n{plain}"
        );
    }
}
