//! Terminal-native palette for minimal/no-theme mode.
//! Copied from grok-build terminal_default.rs.
//! Uses Color::Reset for body text so the terminal's own fg/bg shows through.

use ratatui::style::{Color, Modifier};

use crate::theme_mod::struct_def::Theme;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

impl Theme {
    /// Terminal-native palette — minimal/no-theme mode.
    /// Body text uses Color::Reset so the terminal's own fg/bg shows through.
    pub const fn terminal_default() -> Self {
        Self {
            bg_base: Color::Reset,
            bg_light: Color::Reset,
            bg_dark: Color::Reset,
            bg_highlight: Color::Reset,
            bg_hover: Color::Reset,
            bg_terminal: Color::Reset,

            accent_user: rgb(40, 150, 255),
            accent_assistant: rgb(200, 130, 255),
            accent_thinking: Color::Reset,
            accent_tool: Color::Reset,
            accent_system: rgb(40, 150, 255),
            accent_error: rgb(255, 80, 80),
            accent_success: rgb(80, 200, 80),
            accent_running: rgb(200, 130, 255),
            accent_skill: Color::Reset,

            text_primary: Color::Reset,
            text_secondary: Color::Reset,

            gray_dim: Color::Reset,
            gray: Color::Reset,
            gray_bright: Color::Reset,

            command: rgb(160, 130, 40),
            path: rgb(180, 100, 40),
            running: rgb(60, 180, 220),
            warning: rgb(160, 130, 40),

            fuzzy_accent: rgb(40, 150, 255),

            accent_plan: rgb(180, 150, 50),
            accent_verify: rgb(200, 130, 255),
            accent_feedback: rgb(80, 200, 80),
            accent_remember: rgb(80, 200, 80),

            selection_border: Color::Reset,
            hover_border: Color::Reset,
            prompt_border: Color::Reset,
            prompt_border_active: Color::Reset,

            accent_model: rgb(60, 180, 220),

            scrollbar_bg: Color::Reset,
            scrollbar_fg: Color::Reset,

            diff_delete_bg: Color::Reset,
            diff_delete_fg: rgb(255, 80, 80),
            diff_insert_bg: Color::Reset,
            diff_insert_fg: rgb(80, 200, 80),
            diff_equal_fg: Color::Reset,
            diff_gutter_fg: Color::Reset,

            bg_visual: Color::Reset,

            paste_bg: Color::Reset,
            paste_fg: Color::Reset,
            paste_dim: Color::Reset,

            md_heading_h1: rgb(60, 180, 220),
            md_heading_h1_mod: Modifier::BOLD,
            md_heading_h2: rgb(40, 150, 255),
            md_heading_h2_mod: Modifier::BOLD,
            md_heading_h3: rgb(180, 100, 40),
            md_heading_h3_mod: Modifier::BOLD,
            md_heading_h4: rgb(160, 130, 40),
            md_heading_h4_mod: Modifier::BOLD,
            md_heading_h5: Color::Reset,
            md_heading_h5_mod: Modifier::BOLD,
            md_heading_h6: Color::Reset,
            md_heading_h6_mod: Modifier::empty(),
            md_code: rgb(80, 200, 80),
            md_task_checked: rgb(60, 180, 220),
            md_task_unchecked: rgb(40, 150, 255),
            md_muted: Color::Reset,
            md_code_bg: Color::Reset,
            md_text: Color::Reset,
            link_fg: rgb(40, 150, 255),
        }
    }
}
