//! Git marketplace source support (stub).

use std::path::{Path, PathBuf};

/// Cache sync mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Respect the cache TTL.
    UseTtl,
    /// Always re-fetch.
    Force,
}

/// Lease on a synced marketplace cache directory.
///
/// Upstream also holds a file lock; the stub only exposes `path`.
pub struct SourceCacheLease {
    /// Path to the cached repo.
    pub path: PathBuf,
}

/// Sync a git marketplace source to the persistent cache.
///
/// Stub: always `Err`.
pub fn sync_source_cache(
    _url: &str,
    _branch: Option<&str>,
    _cache_root: &Path,
) -> Result<PathBuf, String> {
    Err("xai-grok-plugin-marketplace stub: git sync unavailable".into())
}

/// Force-sync a git marketplace source.
///
/// Stub: always `Err`.
pub fn force_sync_source_cache(
    _url: &str,
    _branch: Option<&str>,
    _cache_root: &Path,
) -> Result<PathBuf, String> {
    Err("xai-grok-plugin-marketplace stub: git sync unavailable".into())
}

/// Sync with an explicit [`SyncMode`].
///
/// Stub: always `Err`.
pub fn sync_source_cache_with_mode(
    _url: &str,
    _branch: Option<&str>,
    _cache_root: &Path,
    _mode: SyncMode,
) -> Result<SourceCacheLease, String> {
    Err("xai-grok-plugin-marketplace stub: git sync unavailable".into())
}

/// Default cache root directory.
///
/// Stub: returns a relative placeholder path (no I/O).
pub fn default_cache_root() -> PathBuf {
    PathBuf::from(".grok").join("marketplace-cache")
}

/// Build a `git` command.
pub fn git_command() -> std::process::Command {
    std::process::Command::new("git")
}
