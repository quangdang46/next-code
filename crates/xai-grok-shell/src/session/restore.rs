//! Stub of upstream `xai-grok-shell::session::restore`. Upstream ships a
//! real remote-restore implementation gated behind internal build flags,
//! plus a `restore_stub.rs` OSS twin that reports
//! "Remote session restore is not available in this build" — this façade
//! mirrors that OSS-stub pattern rather than the real implementation.

use std::time::Duration;

const UNAVAILABLE: &str = "Remote session restore is not available in this build";

pub type ProgressCallback = Box<dyn Fn(&RestoreProgressEvent) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStep {
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestorePhase {
    Codebase,
    Memory,
    SessionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreProgressEvent {
    pub phase: RestorePhase,
    pub step: PhaseStep,
    pub incomplete: bool,
    pub message: String,
    pub detail: Option<String>,
    pub elapsed: Duration,
}

pub async fn restore_session_with_storage(
    _session_id: &str,
    _on_progress: Option<ProgressCallback>,
) -> Result<(), String> {
    Err(UNAVAILABLE.to_string())
}
