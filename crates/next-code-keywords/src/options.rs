//! Runtime options for keyword detection and turn processing.

use serde::{Deserialize, Serialize};

/// How aggressively to match keyword aliases in user input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode {
    /// Only `$keyword` (exact, case-insensitive) and exact **token** aliases
    /// with word boundaries. No multi-word phrases, no fuzzy matching.
    #[default]
    Strict,
    /// Legacy: multi-word phrase aliases + optional fuzzy Levenshtein.
    Loose,
}

/// Options for [`crate::detector::detect_keywords_with`].
#[derive(Debug, Clone, Copy)]
pub struct DetectOptions {
    pub match_mode: MatchMode,
    /// When true and `match_mode` is [`MatchMode::Loose`], allow Levenshtein ≤ 2
    /// on aliases of length ≥ 5.
    pub allow_fuzzy: bool,
}

impl Default for DetectOptions {
    fn default() -> Self {
        Self {
            match_mode: MatchMode::Strict,
            allow_fuzzy: false,
        }
    }
}

impl DetectOptions {
    pub fn strict() -> Self {
        Self::default()
    }

    pub fn loose(allow_fuzzy: bool) -> Self {
        Self {
            match_mode: MatchMode::Loose,
            allow_fuzzy,
        }
    }
}

/// Options for [`crate::workflow::executor::process_turn_with_options`].
#[derive(Debug, Clone, Copy)]
pub struct ProcessTurnOptions {
    pub enabled: bool,
    pub detect: DetectOptions,
    /// Max turns a sticky mode stays active (including the activation turn).
    pub sticky_turns: u32,
}

impl Default for ProcessTurnOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            detect: DetectOptions::strict(),
            sticky_turns: 10,
        }
    }
}

/// Build options from config-like fields (avoids a config dependency here).
pub fn process_turn_options_from_config(
    enabled: bool,
    match_mode: &str,
    sticky_turns: u32,
    allow_fuzzy: bool,
) -> ProcessTurnOptions {
    let match_mode = match match_mode.trim().to_ascii_lowercase().as_str() {
        "loose" | "legacy" | "nl" => MatchMode::Loose,
        _ => MatchMode::Strict,
    };
    ProcessTurnOptions {
        enabled,
        detect: DetectOptions {
            match_mode,
            allow_fuzzy: allow_fuzzy && matches!(match_mode, MatchMode::Loose),
        },
        sticky_turns: sticky_turns.max(1),
    }
}
