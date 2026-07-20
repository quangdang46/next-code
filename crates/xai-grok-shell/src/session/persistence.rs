//! Façade stub of upstream `xai-grok-shell::session::persistence` — local
//! session-id resolution helpers the future pager uses for resume/restore.
//! No real on-disk session store in this compile-stub layer, so lookups
//! always report "not found" rather than walking `~/.next-code/sessions`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalFeedbackEntry {
    pub session_id: String,
    pub rating: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserFeedbackEntry {
    pub session_id: String,
    pub comment: String,
}

pub fn session_exists_by_id(_session_id: &str) -> bool {
    false
}

pub fn session_exists_for_cwd(_session_id: &str, _cwd: &str) -> bool {
    false
}

pub fn find_local_child_for_remote(_session_id: &str, _cwd: &str) -> Option<String> {
    None
}

pub fn resolve_local_session(_session_id: &str, _cwd: &str) -> Option<String> {
    None
}

pub fn resolve_local_session_any_cwd(_session_id: &str) -> Option<String> {
    None
}
