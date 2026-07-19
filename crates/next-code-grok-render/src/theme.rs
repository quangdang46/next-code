//! `theme` module — Grok-compatible theme system.
//!
//! Wraps next-code-tui-style theme infrastructure for Grok compatibility.

/// Re-export base style types from ratatui.
pub use ratatui::style::Color;
pub use ratatui::style::Style;
pub use ratatui::style::Modifier;

/// Theme definition — basic structure for Grok compatibility.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: Vec<(String, Color)>,
}
