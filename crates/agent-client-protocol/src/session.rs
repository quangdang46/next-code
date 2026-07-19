/// Session management for the agent-client protocol.

use std::path::PathBuf;

/// Identifier for an agent session.
pub type SessionId = String;

/// Configuration for an agent session.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// The session identifier.
    pub session_id: String,
    /// Optional model override.
    pub model: Option<String>,
    /// System prompt for the session.
    pub system_prompt: Option<String>,
    /// Working directory.
    pub cwd: Option<PathBuf>,
}

/// Handle to an active agent session.
#[derive(Debug, Clone)]
pub struct SessionHandle {
    /// The session configuration.
    pub config: SessionConfig,
    /// Whether the session is active.
    pub active: bool,
}
