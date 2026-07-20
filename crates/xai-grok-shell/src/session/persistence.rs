//! Façade stub of upstream `xai-grok-shell::session::persistence` — local
//! session-id resolution helpers the future pager uses for resume/restore.
//! No real on-disk session store in this compile-stub layer.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agent_client_protocol::SessionId;
use serde::{Deserialize, Serialize};

use crate::auth::AuthManager;
use crate::session::info::Info as SessionPathInfo;
use crate::util::grok_home::grok_home;

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

/// Per-session summary row (`summary.json` shape).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub id: SessionId,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub title: Option<String>,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            id: SessionId::new(String::new()),
            cwd: String::new(),
            title: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub info: Info,
    #[serde(default)]
    pub session_summary: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub num_messages: usize,
    #[serde(default)]
    pub num_chat_messages: usize,
    #[serde(default)]
    pub current_model_id: String,
    #[serde(default)]
    pub parent_session_id: Option<String>,
    #[serde(default)]
    pub manual_title: Option<String>,
}

impl Default for Summary {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            info: Info::default(),
            session_summary: String::new(),
            created_at: now,
            updated_at: now,
            num_messages: 0,
            num_chat_messages: 0,
            current_model_id: String::new(),
            parent_session_id: None,
            manual_title: None,
        }
    }
}

impl Summary {
    pub fn is_hidden(&self) -> bool {
        false
    }

    pub fn manual_title_opt(&self) -> Option<String> {
        self.manual_title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .or_else(|| {
                self.info
                    .title
                    .clone()
                    .filter(|t| !t.trim().is_empty())
            })
    }

    pub fn display_title_opt(&self) -> Option<String> {
        self.manual_title_opt().or_else(|| {
            let s = self.session_summary.trim();
            if s.is_empty() {
                None
            } else {
                Some(s.lines().next().unwrap_or(s).trim().to_string())
            }
        })
    }
}

/// List local session summaries, optionally filtered to a cwd.
pub async fn list_summaries(_cwd: Option<&str>) -> anyhow::Result<Vec<Summary>> {
    Ok(vec![])
}

pub async fn list_recent_summaries(_limit: usize) -> std::io::Result<Vec<Summary>> {
    Ok(vec![])
}

/// Resolve the on-disk directory for a session path key.
pub fn session_dir(info: &SessionPathInfo) -> PathBuf {
    grok_home()
        .join("sessions")
        .join(info.cwd.replace(['/', '\\', ':'], "-"))
        .join(info.id.0.as_ref())
}

/// Find a session directory by id across all cwd buckets (stub: synthetic path).
pub fn find_session_dir_by_id(session_id: &str) -> anyhow::Result<PathBuf> {
    Ok(grok_home().join("sessions").join("_").join(session_id))
}

/// Best-effort sandbox profile recorded with a resumed session.
pub fn resumed_session_sandbox_profile(
    _session_id: Option<&str>,
    _cwd: Option<&str>,
) -> Option<String> {
    None
}

#[derive(Debug, Clone, Default)]
pub struct SessionHistoryDeletion {
    pub local_removed: bool,
    pub remote_removed: bool,
}

impl SessionHistoryDeletion {
    pub fn any_removed(&self) -> bool {
        self.local_removed || self.remote_removed
    }
}

/// Delete local (+ optional remote) session history. Stub always reports nothing removed.
pub async fn delete_session_history(
    _session_id: &str,
    _cwd: Option<&str>,
    _needs_remote: bool,
    _auth: Arc<AuthManager>,
) -> anyhow::Result<SessionHistoryDeletion> {
    Ok(SessionHistoryDeletion::default())
}

/// Convenience: build a path-info key without pulling ACP into every call site.
pub fn path_info(session_id: impl Into<String>, cwd: impl Into<String>) -> SessionPathInfo {
    SessionPathInfo {
        id: SessionId::new(session_id.into()),
        cwd: cwd.into(),
    }
}

/// Unused helper kept for API parity with upstream path encoding.
pub fn sessions_root() -> PathBuf {
    grok_home().join("sessions")
}

/// Accept either `&str` or `&Path` filter args at call sites that pass Path.
pub async fn list_summaries_path(cwd: Option<&Path>) -> anyhow::Result<Vec<Summary>> {
    let owned = cwd.map(|p| p.to_string_lossy().into_owned());
    list_summaries(owned.as_deref()).await
}
