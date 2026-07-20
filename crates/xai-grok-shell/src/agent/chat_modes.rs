//! Stub of upstream `xai-grok-shell::agent::chat_modes`.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ChatMode {
    #[default]
    Default,
    Plan,
    Ask,
}

impl ChatMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Plan => "plan",
            Self::Ask => "ask",
        }
    }
}

/// Whether process-level chat mode features are enabled (Face stub: always on).
pub fn process_chat_mode_enabled() -> bool {
    true
}
