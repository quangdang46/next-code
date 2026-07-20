//! Façade stub of upstream `xai-grok-shell::sampling::types`. Upstream
//! re-exports the standalone `xai-grok-sampling-types` crate (not vendored
//! in this PR); this stub defines a self-contained `ReasoningEffort` +
//! `ReasoningEffortOption` shape covering the future pager's model-picker
//! / effort-selector import sites.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ReasoningEffort {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReasoningEffortOption {
    pub effort: ReasoningEffort,
    pub label: &'static str,
}

/// Upstream parses free-form user/config tokens ("low"/"medium"/"high",
/// case-insensitive) into a canonical `ReasoningEffort`.
pub fn parse_canonical_effort_token(token: &str) -> Option<ReasoningEffort> {
    match token.to_ascii_lowercase().as_str() {
        "low" => Some(ReasoningEffort::Low),
        "medium" => Some(ReasoningEffort::Medium),
        "high" => Some(ReasoningEffort::High),
        _ => None,
    }
}

/// Upstream gates the effort selector UI on model metadata; this stub
/// always reports supported (never blocks the picker in this compile-stub
/// layer).
pub fn supports_reasoning_effort_meta(_model: &str) -> bool {
    true
}
