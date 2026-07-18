//! Hooks module — re-exports from the `next-code-hooks` crate.
//!
//! This thin wrapper allows existing `crate::hooks::` import paths to keep
//! working while the actual implementation lives in `crates/next-code-hooks`.

pub use next_code_hooks::*;
