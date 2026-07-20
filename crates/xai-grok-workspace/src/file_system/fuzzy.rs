//! Empty fuzzy matcher stubs (nucleo Utf32String path for pager test fixtures).

use std::path::{Path, PathBuf};
use std::sync::Arc;

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

pub struct FuzzyFileMatcher {
    _root: PathBuf,
}

impl FuzzyFileMatcher {
    pub fn new(root: &Path) -> Self {
        Self {
            _root: root.to_owned(),
        }
    }

    pub fn set_query(&mut self, _query: &str, _dirs: bool) {}

    pub fn restart_walk(&mut self) {}
}

pub struct FuzzyFileMatcherDaemon {
    results: FuzzyMatcherDaemonResults,
    generation: usize,
}

impl FuzzyFileMatcherDaemon {
    pub fn new(_matcher: FuzzyFileMatcher, _topk: usize) -> Self {
        Self {
            results: FuzzyMatcherDaemonResults::default(),
            generation: 0,
        }
    }

    pub fn restart_walk(&mut self, _hidden: bool) {
        self.generation += 1;
        self.results = FuzzyMatcherDaemonResults {
            generation: self.generation,
            ..FuzzyMatcherDaemonResults::default()
        };
    }

    pub fn set_query(&mut self, _query: &str, _dirs: bool) {
        self.generation += 1;
        self.results.generation = self.generation;
        self.results.status.done = true;
    }

    pub fn get(&self) -> FuzzyMatcherDaemonResults {
        self.results.clone()
    }
}
