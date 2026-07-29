//! Stacked toast notification system.
//!
//! Provides a persistent [`ToastStack`] that holds multiple [`ToastMessage`]s
//! at different severity levels, each with its own TTL. Expired toasts are
//! removed on `tick`. Render draws at most 3 stacked toasts in the bottom-
//! right corner of the given area, each with a prefix icon and level-appropriate
//! color.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;

use crate::theme::Theme;

/// Severity level for a toast notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warn,
    Error,
}

impl ToastLevel {
    /// Default TTL for this level.
    fn default_ttl(self) -> Duration {
        match self {
            ToastLevel::Info | ToastLevel::Success => Duration::from_secs(3),
            ToastLevel::Warn => Duration::from_secs(5),
            ToastLevel::Error => Duration::from_secs(8),
        }
    }

    /// Prefix icon displayed before the message.
    fn prefix_icon(self) -> &'static str {
        match self {
            ToastLevel::Info => "\u{2713}",   // ✓
            ToastLevel::Success => "\u{2713}", // ✓
            ToastLevel::Warn => "\u{26A0}",    // ⚠
            ToastLevel::Error => "\u{2715}",   // ✕
        }
    }

    /// Foreground color for the toast text (icon uses a brighter variant).
    fn color(self, theme: &Theme) -> ratatui::style::Color {
        match self {
            ToastLevel::Info => theme.accent_user,
            ToastLevel::Success => theme.accent_success,
            ToastLevel::Warn => theme.warning,
            ToastLevel::Error => theme.accent_error,
        }
    }
}

/// A single toast notification with text, level, creation time, and TTL.
#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub text: String,
    pub level: ToastLevel,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl ToastMessage {
    /// Create a new toast with the given text and level, using the default TTL.
    pub fn new(text: String, level: ToastLevel) -> Self {
        Self {
            text,
            level,
            created_at: Instant::now(),
            ttl: level.default_ttl(),
        }
    }

    /// Whether this toast has expired relative to `now`.
    pub fn expired(&self, now: Instant) -> bool {
        now.duration_since(self.created_at) >= self.ttl
    }
}

/// A stack of toast notifications displayed in the UI.
///
/// Up to [`MAX_VISIBLE`] toasts are rendered at once. Expired toasts are
/// removed on [`tick`](Self::tick). The stack supports both LIFO-style push
/// and FIFO-style dismiss (oldest first).
#[derive(Debug, Clone)]
pub struct ToastStack {
    messages: VecDeque<ToastMessage>,
}

/// Maximum number of toast messages rendered at once.
const MAX_VISIBLE: usize = 3;

impl ToastStack {
    /// Create a new empty toast stack.
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    /// Push a new toast onto the stack with the given text and level.
    ///
    /// When the stack exceeds [`MAX_VISIBLE`], the oldest (front) message
    /// is dropped so the newest messages remain visible.
    pub fn push(&mut self, text: String, level: ToastLevel) {
        self.messages.push_back(ToastMessage::new(text, level));
        // Keep newest messages; drop oldest when over capacity.
        if self.messages.len() > MAX_VISIBLE {
            self.messages.pop_front();
        }
    }

    /// Remove any expired toasts relative to `now`. Returns `true` if any
    /// were removed (caller may need a redraw).
    pub fn tick(&mut self, now: Instant) -> bool {
        let before = self.messages.len();
        self.messages.retain(|m| !m.expired(now));
        self.messages.len() != before
    }

    /// Whether the stack has no messages.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Remove the oldest toast (FIFO dismiss). Returns the removed message,
    /// or `None` if the stack was already empty.
    pub fn dismiss(&mut self) -> Option<ToastMessage> {
        self.messages.pop_front()
    }

    /// Render up to [`MAX_VISIBLE`] toasts stacked from the bottom-right of
    /// `area`. Each toast is a single line with a prefix icon and the message
    /// text, colored per level.
    ///
    /// The stack grows upward from the bottom of `area`, so the newest toast
    /// is at the top of the stack and the oldest (soonest to expire) is at the
    /// bottom. This matches the LIFO visual priority: what just happened is
    /// most visible.
    pub fn render(&self, buf: &mut Buffer, area: Rect, theme: &Theme) {
        if self.messages.is_empty() || area.width < 10 || area.height == 0 {
            return;
        }

        let max_lines = (self.messages.len() as u16).min(MAX_VISIBLE as u16).min(area.height);
        let bottom_y = area.bottom().saturating_sub(1);

        // Iterate from newest (back) to oldest (front), stacking upward.
        for (i, msg) in self.messages.iter().rev().take(MAX_VISIBLE).enumerate() {
            let i = i as u16;
            if i >= max_lines {
                break;
            }
            let y = bottom_y.saturating_sub(i);
            if y < area.y {
                break;
            }

            let icon = msg.level.prefix_icon();
            let color = msg.level.color(theme);
            // Style: icon slightly brighter via bold, text normal.
            let icon_style = Style::default().fg(color).bg(theme.bg_base);
            let text_style = Style::default()
                .fg(color)
                .bg(theme.bg_base);

            // Max available width for one toast line (icon + space + text + padding).
            let max_w = area.width.saturating_sub(2) as usize;
            if max_w < 3 {
                continue;
            }
            let icon_w = icon.chars().count();
            let text_budget = max_w.saturating_sub(icon_w + 1);
            let display_text: String = if msg.text.chars().count() > text_budget {
                msg.text
                    .chars()
                    .take(text_budget.saturating_sub(1))
                    .collect::<String>()
                    + "\u{2026}"
            } else {
                msg.text.clone()
            };

            // Right-align the toast in the area.
            let line = format!(" {icon} {display_text} ");
            let line_w = line.chars().count() as u16;
            let x = area.right().saturating_sub(line_w + 1).max(area.x);

            for (j, ch) in line.chars().enumerate() {
                let col = x + j as u16;
                if col >= area.right() {
                    break;
                }
                if let Some(cell) = buf.cell_mut((col, y)) {
                    cell.set_char(ch);
                    cell.bg = theme.bg_base;
                    if j < icon_w.saturating_add(2) {
                        // The " " + icon + " " portion
                        cell.set_style(icon_style);
                    } else {
                        cell.set_style(text_style);
                    }
                }
            }
        }
    }
}

impl Default for ToastStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use ratatui::buffer::Buffer;

    /// Helper: render the stack into a minimal buffer and return lines of text.
    fn render_lines(stack: &ToastStack, width: u16, height: u16) -> Vec<String> {
        let theme = Theme::current();
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        stack.render(&mut buf, area, &theme);
        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .filter_map(|x| buf.cell((x, y)).map(|c| c.symbol().to_string()))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn empty_stack_renders_nothing() {
        let stack = ToastStack::new();
        let lines = render_lines(&stack, 40, 5);
        // Render fills the area with space characters, so each line
        // should contain only spaces.
        assert!(lines.iter().all(|l| l.chars().all(|c| c == ' ')));
    }

    #[test]
    fn pushes_info_toast() {
        let mut stack = ToastStack::new();
        stack.push("Hello".into(), ToastLevel::Info);
        assert_eq!(stack.messages.len(), 1);
        let msg = &stack.messages[0];
        assert_eq!(msg.text, "Hello");
        assert_eq!(msg.level, ToastLevel::Info);
    }

    #[test]
    fn dismiss_oldest() {
        let mut stack = ToastStack::new();
        stack.push("A".into(), ToastLevel::Info);
        stack.push("B".into(), ToastLevel::Info);
        let removed = stack.dismiss();
        assert_eq!(removed.unwrap().text, "A");
        assert_eq!(stack.messages.len(), 1);
        assert_eq!(stack.messages[0].text, "B");
    }

    #[test]
    fn oldest_dropped_on_capacity() {
        let mut stack = ToastStack::new();
        stack.push("A".into(), ToastLevel::Info);
        stack.push("B".into(), ToastLevel::Info);
        stack.push("C".into(), ToastLevel::Info);
        stack.push("D".into(), ToastLevel::Info);
        assert_eq!(stack.messages.len(), MAX_VISIBLE);
        assert_eq!(stack.messages[0].text, "B");
        assert_eq!(stack.messages[1].text, "C");
        assert_eq!(stack.messages[2].text, "D");
    }

    #[test]
    fn tick_removes_expired() {
        let mut stack = ToastStack::new();
        stack.push("Old".into(), ToastLevel::Info);
        let now = Instant::now();
        // Advance time past the TTL.
        let future = now + Duration::from_secs(10);
        let changed = stack.tick(future);
        assert!(changed);
        assert!(stack.is_empty());
    }

    #[test]
    fn tick_noop_on_fresh() {
        let mut stack = ToastStack::new();
        stack.push("Fresh".into(), ToastLevel::Info);
        let now = Instant::now();
        let changed = stack.tick(now);
        assert!(!changed);
        assert!(!stack.is_empty());
    }
}
