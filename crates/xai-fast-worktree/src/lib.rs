//! Facade of `xai-org/grok-build` `xai-fast-worktree` (Apache-2.0) for the
//! next-code Grok Face migration (PR7).
//!
//! Upstream creates CoW worktrees with SQLite metadata. This stub only
//! reproduces the disk-space constants and (behind `metadata`) the DB types
//! the pager imports.

/// Human-readable context string attached when worktree creation hits ENOSPC.
pub const OUT_OF_DISK_CONTEXT: &str = "not enough free disk space";

/// OS error message substring for "no space left on device".
pub const ENOSPC_OS_MESSAGE: &str = "No space left on device";

#[cfg(feature = "metadata")]
pub mod db;

#[cfg(feature = "metadata")]
pub use db::{WorktreeDb, WorktreeKind, WorktreeRecord, WorktreeStatus};
