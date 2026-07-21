//! Foreign-session listing stubs (SUMMARY: empty / skip).

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ForeignSessionTool {
    Claude,
    Codex,
    Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ForeignSessionSource {
    ClaudeCode,
    CodexCli,
    CodexVsCode,
    CodexAtlas,
    CodexChatGpt,
    CursorDesktop,
    CursorCli,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignSessionSummary {
    pub tool: ForeignSessionTool,
    pub source: ForeignSessionSource,
    pub native_id: String,
    pub title: String,
    pub cwd: PathBuf,
    pub updated_at: SystemTime,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentForeignSession {
    pub tool: ForeignSessionTool,
    pub native_id: String,
    pub age: Duration,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnabledForeignSessionSources {
    pub claude: bool,
    pub codex: bool,
    pub cursor: bool,
}

pub fn scan_foreign_sessions(
    _cwd: &Path,
    _enabled: EnabledForeignSessionSources,
) -> Vec<ForeignSessionSummary> {
    Vec::new()
}

pub fn most_recent_foreign_session(
    _cwd: &Path,
    _enabled: EnabledForeignSessionSources,
    _within: Duration,
) -> Option<RecentForeignSession> {
    None
}
