use std::path::PathBuf;

use agent_client_protocol as acp;

pub mod info;

pub use info::Info;

/// Terminal fingerprint for feedback (local struct; not prod-mc).
#[derive(Debug, Clone)]
pub struct FeedbackTerminalInfo {
    pub brand: String,
    pub multiplexer: String,
    pub is_ssh: bool,
    pub is_byobu: bool,
    pub term_var: String,
    pub tmux_version: Option<String>,
    pub hyperlink_osc8_support: Option<String>,
    pub clipboard_route: Option<String>,
    pub clipboard_native_tool: Option<String>,
    pub display_server: Option<String>,
}

pub fn session_dir(info: &Info) -> PathBuf {
    xai_grok_tools::util::grok_home::sessions_cwd_dir(&info.cwd).join(info.id.to_string())
}

// Re-export SessionId convenience for callers that used acp via session.
pub type SessionId = acp::SessionId;
