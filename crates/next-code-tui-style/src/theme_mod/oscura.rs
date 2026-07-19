//! Oscura Midnight theme — copied from grok-build.

use ratatui::style::{Color, Modifier};

use crate::theme_mod::struct_def::Theme;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

#[allow(dead_code)]
mod palette {
    use super::*;
    pub const BG: Color = rgb(20, 22, 28);
    pub const BG_DARK: Color = rgb(17, 18, 24);
    pub const BG_HIGHLIGHT: Color = rgb(30, 33, 42);
    pub const BG_STORM: Color = rgb(24, 26, 34);
    pub const BG_STORM_DARK: Color = rgb(21, 23, 30);
    pub const FG: Color = rgb(210, 215, 230);
    pub const FG_DARK: Color = rgb(180, 186, 204);
    pub const FG_GUTTER: Color = rgb(68, 72, 88);
    pub const COMMENT: Color = rgb(98, 102, 120);
    pub const DARK3: Color = rgb(90, 94, 112);
    pub const DARK5: Color = rgb(115, 120, 140);
    pub const BLUE: Color = rgb(100, 160, 240);
    pub const BLUE1: Color = rgb(50, 140, 190);
    pub const CYAN: Color = rgb(100, 190, 220);
    pub const GREEN: Color = rgb(130, 200, 130);
    pub const GREEN1: Color = rgb(95, 185, 155);
    pub const MAGENTA: Color = rgb(170, 140, 230);
    pub const ORANGE: Color = rgb(230, 150, 90);
    pub const PURPLE: Color = rgb(145, 112, 200);
    pub const RED: Color = rgb(220, 105, 120);
    pub const RED1: Color = rgb(195, 75, 90);
    pub const TEAL: Color = rgb(80, 175, 165);
    pub const YELLOW: Color = rgb(200, 175, 110);
}
use palette::*;

impl Theme {
    /// Oscura Midnight theme.
    pub const fn oscura_midnight() -> Self {
        Self {
            bg_base: BG_STORM,
            bg_light: BG_HIGHLIGHT,
            bg_dark: BG_HIGHLIGHT,
            bg_highlight: BG_HIGHLIGHT,
            bg_hover: rgb(38, 41, 52),
            bg_terminal: BG,

            accent_user: BLUE,
            accent_assistant: MAGENTA,
            accent_thinking: FG_GUTTER,
            accent_tool: DARK5,
            accent_system: BLUE,
            accent_error: RED,
            accent_success: GREEN,
            accent_running: MAGENTA,
            accent_skill: TEAL,

            text_primary: FG,
            text_secondary: FG_DARK,

            gray_dim: FG_GUTTER,
            gray: COMMENT,
            gray_bright: DARK5,

            command: YELLOW,
            path: ORANGE,
            running: CYAN,
            warning: YELLOW,

            fuzzy_accent: BLUE,

            accent_plan: rgb(210, 180, 80),

            accent_verify: MAGENTA,

            accent_feedback: GREEN1,

            accent_remember: Color::Rgb(130, 200, 120),

            selection_border: rgb(68, 72, 88),
            prompt_border: rgb(60, 64, 80),
            prompt_border_active: rgb(80, 84, 100),
            hover_border: rgb(38, 41, 52),

            accent_model: TEAL,

            scrollbar_bg: BG_STORM_DARK,
            scrollbar_fg: BG_HIGHLIGHT,

            diff_delete_bg: rgb(68, 20, 30),
            diff_delete_fg: RED,
            diff_insert_bg: rgb(15, 52, 25),
            diff_insert_fg: GREEN,
            diff_equal_fg: COMMENT,
            diff_gutter_fg: COMMENT,

            bg_visual: rgb(38, 41, 55),

            paste_bg: BG_STORM_DARK,
            paste_fg: FG_DARK,
            paste_dim: FG_GUTTER,

            md_heading_h1: TEAL,
            md_heading_h1_mod: Modifier::BOLD,
            md_heading_h2: BLUE,
            md_heading_h2_mod: Modifier::BOLD,
            md_heading_h3: PURPLE,
            md_heading_h3_mod: Modifier::BOLD,
            md_heading_h4: DARK5,
            md_heading_h4_mod: Modifier::BOLD,
            md_heading_h5: COMMENT,
            md_heading_h5_mod: Modifier::BOLD,
            md_heading_h6: DARK3,
            md_heading_h6_mod: Modifier::empty(),
            md_code: GREEN1,
            md_task_checked: GREEN,
            md_task_unchecked: FG_DARK,
            md_muted: COMMENT,
            md_code_bg: BG_HIGHLIGHT,
            md_text: FG_DARK,
            link_fg: BLUE,
        }
    }
}
