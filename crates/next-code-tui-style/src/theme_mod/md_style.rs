//! Theme-aware markdown rendering style.
//! Adapted from grok-build md_style.rs.
//! Builds markdown style config from the current Theme.

use ratatui::style::{Color, Modifier, Style};

use crate::theme_mod::Theme;

/// Markdown style configuration built from the current Theme.
pub struct MarkdownStyle {
    pub heading_h1: Style,
    pub heading_h2: Style,
    pub heading_h3: Style,
    pub heading_h4: Style,
    pub heading_h5: Style,
    pub heading_h6: Style,
    pub code: Style,
    pub code_bg: Color,
    pub text: Style,
    pub muted: Style,
    pub link: Style,
    pub task_checked: Style,
    pub task_unchecked: Style,
}

/// Build a MarkdownStyle from the current theme.
pub fn style() -> MarkdownStyle {
    let t = Theme::current();
    let fg = |c| Style::default().fg(c);
    let fg_mod = |c, m| Style::default().fg(c).add_modifier(m);
    let mut muted = Style::default().fg(t.md_muted);
    if t.md_muted == Color::Reset {
        muted = muted.add_modifier(Modifier::DIM);
    }

    MarkdownStyle {
        heading_h1: fg_mod(t.md_heading_h1, t.md_heading_h1_mod),
        heading_h2: fg_mod(t.md_heading_h2, t.md_heading_h2_mod),
        heading_h3: fg_mod(t.md_heading_h3, t.md_heading_h3_mod),
        heading_h4: fg_mod(t.md_heading_h4, t.md_heading_h4_mod),
        heading_h5: fg_mod(t.md_heading_h5, t.md_heading_h5_mod),
        heading_h6: fg_mod(t.md_heading_h6, t.md_heading_h6_mod),
        code: fg(t.md_code),
        code_bg: t.md_code_bg,
        text: fg(t.md_text),
        muted,
        link: Style::default().fg(t.link_fg).add_modifier(Modifier::UNDERLINED),
        task_checked: fg(t.md_task_checked),
        task_unchecked: fg(t.md_task_unchecked),
    }
}
