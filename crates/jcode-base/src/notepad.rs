//! 3-tier file-based notepad for compaction-resistant context notes.
//!
//! The notepad stores short text notes across three tiers, each backed by
//! a plain markdown file in `<working_dir>/.jcode/notepad/`:
//!
//! - **Priority** – critical context that is injected into the system prompt
//!   every turn, surviving compaction. The model uses it as always-present
//!   context (current goal, key constraints, pinned decisions).
//! - **Working** – scratchpad for the current session. Cleared between
//!   sessions. Not injected automatically.
//! - **Manual** – user-authored notes that persist across sessions. Not
//!   injected automatically.
//!
//! # Concurrency safety
//!
//! Writes use an advisory lockfile (`<notepad_dir>/.lock`) so concurrent
//! agent instances do not corrupt each other's data. A 5-second timeout
//! prevents deadlocks. The actual write is atomic on the same filesystem
//! (write to `.tmp`, then `rename`).

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Tiers
// ---------------------------------------------------------------------------

/// The three notepad tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotepadTier {
    /// Injected into the system prompt every turn.
    Priority,
    /// Session-scoped scratchpad.
    Working,
    /// Persistent user notes.
    Manual,
}

impl NotepadTier {
    /// Human-readable label for the tier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Priority => "priority",
            Self::Working => "working",
            Self::Manual => "manual",
        }
    }

    /// The file name used on disk (e.g. `priority.md`).
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Priority => "priority.md",
            Self::Working => "working.md",
            Self::Manual => "manual.md",
        }
    }

    /// All tiers for iteration.
    pub fn all() -> &'static [NotepadTier] {
        &[Self::Priority, Self::Working, Self::Manual]
    }
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Notepad subsection of the main [`Config`](crate::config::Config).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotepadConfig {
    /// Whether the notepad feature is enabled (default: `true`).
    pub enabled: bool,

    /// Directory for notepad files, relative to the working directory
    /// (default: `.jcode/notepad`).
    pub dir: String,

    /// Maximum characters for a single tier's content (default: 4096).
    pub max_chars_per_tier: usize,
}

impl Default for NotepadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dir: ".jcode/notepad".to_string(),
            max_chars_per_tier: 4096,
        }
    }
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

/// Summary of the notepad file state.
#[derive(Debug, Clone, Serialize)]
pub struct NotepadStats {
    /// Whether any notepad files exist.
    pub exists: bool,
    /// Total bytes across all tier files.
    pub total_size_bytes: u64,
    /// Per-tier breakdown.
    pub tiers: Vec<TierStats>,
}

/// Per-tier statistics.
#[derive(Debug, Clone, Serialize)]
pub struct TierStats {
    pub name: &'static str,
    pub file_size_bytes: u64,
    pub has_content: bool,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

const LOCK_TIMEOUT: Duration = Duration::from_millis(5000);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(25);

/// The notepad engine — reads/writes/clears tiered note files on disk.
///
/// Writes are serialized by an advisory lockfile and are atomic
/// (tmp+rename). Reads are lock-free (last-writer-wins semantics are
/// acceptable for note content).
pub struct Notepad {
    base_dir: PathBuf,
    enabled: bool,
    max_chars_per_tier: usize,
}

impl Notepad {
    /// Create a new `Notepad` if enabled, or returns `None`.
    ///
    /// `working_dir` is the session's working directory; the note files are
    /// placed under `<working_dir>/<config.dir>`.
    pub fn new(working_dir: Option<&Path>, config: &NotepadConfig) -> Option<Self> {
        if !config.enabled {
            return None;
        }
        let base = working_dir
            .map(|wd| wd.join(&config.dir))
            .unwrap_or_else(|| PathBuf::from(&config.dir));
        Some(Self {
            base_dir: base,
            enabled: true,
            max_chars_per_tier: config.max_chars_per_tier,
        })
    }

    // -- helpers -----------------------------------------------------------

    fn tier_path(&self, tier: NotepadTier) -> PathBuf {
        self.base_dir.join(tier.filename())
    }

    fn lock_path(&self) -> PathBuf {
        self.base_dir.join(".lock")
    }

    /// Acquire an advisory exclusive lock via a lockfile.
    ///
    /// Creates the lockfile with `create_new(true)` and spins until
    /// the timeout expires. The returned guard removes the lockfile on
    /// drop, so no explicit unlock is needed.
    fn acquire_lock(&self) -> Result<LockGuard, NotepadError> {
        let lp = self.lock_path();
        let deadline = Instant::now() + LOCK_TIMEOUT;
        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lp)
            {
                Ok(_) => return Ok(LockGuard(lp)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if Instant::now() >= deadline {
                        return Err(NotepadError::LockTimeout);
                    }
                    std::thread::sleep(LOCK_RETRY_INTERVAL);
                }
                Err(e) => return Err(NotepadError::Io(e)),
            }
        }
    }

    /// Atomic write: write to `.tmp`, then rename (atomic on same fs).
    fn atomic_write(path: &Path, content: &[u8]) -> std::io::Result<()> {
        let tmp = path.with_extension("tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(content)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Ensure the base directory exists (called lazily before writes).
    fn ensure_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_dir)
    }

    // -- public API --------------------------------------------------------

    /// Read the content of a tier. Returns an empty string if the file does
    /// not exist or cannot be read.
    pub fn read(&self, tier: NotepadTier) -> String {
        if !self.enabled {
            return String::new();
        }
        fs::read_to_string(self.tier_path(tier)).unwrap_or_default()
    }

    /// Write `content` to a tier, truncating if `max_chars_per_tier` is
    /// exceeded. Returns an error if the write fails.
    ///
    /// The write is serialized by an advisory lock and is atomic
    /// (tmp+rename) to prevent partial reads.
    pub fn write(&self, tier: NotepadTier, content: &str) -> Result<(), NotepadError> {
        if !self.enabled {
            return Ok(());
        }
        let _guard = self.acquire_lock()?;
        self.ensure_dir()?;

        let path = self.tier_path(tier);
        let truncated = if content.len() > self.max_chars_per_tier {
            &content[..content.floor_char_boundary(self.max_chars_per_tier)]
        } else {
            content
        };
        Self::atomic_write(&path, truncated.as_bytes()).map_err(NotepadError::Io)
    }

    /// Clear a tier's content (write an empty string).
    pub fn clear(&self, tier: NotepadTier) -> Result<(), NotepadError> {
        self.write(tier, "")
    }

    /// Read the priority tier and format it as a prompt block suitable for
    /// system-prompt injection. Returns `None` when the tier is empty or
    /// the notepad is disabled.
    pub fn priority_prompt_block(&self) -> Option<String> {
        let content = self.read(NotepadTier::Priority);
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return None;
        }
        Some(format!("# Priority Notes\n\n{}", trimmed))
    }

    /// The resolved base directory for notepad files.
    pub fn dir(&self) -> &Path {
        &self.base_dir
    }

    /// Collect file statistics for all three tiers.
    ///
    /// Does not acquire any lock — sizes are best-effort snapshots.
    pub fn stats(&self) -> NotepadStats {
        let mut total = 0u64;
        let mut tier_stats = Vec::with_capacity(3);
        for tier in NotepadTier::all() {
            let path = self.tier_path(*tier);
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            total += size;
            tier_stats.push(TierStats {
                name: tier.as_str(),
                file_size_bytes: size,
                has_content: size > 0,
            });
        }
        NotepadStats {
            exists: total > 0,
            total_size_bytes: total,
            tiers: tier_stats,
        }
    }

    /// Clear the working tier (session-scoped scratchpad).
    ///
    /// In per-file tier architecture this is the closest analog to
    /// pruning — working memory is the session-scoped tier that
    /// benefits from periodic cleanup.
    pub fn prune(&self) -> Result<(), NotepadError> {
        self.clear(NotepadTier::Working)
    }
}

// ---------------------------------------------------------------------------
// Lock guard
// ---------------------------------------------------------------------------

/// RAII guard that removes the lockfile on drop.
struct LockGuard(PathBuf);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during notepad operations.
#[derive(Debug)]
pub enum NotepadError {
    /// Another writer held the lock longer than the timeout.
    LockTimeout,
    /// I/O error during file operations.
    Io(std::io::Error),
}

impl std::fmt::Display for NotepadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LockTimeout => write!(f, "notepad lock timeout (another writer is busy)"),
            Self::Io(e) => write!(f, "notepad I/O error: {e}"),
        }
    }
}

impl std::error::Error for NotepadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LockTimeout => None,
            Self::Io(e) => Some(e),
        }
    }
}

impl From<NotepadError> for std::io::Error {
    fn from(e: NotepadError) -> Self {
        match e {
            NotepadError::LockTimeout => {
                std::io::Error::new(std::io::ErrorKind::TimedOut, e.to_string())
            }
            NotepadError::Io(ioe) => ioe,
        }
    }
}

impl From<std::io::Error> for NotepadError {
    fn from(e: std::io::Error) -> Self {
        NotepadError::Io(e)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_notepad() -> (tempfile::TempDir, Notepad) {
        let dir = tempfile::tempdir().unwrap();
        let config = NotepadConfig {
            enabled: true,
            dir: ".notepad".to_string(),
            max_chars_per_tier: 4096,
        };
        let notepad = Notepad::new(Some(dir.path()), &config).unwrap();
        (dir, notepad)
    }

    #[test]
    fn test_read_write_roundtrip() {
        let (_dir, np) = temp_notepad();
        np.write(NotepadTier::Priority, "hello world").unwrap();
        assert_eq!(np.read(NotepadTier::Priority), "hello world");
    }

    #[test]
    fn test_clear() {
        let (_dir, np) = temp_notepad();
        np.write(NotepadTier::Working, "data").unwrap();
        np.clear(NotepadTier::Working).unwrap();
        assert_eq!(np.read(NotepadTier::Working), "");
    }

    #[test]
    fn test_read_nonexistent_returns_empty() {
        let (_dir, np) = temp_notepad();
        assert_eq!(np.read(NotepadTier::Manual), "");
    }

    #[test]
    fn test_priority_prompt_block_returns_none_when_empty() {
        let (_dir, np) = temp_notepad();
        assert!(np.priority_prompt_block().is_none());
    }

    #[test]
    fn test_priority_prompt_block_formats_content() {
        let (_dir, np) = temp_notepad();
        np.write(NotepadTier::Priority, "Keep this in mind").unwrap();
        let block = np.priority_prompt_block().unwrap();
        assert!(block.contains("Priority Notes"));
        assert!(block.contains("Keep this in mind"));
    }

    #[test]
    fn test_disabled_notepad_returns_none() {
        let config = NotepadConfig {
            enabled: false,
            ..Default::default()
        };
        let np = Notepad::new(None, &config);
        assert!(np.is_none());
    }

    #[test]
    fn test_truncation() {
        let dir = tempfile::tempdir().unwrap();
        let config = NotepadConfig {
            enabled: true,
            dir: ".notepad".to_string(),
            max_chars_per_tier: 10,
        };
        let np = Notepad::new(Some(dir.path()), &config).unwrap();
        np.write(NotepadTier::Priority, "this is way too long for the limit")
            .unwrap();
        let content = np.read(NotepadTier::Priority);
        assert!(content.len() <= 10);
    }

    #[test]
    fn test_tier_as_str() {
        assert_eq!(NotepadTier::Priority.as_str(), "priority");
        assert_eq!(NotepadTier::Working.as_str(), "working");
        assert_eq!(NotepadTier::Manual.as_str(), "manual");
    }

    #[test]
    fn test_tier_filename() {
        assert_eq!(NotepadTier::Priority.filename(), "priority.md");
        assert_eq!(NotepadTier::Working.filename(), "working.md");
        assert_eq!(NotepadTier::Manual.filename(), "manual.md");
    }

    #[test]
    fn test_all_tiers() {
        let tiers = NotepadTier::all();
        assert_eq!(tiers.len(), 3);
        assert!(tiers.contains(&NotepadTier::Priority));
        assert!(tiers.contains(&NotepadTier::Working));
        assert!(tiers.contains(&NotepadTier::Manual));
    }

    #[test]
    fn test_stats_empty() {
        let (_dir, np) = temp_notepad();
        let s = np.stats();
        assert!(!s.exists);
        assert_eq!(s.total_size_bytes, 0);
        for t in &s.tiers {
            assert!(!t.has_content);
        }
    }

    #[test]
    fn test_stats_after_write() {
        let (_dir, np) = temp_notepad();
        np.write(NotepadTier::Priority, "hello").unwrap();
        let s = np.stats();
        assert!(s.exists);
        assert!(s.total_size_bytes > 0);
        let priority_tier = s.tiers.iter().find(|t| t.name == "priority").unwrap();
        assert!(priority_tier.has_content);
    }

    #[test]
    fn test_prune_clears_working() {
        let (_dir, np) = temp_notepad();
        np.write(NotepadTier::Working, "session data").unwrap();
        assert!(!np.read(NotepadTier::Working).is_empty());
        np.prune().unwrap();
        assert_eq!(np.read(NotepadTier::Working), "");
    }

    #[test]
    fn test_lock_timeout_surfaces_error() {
        let dir = tempfile::tempdir().unwrap();
        let config = NotepadConfig {
            enabled: true,
            dir: ".notepad".to_string(),
            max_chars_per_tier: 4096,
        };
        let np = Notepad::new(Some(dir.path()), &config).unwrap();

        // Hold the lock by creating the lockfile manually
        let lock_path = np.lock_path();
        let _lock_file = fs::File::create(&lock_path).unwrap();

        // A very tight timeout so the second attempt fails fast (use a
        // notepad with a zero-ish timeout — we just check the error kind).
        let result = np.write(NotepadTier::Priority, "data");
        assert!(result.is_err());
        match result.unwrap_err() {
            NotepadError::LockTimeout => {} // expected
            other => panic!("expected LockTimeout, got {other:?}"),
        }
    }

    #[test]
    fn test_new_does_not_create_dir_unless_write() {
        let dir = tempfile::tempdir().unwrap();
        let config = NotepadConfig {
            enabled: true,
            dir: ".notepad".to_string(),
            max_chars_per_tier: 4096,
        };
        let np = Notepad::new(Some(dir.path()), &config).unwrap();
        // The base_dir should NOT exist yet — we only create it on write.
        assert!(!np.dir().exists());
        // After a write it should exist.
        np.write(NotepadTier::Priority, "hello").unwrap();
        assert!(np.dir().exists());
    }
}
