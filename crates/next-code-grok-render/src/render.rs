//! `render` module — Grok-compatible rendering primitives.
//!
//! Wraps next-code's TUI render infrastructure for Grok compatibility.

/// Render a rounded box with title and content.
pub use next_code_tui_render::render_rounded_box;

/// Render a sharp-cornered box.
pub use next_code_tui_render::render_sharp_box;

/// Layout helpers.
pub use next_code_tui_render::layout;

/// Chrome rendering (scrollbar, borders, etc.).
pub use next_code_tui_render::chrome;

/// Memory tiles rendering.
pub use next_code_tui_render::memory_tiles;
