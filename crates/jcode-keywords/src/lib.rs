//! Magic keyword system for jcode.
//!
//! Detects keyword triggers in user input (default: **Strict** `$keyword` /
//! token aliases), manages persistent mode state, builds prompt injections,
//! and dispatches to workflow handlers.
//!
//! # Architecture
//!
//! ```text
//! User types "$ultrawork fix the bug"
//!     ↓
//! detector::detect_keywords_with(Strict) → DetectedKeyword
//!     ↓
//! state::update_modes_with_limit() → ModeState (.jcode/state/modes.toml)
//!     ↓
//! workflow::executor::process_turn_with_options()
//!     ↓
//! keyword_prompt injected into system prompt + optional status chips
//! ```

pub mod conflict;
pub mod detector;
pub mod intent;
pub mod options;
pub mod registry;
pub mod sanitizer;
pub mod state;
pub mod task_size;
pub mod visual;
pub mod workflow;

// Re-exports for convenience
pub use detector::{DetectedKeyword, detect_keywords, detect_keywords_with};
pub use options::{
    DetectOptions, MatchMode, ProcessTurnOptions, process_turn_options_from_config,
};
pub use registry::{KeywordEntry, WorkflowKind, list_canonical_keywords};
pub use state::{
    ModeChip, ModeState, clear_modes, clear_modes_if_session_start, load_state, mode_chips,
};
pub use visual::{KeywordHighlight, compute_highlights, compute_highlights_with};
pub use workflow::executor::DeferredSpawn;
pub use workflow::executor::{
    apply_actions, build_workflow_prompt, execute_active_workflows, process_turn,
    process_turn_response, process_turn_with_options,
};
pub use workflow::{WorkflowAction, WorkflowContext, WorkflowHandler};
