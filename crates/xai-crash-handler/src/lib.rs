//! Facade of `xai-org/grok-build` `xai-crash-handler` (Apache-2.0) for the
//! next-code Grok Face migration (PR7).
//!
//! Upstream installs real SIGSEGV/SIGBUS handlers with crash dumps. This stub
//! only reproduces the surfaces the pager imports: terminal-escape restore
//! toggles (no-ops) and the CSI restore constants in [`terminal`].

pub mod terminal;

/// Upgrade SIGSEGV/SIGBUS handlers to include terminal escape code
/// restoration. Call when TUI modes are enabled.
///
/// Stub: no-op (no crash handler is installed).
pub fn enable_terminal_escape_restore() {}

/// Downgrade SIGSEGV/SIGBUS handlers to termios-only restoration.
/// Call when TUI modes are disabled.
///
/// Stub: no-op.
pub fn disable_terminal_escape_restore() {}
