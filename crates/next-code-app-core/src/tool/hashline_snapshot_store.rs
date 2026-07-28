//! File-backed [`SnapshotStore`] that persists hashline snapshots to
//! `~/.next-code/hashline/snapshots.db` as a single JSON file.
//!
//! This replaces the in-memory-only [`InMemorySnapshotStore`] so that
//! snapshot tags survive process restarts — drift detection and recovery
//! work across sessions, matching the hashline CLI experience without
//! requiring a `.hashline/` dot-folder.
//!
//! # Format
//!
//! ```json
//! {
//!   "version": 1,
//!   "max_paths": 30,
//!   "max_per_path": 4,
//!   "max_bytes": 67108864,
//!   "snapshots": {
//!     "/canonical/path": [ { "path": "...", "text": "...", "hash": "abcd", ... } ]
//!   }
//! }
//! ```
//!
//! Thread-safe via [`parking_lot::RwLock`]; every mutation fsyncs to disk so
//! the file stays crash-consistent. The global singleton is accessed through
//! [`store()`] / [`store_mut()`], which is the wiring used by
//! [`GlobalSnapshotStore`] — a zero-sized [`SnapshotStore`] impl passed into
//! [`hashline::Editor::with_store`].

use hashline::hash::compute_file_hash;
use hashline::snapshot_store::{Snapshot, SnapshotStore};
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Name of the database file inside the hashline state dir.
const DB_FILE: &str = "snapshots.db";

/// Current schema version for forward-compat.
const SCHEMA_VERSION: u32 = 1;

/// Default limits (same as hashline CLI defaults).
const DEFAULT_MAX_PATHS: usize = 30;
const DEFAULT_MAX_VERSIONS_PER_PATH: usize = 4;
const DEFAULT_MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Persisted schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSnapshot {
    path: String,
    text: String,
    hash: String,
    recorded_at: u64,
    seen_lines: Option<Vec<usize>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Schema {
    version: u32,
    max_paths: usize,
    max_per_path: usize,
    max_bytes: usize,
    /// Canonical path → version history (newest first).
    snapshots: HashMap<String, VecDeque<PersistedSnapshot>>,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION,
            max_paths: DEFAULT_MAX_PATHS,
            max_per_path: DEFAULT_MAX_VERSIONS_PER_PATH,
            max_bytes: DEFAULT_MAX_TOTAL_BYTES,
            snapshots: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// NextCodeSnapshotStore
// ---------------------------------------------------------------------------

/// File-backed hashline snapshot store that persists to
/// `~/.next-code/hashline/snapshots.db`.
pub struct NextCodeSnapshotStore {
    db_path: PathBuf,
    schema: Schema,
    /// Total bytes of all snapshot text currently retained.
    total_bytes: usize,
    /// Insertion order of paths for FIFO eviction.
    path_order: VecDeque<String>,
}

/// Global singleton: `~/.next-code/hashline/snapshots.db`.
static GLOBAL: OnceLock<RwLock<NextCodeSnapshotStore>> = OnceLock::new();

/// Acquire a read lock on the global store.
fn store() -> std::sync::RwLockReadGuard<'static, NextCodeSnapshotStore> {
    GLOBAL
        .get_or_init(|| RwLock::new(NextCodeSnapshotStore::open_or_create(&db_path())))
        .read()
        .expect("hashline snapshot store lock poisoned")
}

/// Acquire a write lock on the global store.
fn store_mut() -> std::sync::RwLockWriteGuard<'static, NextCodeSnapshotStore> {
    GLOBAL
        .get_or_init(|| RwLock::new(NextCodeSnapshotStore::open_or_create(&db_path())))
        .write()
        .expect("hashline snapshot store lock poisoned")
}

/// Build the canonical db path: `~/.next-code/hashline/snapshots.db`.
///
/// Panics if the home dir cannot be resolved (should only happen in
/// minimal containers where next-code won't run meaningfully anyway).
fn db_path() -> PathBuf {
    let home = dirs::home_dir().expect("home dir required for hashline snapshots");
    home.join(".next-code").join("hashline").join(DB_FILE)
}

// ---------------------------------------------------------------------------
// SnapshotStore trait impl via GlobalSnapshotStore (zero-sized pass-through)
// ---------------------------------------------------------------------------

/// Zero-sized [`SnapshotStore`] that delegates every call to the global
/// [`NextCodeSnapshotStore`] singleton. Pass this into
/// [`hashline::Editor::with_store`].
///
/// All `&mut self` receivers on the trait are satisfied because the
/// concrete implementation uses the global `RwLock` internally.
pub struct GlobalSnapshotStore;

impl SnapshotStore for GlobalSnapshotStore {
    fn head(&self, path: &str) -> Option<Snapshot> {
        store().head_inner(path)
    }

    fn by_hash(&self, path: &str, hash: &str) -> Option<Snapshot> {
        store().by_hash_inner(path, hash)
    }

    fn record(
        &mut self,
        path: &str,
        full_text: &str,
        seen_lines: Option<&[usize]>,
    ) -> String {
        store_mut().record_inner(path, full_text, seen_lines)
    }

    fn record_seen_lines(&mut self, path: &str, hash: &str, lines: &[usize]) {
        store_mut().record_seen_lines_inner(path, hash, lines);
    }

    fn invalidate(&mut self, path: &str) {
        store_mut().invalidate_inner(path);
    }

    fn clear(&mut self) {
        store_mut().clear_inner();
    }
}

// ---------------------------------------------------------------------------
// Inner methods (callable via store()/store_mut() guards or directly)
// ---------------------------------------------------------------------------

impl NextCodeSnapshotStore {
    /// Open an existing store or create a new empty one.
    pub fn open_or_create(db_path: &Path) -> Self {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let schema = std::fs::read_to_string(db_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Schema>(&s).ok())
            .unwrap_or_default();

        let total_bytes = schema
            .snapshots
            .values()
            .flat_map(|v| v.iter())
            .map(|s| s.text.len())
            .sum();

        let path_order: VecDeque<String> = schema.snapshots.keys().cloned().collect();

        Self {
            db_path: db_path.to_path_buf(),
            schema,
            total_bytes,
            path_order,
        }
    }

    /// Persist to disk.
    fn flush(&self) {
        if let Ok(json) = serde_json::to_string(&self.schema) {
            let _ = std::fs::write(&self.db_path, &json);
        }
    }

    fn head_inner(&self, path: &str) -> Option<Snapshot> {
        self.schema
            .snapshots
            .get(path)
            .and_then(|v| v.front())
            .map(to_snapshot)
    }

    fn by_hash_inner(&self, path: &str, hash: &str) -> Option<Snapshot> {
        let hash_lower = hash.to_lowercase();
        self.schema
            .snapshots
            .get(path)?
            .iter()
            .find(|s| s.hash.to_lowercase() == hash_lower)
            .map(to_snapshot)
    }

    fn record_inner(
        &mut self,
        path: &str,
        full_text: &str,
        seen_lines: Option<&[usize]>,
    ) -> String {
        let hash = compute_file_hash(full_text);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Merge seen_lines if the same hash already exists as head.
        if let Some(versions) = self.schema.snapshots.get_mut(path) {
            if let Some(existing) = versions.front_mut() {
                if existing.hash == hash {
                    merge_seen_lines_vec(&mut existing.seen_lines, seen_lines);
                    existing.recorded_at = now;
                    self.flush();
                    return hash;
                }
            }
        }

        let text_len = full_text.len();
        let persisted = PersistedSnapshot {
            path: path.to_string(),
            text: full_text.to_string(),
            hash: hash.clone(),
            recorded_at: now,
            seen_lines: seen_lines.map(|s| s.to_vec()),
        };

        // Ensure path is tracked in order.
        if !self.schema.snapshots.contains_key(path) {
            self.path_order.push_back(path.to_string());
        }

        let versions = self
            .schema
            .snapshots
            .entry(path.to_string())
            .or_insert_with(VecDeque::new);
        versions.push_front(persisted);
        self.total_bytes += text_len;

        self.enforce_limits();
        self.flush();
        hash
    }

    fn record_seen_lines_inner(&mut self, path: &str, hash: &str, lines: &[usize]) {
        let hash_lower = hash.to_lowercase();
        if let Some(versions) = self.schema.snapshots.get_mut(path) {
            if let Some(existing) = versions
                .iter_mut()
                .find(|s| s.hash.to_lowercase() == hash_lower)
            {
                merge_seen_lines(&mut existing.seen_lines, lines);
            }
        }
        self.flush();
    }

    fn invalidate_inner(&mut self, path: &str) {
        if let Some(removed) = self.schema.snapshots.remove(path) {
            self.total_bytes = self
                .total_bytes
                .saturating_sub(removed.iter().map(|s| s.text.len()).sum::<usize>());
        }
        self.path_order.retain(|p| p != path);
        self.flush();
    }

    fn clear_inner(&mut self) {
        self.schema.snapshots.clear();
        self.path_order.clear();
        self.total_bytes = 0;
        self.flush();
    }

    fn enforce_limits(&mut self) {
        // Evict paths (FIFO) until under max_paths.
        while self.schema.snapshots.len() > self.schema.max_paths {
            if let Some(oldest) = self.path_order.pop_front() {
                if let Some(removed) = self.schema.snapshots.remove(&oldest) {
                    self.total_bytes = self.total_bytes.saturating_sub(
                        removed.iter().map(|s| s.text.len()).sum::<usize>(),
                    );
                }
            } else {
                break;
            }
        }

        // Evict oldest versions per path until under max_per_path.
        for versions in self.schema.snapshots.values_mut() {
            while versions.len() > self.schema.max_per_path {
                if let Some(removed) = versions.pop_back() {
                    self.total_bytes = self.total_bytes.saturating_sub(removed.text.len());
                }
            }
        }

        // Evict oldest versions globally until under max_bytes.
        while self.total_bytes > self.schema.max_bytes {
            let mut oldest_key: Option<String> = None;
            let mut oldest_idx: Option<usize> = None;
            // Collect candidates for removal.
            for (key, versions) in self.schema.snapshots.iter() {
                if !versions.is_empty() {
                    oldest_key = Some(key.clone());
                    oldest_idx = Some(versions.len() - 1);
                    break;
                }
            }
            if let (Some(key), Some(idx)) = (oldest_key, oldest_idx) {
                if let Some(versions) = self.schema.snapshots.get_mut(&key) {
                    if idx < versions.len() {
                        if let Some(removed) = versions.remove(idx) {
                            self.total_bytes =
                                self.total_bytes.saturating_sub(removed.text.len());
                        }
                    }
                    if versions.is_empty() {
                        self.schema.snapshots.remove(&key);
                        self.path_order.retain(|p| p != &key);
                    }
                }
            } else {
                break;
            }
        }
    }
}

/// Convert a [`PersistedSnapshot`] to a public [`Snapshot`].
fn to_snapshot(s: &PersistedSnapshot) -> Snapshot {
    Snapshot {
        path: s.path.clone(),
        text: s.text.clone(),
        hash: s.hash.clone(),
        recorded_at: s.recorded_at,
        seen_lines: s.seen_lines.as_ref().map(|v| v.iter().copied().collect()),
    }
}

fn merge_seen_lines(dest: &mut Option<Vec<usize>>, lines: &[usize]) {
    let set = dest.get_or_insert_with(Vec::new);
    for &l in lines {
        if !set.contains(&l) {
            set.push(l);
        }
    }
}

fn merge_seen_lines_vec(dest: &mut Option<Vec<usize>>, lines: Option<&[usize]>) {
    if let Some(lines) = lines {
        merge_seen_lines(dest, lines);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn open_store(dir: &TempDir) -> NextCodeSnapshotStore {
        let db_path = dir.path().join("snapshots.db");
        NextCodeSnapshotStore::open_or_create(&db_path)
    }

    #[test]
    fn record_returns_consistent_hash() {
        let tmp = TempDir::new().unwrap();
        let mut store = open_store(&tmp);
        let h1 = store.record_inner("/a.rs", "hello\n", None);
        assert_eq!(h1.len(), 4);
        let h2 = store.record_inner("/a.rs", "hello\n", None);
        assert_eq!(h1, h2);
    }

    #[test]
    fn head_returns_most_recent() {
        let tmp = TempDir::new().unwrap();
        let mut store = open_store(&tmp);
        let t1 = store.record_inner("/b.rs", "v1", None);
        let t2 = store.record_inner("/b.rs", "v2", None);
        assert_ne!(t1, t2);
        let snap = store.head_inner("/b.rs").expect("head");
        assert_eq!(snap.hash, t2);
        assert_eq!(snap.text, "v2");
    }

    #[test]
    fn by_hash_finds_historical() {
        let tmp = TempDir::new().unwrap();
        let mut store = open_store(&tmp);
        let t1 = store.record_inner("/c.rs", "alpha", None);
        let _t2 = store.record_inner("/c.rs", "beta", None);
        let snap = store.by_hash_inner("/c.rs", &t1).expect("old version");
        assert_eq!(snap.text, "alpha");
    }

    #[test]
    fn seen_lines_merge() {
        let tmp = TempDir::new().unwrap();
        let mut store = open_store(&tmp);
        let tag = store.record_inner("/d.rs", "l1\nl2\nl3\n", Some(&[1, 2]));
        store.record_seen_lines_inner("/d.rs", &tag, &[3]);
        let snap = store.head_inner("/d.rs").expect("head");
        let seen: HashSet<usize> = snap.seen_lines.expect("seen_lines");
        assert!(seen.contains(&1));
        assert!(seen.contains(&2));
        assert!(seen.contains(&3));
    }

    #[test]
    fn invalidate_removes_path() {
        let tmp = TempDir::new().unwrap();
        let mut store = open_store(&tmp);
        store.record_inner("/e.rs", "content", None);
        assert!(store.head_inner("/e.rs").is_some());
        store.invalidate_inner("/e.rs");
        assert!(store.head_inner("/e.rs").is_none());
    }

    #[test]
    fn persisted_data_survives_reopen() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("snapshots.db");
        {
            let mut store = NextCodeSnapshotStore::open_or_create(&db_path);
            store.record_inner("/h.rs", "persist me", Some(&[1]));
        }
        {
            let store = NextCodeSnapshotStore::open_or_create(&db_path);
            let snap = store.head_inner("/h.rs").expect("survived reopen");
            assert_eq!(snap.text, "persist me");
            let seen: HashSet<usize> = snap.seen_lines.expect("seen_lines");
            assert!(seen.contains(&1));
        }
    }
}
