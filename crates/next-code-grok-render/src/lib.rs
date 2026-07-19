//! # next-code-grok-render
//!
//! Grok-compatible render adapter wrapping next-code's existing render infrastructure.
//! Provides the same public module structure as `xai-grok-pager-render` so that
//! downstream consumers (pager crate, theme system) can import from here
//! without source changes.
//!
//! Each module re-exports the most suitable type from next-code's own
//! render/TUI crates, or provides a local implementation where the gap is
//! too wide.

pub mod appearance;
pub mod clipboard;
pub mod gboom;
pub mod glyphs;
pub mod host;
pub mod link_opener;
pub mod modal_window_state;
pub mod prompt_images;
pub mod render;
pub mod syntax;
pub mod terminal;
pub mod theme;
pub mod util;
