//! SQLite-backed metadata database for tracking worktrees (stub).

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// Kind of worktree tracked in the DB.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorktreeKind {
    Session,
    Ab,
    Pool,
    Fork,
    Manual,
    Subagent,
}

impl WorktreeKind {
    /// Stable string form.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Ab => "ab",
            Self::Pool => "pool",
            Self::Fork => "fork",
            Self::Manual => "manual",
            Self::Subagent => "subagent",
        }
    }

    /// Parse with a Manual fallback for unknown values.
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "session" => Self::Session,
            "ab" => Self::Ab,
            "pool" => Self::Pool,
            "fork" => Self::Fork,
            "manual" => Self::Manual,
            "subagent" => Self::Subagent,
            _ => Self::Manual,
        }
    }
}

/// Liveness status of a tracked worktree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorktreeStatus {
    Alive,
    Dead,
}

impl WorktreeStatus {
    /// Stable string form.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Alive => "alive",
            Self::Dead => "dead",
        }
    }

    /// Parse with a Dead fallback for unknown values.
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "alive" => Self::Alive,
            "dead" => Self::Dead,
            _ => Self::Dead,
        }
    }
}

/// One row in the worktree metadata DB.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorktreeRecord {
    /// Stable worktree id.
    pub id: String,
    /// Filesystem path of the worktree.
    pub path: PathBuf,
    /// Source repository path.
    pub source_repo: PathBuf,
    /// Short repo name.
    pub repo_name: String,
    /// Worktree kind.
    pub kind: WorktreeKind,
    /// Creation mode string (e.g. `"linked"`).
    pub creation_mode: String,
    /// Optional git ref.
    pub git_ref: Option<String>,
    /// Optional HEAD commit.
    pub head_commit: Option<String>,
    /// Optional owning session id.
    pub session_id: Option<String>,
    /// Optional creator process id.
    pub creator_pid: Option<u32>,
    /// Creation timestamp (unix seconds).
    pub created_at: i64,
    /// Last-accessed timestamp (unix seconds).
    pub last_accessed_at: Option<i64>,
    /// Liveness status.
    pub status: WorktreeStatus,
    /// Opaque JSON metadata (label, etc.).
    pub metadata: Option<serde_json::Value>,
}

/// Filter for [`WorktreeDb::list`].
#[derive(Default)]
pub struct ListFilter {
    /// Filter by repo name.
    pub repo_name: Option<String>,
    /// Filter by source repo path.
    pub source_repo: Option<PathBuf>,
    /// Filter by kind.
    pub kind: Option<WorktreeKind>,
    /// Filter by status.
    pub status: Option<WorktreeStatus>,
    /// Include dead records.
    pub include_dead: bool,
}

/// Stub worktree metadata DB (no SQLite).
pub struct WorktreeDb;

impl WorktreeDb {
    /// Open the default DB at `~/.grok/worktrees.db`.
    ///
    /// Stub: always `Err`.
    pub fn open_default() -> Result<Self> {
        bail!("xai-fast-worktree stub: metadata DB unavailable")
    }

    /// Open (or create) the DB under `grok_home`.
    ///
    /// Stub: always `Err`.
    pub fn open(_grok_home: &Path) -> Result<Self> {
        bail!("xai-fast-worktree stub: metadata DB unavailable")
    }

    /// Look up a record by id or path.
    ///
    /// Stub: always `Err`.
    pub fn get(&self, _id_or_path: &str) -> Result<Option<WorktreeRecord>> {
        bail!("xai-fast-worktree stub: metadata DB unavailable")
    }

    /// List records matching `filter`.
    ///
    /// Stub: always `Err`.
    pub fn list(&self, _filter: &ListFilter) -> Result<Vec<WorktreeRecord>> {
        bail!("xai-fast-worktree stub: metadata DB unavailable")
    }
}
