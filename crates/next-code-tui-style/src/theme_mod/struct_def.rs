//! Theme struct — copied from grok-build tokyonight.rs.
//! All colors come from the `Theme` struct. No hardcoded colors elsewhere.

use ratatui::style::{Color, Modifier, Style};

/// Helper for concise const Color::Rgb definitions.
const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

/// Theme for next-code UI rendering.
/// Copied from grok-build's Theme struct (61 fields + 6 Modifier fields).
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // Backgrounds
    pub bg_base: Color,
    pub bg_light: Color,
    pub bg_dark: Color,
    pub bg_highlight: Color,
    pub bg_hover: Color,
    pub bg_terminal: Color,

    // Accent colors
    pub accent_user: Color,
    pub accent_assistant: Color,
    pub accent_thinking: Color,
    pub accent_tool: Color,
    pub accent_system: Color,
    pub accent_error: Color,
    pub accent_success: Color,
    pub accent_running: Color,
    pub accent_skill: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,

    // Gray scale
    pub gray_dim: Color,
    pub gray: Color,
    pub gray_bright: Color,

    // Semantic colors
    pub command: Color,
    pub path: Color,
    pub running: Color,
    pub warning: Color,

    // Search
    pub fuzzy_accent: Color,

    // Plan mode
    pub accent_plan: Color,

    // Verify/feedback/remember
    pub accent_verify: Color,
    pub accent_feedback: Color,
    pub accent_remember: Color,

    // Selection/border
    pub selection_border: Color,
    pub hover_border: Color,
    pub prompt_border: Color,
    pub prompt_border_active: Color,

    // Prompt info
    pub accent_model: Color,

    // Scrollbar
    pub scrollbar_bg: Color,
    pub scrollbar_fg: Color,

    // Diff
    pub diff_delete_bg: Color,
    pub diff_delete_fg: Color,
    pub diff_insert_bg: Color,
    pub diff_insert_fg: Color,
    pub diff_equal_fg: Color,
    pub diff_gutter_fg: Color,

    // Visual selection
    pub bg_visual: Color,

    // Paste
    pub paste_bg: Color,
    pub paste_fg: Color,
    pub paste_dim: Color,

    // Markdown headings
    pub md_heading_h1: Color,
    pub md_heading_h1_mod: Modifier,
    pub md_heading_h2: Color,
    pub md_heading_h2_mod: Modifier,
    pub md_heading_h3: Color,
    pub md_heading_h3_mod: Modifier,
    pub md_heading_h4: Color,
    pub md_heading_h4_mod: Modifier,
    pub md_heading_h5: Color,
    pub md_heading_h5_mod: Modifier,
    pub md_heading_h6: Color,
    pub md_heading_h6_mod: Modifier,

    // Markdown inline/code
    pub md_code: Color,
    pub md_task_checked: Color,
    pub md_task_unchecked: Color,
    pub md_muted: Color,
    pub md_code_bg: Color,
    pub md_text: Color,
    pub link_fg: Color,
}

impl Theme {
    /// Get a style with the given foreground color.
    pub const fn fg(&self, color: Color) -> Style {
        Style::new().fg(color)
    }

    /// Get a style with muted text (gray — medium).
    pub const fn muted(&self) -> Style {
        match self.gray {
            Color::Reset => Style::new().add_modifier(Modifier::DIM),
            c => Style::new().fg(c),
        }
    }

    /// Style for hyperlink overlay text.
    pub fn link_style(&self) -> Style {
        Style::new()
            .fg(self.link_fg)
            .add_modifier(ratatui::style::Modifier::UNDERLINED)
    }

    /// Get a style with dim text (gray_dim — dimmest).
    pub const fn dim(&self) -> Style {
        match self.gray_dim {
            Color::Reset => Style::new().add_modifier(Modifier::DIM),
            c => Style::new().fg(c),
        }
    }

    /// Get a style for primary text.
    pub const fn primary(&self) -> Style {
        Style::new().fg(self.text_primary)
    }

    /// Get a bold style.
    pub const fn bold(&self) -> Style {
        Style::new().add_modifier(Modifier::BOLD)
    }

    /// Whether this theme is a dark theme (bg_base is dark).
    pub fn is_dark(&self) -> bool {
        match self.bg_base {
            Color::Rgb(r, g, b) => {
                // ITU-R BT.709 luminance
                let lum = 0.2126 * (r as f32 / 255.0)
                    + 0.7152 * (g as f32 / 255.0)
                    + 0.0722 * (b as f32 / 255.0);
                lum < 0.5
            }
            _ => true,
        }
    }
}
