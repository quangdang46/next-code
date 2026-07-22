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

/// Whether process-level Grok **Chat** mode features are enabled.
///
/// Stock grok-build: true only under `--chat` / `GROK_CHAT_MODE`.
/// next-code Face is Build-equivalent (coding agent), so this is always
/// `false` — otherwise `SetDefaultModel` skips persisting
/// `[provider].default_model` (gated on `!process_chat_mode_enabled()`).
pub fn process_chat_mode_enabled() -> bool {
    false
}
