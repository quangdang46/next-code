//! Face `@` fuzzy file matcher backed by next-code's `ffs-search` FilePicker.
//!
//! Maps the Grok Face daemon API (`restart_walk` / `set_query` / `get`) onto
//! the same SharedFilePicker + fuzzy_search pipelines used by TUI `AtPicker`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ffs_search::{
    FfsMode, FilePicker, FilePickerOptions, FuzzySearchOptions, MixedItemRef, PaginationArgs,
    QueryParser, SharedFilePicker, SharedFrecency,
};

#[derive(Debug, Clone, Default)]
pub struct FuzzyMatchResult {
    pub path: nucleo::Utf32String,
    pub score: u32,
    pub indices: Vec<u32>,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FuzzyMatcherStatus {
    pub changed: bool,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct FuzzyMatcherDaemonResults {
    pub topk: Arc<[FuzzyMatchResult]>,
    pub num_items: usize,
    pub status: FuzzyMatcherStatus,
    pub generation: usize,
}

impl Default for FuzzyMatcherDaemonResults {
    fn default() -> Self {
        Self {
            topk: Arc::from([]),
            num_items: 0,
            status: FuzzyMatcherStatus {
                changed: false,
                done: true,
            },
            generation: 0,
        }
    }
}

impl AsRef<[FuzzyMatchResult]> for FuzzyMatcherDaemonResults {
    fn as_ref(&self) -> &[FuzzyMatchResult] {
        self.topk.as_ref()
    }
}

/// Thin root holder — Face constructs this then moves it into the daemon.
pub struct FuzzyFileMatcher {
    root: PathBuf,
}

impl FuzzyFileMatcher {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
        }
    }

    pub fn set_query(&mut self, _query: &str, _dirs: bool) {}

    pub fn restart_walk(&mut self) {}
}

/// Background-friendly ffs bridge. Scan runs on ffs worker threads; Face polls
/// [`Self::get`] on the UI tick (~4ms) which re-queries once the index is ready.
pub struct FuzzyFileMatcherDaemon {
    root: PathBuf,
    topk: usize,
    shared_picker: SharedFilePicker,
    query: String,
    dirs_only: bool,
    generation: usize,
    results: FuzzyMatcherDaemonResults,
    /// Last (generation, query, dirs_only, scan_done) applied into `results`.
    last_applied: Option<(usize, String, bool, bool)>,
}

impl FuzzyFileMatcherDaemon {
    pub fn new(matcher: FuzzyFileMatcher, topk: usize) -> Self {
        let mut daemon = Self {
            root: matcher.root,
            topk: topk.max(1),
            shared_picker: SharedFilePicker::default(),
            query: String::new(),
            dirs_only: false,
            generation: 0,
            results: FuzzyMatcherDaemonResults::default(),
            last_applied: None,
        };
        daemon.spawn_picker();
        daemon
    }

    pub fn restart_walk(&mut self, _hidden: bool) {
        // ffs WalkBuilder always respects gitignore; Face `!` hidden mode is
        // best-effort (generation bump + fresh scan) until ffs exposes a toggle.
        self.generation = self.generation.saturating_add(1);
        self.last_applied = None;
        self.results = FuzzyMatcherDaemonResults {
            generation: self.generation,
            status: FuzzyMatcherStatus {
                changed: true,
                done: false,
            },
            ..FuzzyMatcherDaemonResults::default()
        };
        self.spawn_picker();
    }

    pub fn set_query(&mut self, query: &str, dirs: bool) {
        self.generation = self.generation.saturating_add(1);
        self.query = query.to_owned();
        // Face always passes `dirs=false`; dir-only is also implied by a
        // trailing `/` on the matcher query (see AtContext::is_dir_mode).
        self.dirs_only = dirs || query.ends_with('/');
        self.last_applied = None;
        self.results.generation = self.generation;
        self.results.status.done = false;
        self.results.status.changed = true;
        self.refresh();
    }

    /// Snapshot current results, refreshing from ffs when the index/query changed.
    pub fn get(&mut self) -> FuzzyMatcherDaemonResults {
        self.refresh();
        self.results.clone()
    }

    fn spawn_picker(&mut self) {
        // Drop the previous SharedFilePicker so its scan/watcher can wind down.
        self.shared_picker = SharedFilePicker::default();
        let opts = FilePickerOptions {
            base_path: self.root.to_string_lossy().into_owned(),
            mode: FfsMode::Ai,
            enable_mmap_cache: false,
            enable_content_indexing: false,
            cache_budget: None,
            watch: true,
            follow_symlinks: false,
        };
        if let Err(err) = FilePicker::new_with_shared_state(
            self.shared_picker.clone(),
            SharedFrecency::default(),
            opts,
        ) {
            // Surface a finished empty generation so Face can stop waiting.
            self.results = FuzzyMatcherDaemonResults {
                topk: Arc::from([]),
                num_items: 0,
                status: FuzzyMatcherStatus {
                    changed: true,
                    done: true,
                },
                generation: self.generation,
            };
            let _ = err;
        }
    }

    fn refresh(&mut self) {
        let Ok(guard) = self.shared_picker.read() else {
            return;
        };
        let Some(picker) = guard.as_ref() else {
            // Background thread has not installed the picker yet.
            self.results.status.done = false;
            return;
        };

        let scan_done = !picker.is_scan_active();
        let apply_key = (
            self.generation,
            self.query.clone(),
            self.dirs_only,
            scan_done,
        );
        if self.last_applied.as_ref() == Some(&apply_key) {
            self.results.status.changed = false;
            return;
        }

        // While the first scan is still empty, keep `done=false` so Face skips
        // empty intermediate snapshots (see FileSearchState::poll).
        if !scan_done && picker.get_files().is_empty() {
            self.results.status.done = false;
            self.results.generation = self.generation;
            return;
        }

        let limit = self.topk;
        let dirs_only = self.dirs_only;
        let query_str = self.query.as_str();
        let parser = QueryParser::default();
        let parsed = parser.parse(query_str);
        let opts = FuzzySearchOptions {
            pagination: PaginationArgs {
                offset: 0,
                limit,
            },
            ..Default::default()
        };

        let (mut matches, num_items) = if dirs_only {
            let result = picker.fuzzy_search_directories(&parsed, opts);
            let items: Vec<FuzzyMatchResult> = result
                .items
                .iter()
                .zip(result.scores.iter())
                .map(|(item, score)| {
                    let path = item.relative_path(picker);
                    to_match_result(&path, score.total, true, query_str)
                })
                .collect();
            (items, result.total_matched.max(result.total_dirs))
        } else {
            let result = picker.fuzzy_search_mixed(&parsed, None, opts);
            let items: Vec<FuzzyMatchResult> = result
                .items
                .iter()
                .zip(result.scores.iter())
                .map(|(item, score)| match item {
                    MixedItemRef::File(f) => {
                        let path = f.relative_path(picker);
                        to_match_result(&path, score.total, false, query_str)
                    }
                    MixedItemRef::Dir(d) => {
                        let path = d.relative_path(picker);
                        to_match_result(&path, score.total, true, query_str)
                    }
                })
                .collect();
            (
                items,
                result
                    .total_matched
                    .max(result.total_files.saturating_add(result.total_dirs)),
            )
        };

        // Cap in case mixed search returns more than topk after merge.
        if matches.len() > limit {
            matches.truncate(limit);
        }

        self.results = FuzzyMatcherDaemonResults {
            topk: Arc::from(matches),
            num_items,
            status: FuzzyMatcherStatus {
                changed: true,
                done: scan_done,
            },
            generation: self.generation,
        };
        self.last_applied = Some(apply_key);
    }
}

fn to_match_result(path: &str, score: i32, is_dir: bool, query: &str) -> FuzzyMatchResult {
    FuzzyMatchResult {
        path: nucleo::Utf32String::from(path),
        score: score.max(0) as u32,
        indices: best_effort_match_indices(query, path),
        is_dir,
    }
}

/// Greedy case-insensitive subsequence highlight indices (char offsets).
/// ffs/frizbee does not expose match positions yet (same gap as MentionResolver).
fn best_effort_match_indices(query: &str, path: &str) -> Vec<u32> {
    let needle: String = query
        .chars()
        .filter(|c| *c != '/' && *c != '\\')
        .collect();
    if needle.is_empty() {
        return Vec::new();
    }

    let mut indices = Vec::with_capacity(needle.chars().count());
    let mut path_iter = path.char_indices().enumerate();
    for qc in needle.chars() {
        let target = qc.to_ascii_lowercase();
        let mut found = false;
        while let Some((char_idx, (_byte, ch))) = path_iter.next() {
            if ch.to_ascii_lowercase() == target {
                indices.push(char_idx as u32);
                found = true;
                break;
            }
        }
        if !found {
            break;
        }
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{Duration, Instant};

    /// Block until the ffs scan finishes (or timeout).
    fn wait_scan(daemon: &FuzzyFileMatcherDaemon, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if daemon
                .shared_picker
                .wait_for_scan(Duration::from_millis(50))
            {
                let Ok(guard) = daemon.shared_picker.read() else {
                    continue;
                };
                if guard.as_ref().is_some_and(|p| !p.is_scan_active()) {
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    fn setup_tree() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().expect("tempdir");
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"t\"\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src").join("lib.rs"), "// lib").unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::write(dir.path().join("docs").join("README.md"), "# docs").unwrap();
        dir
    }

    #[test]
    fn daemon_returns_files_after_scan() {
        let tree = setup_tree();
        let mut daemon =
            FuzzyFileMatcherDaemon::new(FuzzyFileMatcher::new(tree.path()), 50);
        daemon.restart_walk(false);
        daemon.set_query("", false);
        assert!(
            wait_scan(&daemon, Duration::from_secs(10)),
            "ffs scan timed out"
        );

        let mut found = false;
        for _ in 0..50 {
            let snap = daemon.get();
            if snap.status.done && !snap.topk.is_empty() {
                found = snap
                    .topk
                    .iter()
                    .any(|r| r.path.to_string().contains("main.rs"));
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(found, "expected main.rs in @ results");
    }

    #[test]
    fn daemon_filters_by_query() {
        let tree = setup_tree();
        let mut daemon =
            FuzzyFileMatcherDaemon::new(FuzzyFileMatcher::new(tree.path()), 50);
        daemon.restart_walk(false);
        assert!(wait_scan(&daemon, Duration::from_secs(10)));

        daemon.set_query("Cargo", false);
        let snap = daemon.get();
        assert!(snap.status.done);
        assert!(
            snap.topk
                .iter()
                .any(|r| r.path.to_string().contains("Cargo.toml")),
            "expected Cargo.toml, got {:?}",
            snap.topk
                .iter()
                .map(|r| r.path.to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn daemon_dirs_only_on_trailing_slash() {
        let tree = setup_tree();
        let mut daemon =
            FuzzyFileMatcherDaemon::new(FuzzyFileMatcher::new(tree.path()), 50);
        daemon.restart_walk(false);
        assert!(wait_scan(&daemon, Duration::from_secs(10)));

        daemon.set_query("docs/", false);
        let snap = daemon.get();
        assert!(snap.status.done);
        assert!(
            !snap.topk.is_empty(),
            "expected directory hits for docs/"
        );
        assert!(
            snap.topk.iter().all(|r| r.is_dir),
            "trailing slash should yield dirs only: {:?}",
            snap.topk
                .iter()
                .map(|r| (r.path.to_string(), r.is_dir))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn best_effort_indices_highlight_subsequence() {
        let idx = best_effort_match_indices("mr", "src/main.rs");
        assert_eq!(idx, vec![4, 9]);
    }
}
