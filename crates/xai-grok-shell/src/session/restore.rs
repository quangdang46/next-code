//! Stub of upstream `xai-grok-shell::session::restore`. Upstream ships a
//! real remote-restore implementation gated behind internal build flags;
//! this façade mirrors the OSS-stub pattern.

use std::path::Path;
use std::time::Duration;

use crate::agent::session_registry_client::SessionRegistryClient;
use crate::auth::credential_provider::StorageClient;

const UNAVAILABLE: &str = "Remote session restore is not available in this build";

pub type ProgressCallback = Box<dyn Fn(&RestoreProgressEvent) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStep {
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestorePhase {
    Download,
    Codebase,
    Memory,
    SessionState,
    Finalize,
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

impl RestoreProgressEvent {
    pub fn display_line(&self) -> String {
        let phase = match self.phase {
            RestorePhase::Download => "download",
            RestorePhase::Codebase => "codebase",
            RestorePhase::Memory => "memory",
            RestorePhase::SessionState => "session",
            RestorePhase::Finalize => "finalize",
        };
        let step = match self.step {
            PhaseStep::Start => "start",
            PhaseStep::End => "end",
        };
        if let Some(detail) = &self.detail {
            format!("[{phase}/{step}] {} ({detail})", self.message)
        } else {
            format!("[{phase}/{step}] {}", self.message)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RestoreResult {
    pub local_session_id: String,
}

pub async fn restore_session_with_storage(
    _registry: &SessionRegistryClient,
    _storage: &StorageClient,
    _session_id: &str,
    _cwd: &str,
    _preferred_local_id: Option<&str>,
    _on_progress: Option<ProgressCallback>,
) -> anyhow::Result<RestoreResult> {
    let _ = Path::new(_cwd);
    Err(anyhow::anyhow!(UNAVAILABLE))
}
