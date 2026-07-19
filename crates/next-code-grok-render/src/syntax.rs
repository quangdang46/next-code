//! `syntax` module — Grok-compatible syntax highlighting.
//!
//! Wraps syntect-based syntax highlighting compatible with Grok's API.

/// Syntax highlighting configuration.
#[derive(Debug, Clone)]
pub struct SyntaxConfig {
    pub theme: String,
    pub enable: bool,
}

impl Default for SyntaxConfig {
    fn default() -> Self {
        Self {
            theme: "tokyo-night".into(),
            enable: true,
        }
    }
}
