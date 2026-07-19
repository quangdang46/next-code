//! Terminal context stubs for grok input module compatibility.
//! Types match grok-build's terminal module.

/// Terminal brand identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalName {
    Unknown,
    AppleTerminal,
    Iterm2,
    Ghostty,
    Kitty,
    WezTerm,
    Alacritty,
    Rio,
    WarpTerminal,
    VsCode,
    WindowsTerminal,
    Foot,
    Cursor,
    Windsurf,
    Zed,
    GrokDesktop,
    Vte,
    Terminator,
    JetBrains,
    Otty,
}

/// How modifiers are delivered by the terminal/OS combo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierDelivery {
    Standard,
    Enhanced,
}

impl Default for ModifierDelivery {
    fn default() -> Self {
        Self::Standard
    }
}

/// ModifierFate.
#[derive(Debug, Clone, Copy)]
pub enum ModifierFate {
    Passthrough,
    Consumed,
}

/// Keyboard capabilities.
#[derive(Debug, Clone, Copy)]
pub enum KeyboardCapabilities {
    Basic,
    Enhanced,
}

/// Multiplexer kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiplexerKind {
    Tmux,
    Screen,
    Zellij,
    Cmux,
    Undetected,
}

impl Default for MultiplexerKind {
    fn default() -> Self {
        Self::Undetected
    }
}

/// Terminal context snapshot.
pub struct TerminalContext {
    pub brand: TerminalName,
    pub delivery: ModifierDelivery,
    pub keyboard_caps: KeyboardCapabilities,
    pub multiplexer: MultiplexerKind,
}

/// Get the current terminal context.
pub fn terminal_context() -> TerminalContext {
    TerminalContext {
        brand: TerminalName::Unknown,
        delivery: ModifierDelivery::Standard,
        keyboard_caps: KeyboardCapabilities::Basic,
        multiplexer: MultiplexerKind::Undetected,
    }
}
