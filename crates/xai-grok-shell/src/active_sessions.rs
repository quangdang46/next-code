//! Façade stub of upstream `xai-grok-shell::active_sessions`. Signatures
//! match upstream (`register`/`try_unregister`/`list_in`/…), but this
//! compile-stub layer has no on-disk `active_sessions.json` — everything
//! is a no-op / empty-list placeholder.

use std::io;
use std::path::Path;

use agent_client_protocol as acp;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ActiveSession {
    pub session_id: acp::SessionId,
    pub pid: u32,
    pub cwd: String,
    pub opened_at: DateTime<Utc>,
}

pub fn register(_session: ActiveSession) -> io::Result<()> {
    Ok(())
}

pub fn unregister(_session_id: &acp::SessionId) -> io::Result<()> {
    Ok(())
}

pub fn try_unregister(_session_id: &acp::SessionId) -> io::Result<bool> {
    Ok(false)
}

pub fn collect_crashed() -> io::Result<Vec<ActiveSession>> {
    Ok(Vec::new())
}

pub fn register_in(_root: &Path, _session: ActiveSession) -> io::Result<()> {
    Ok(())
}

pub fn unregister_in(_root: &Path, _session_id: &acp::SessionId) -> io::Result<()> {
    Ok(())
}

pub fn try_unregister_in(_root: &Path, _session_id: &acp::SessionId) -> io::Result<bool> {
    Ok(false)
}

pub fn collect_crashed_in(_root: &Path) -> io::Result<Vec<ActiveSession>> {
    Ok(Vec::new())
}

pub fn list_in(_root: &Path) -> io::Result<Vec<ActiveSession>> {
    Ok(Vec::new())
}
