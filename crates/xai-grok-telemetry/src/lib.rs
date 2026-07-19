pub mod client { pub fn is_enabled() -> bool { false } }
pub mod session_ctx { pub fn log_event(_: crate::events::ClipboardCopy) {} }
pub mod events {
    pub struct ClipboardCopy;
    pub struct PasteKeyEmptyHostClipboard;
    pub struct ClipboardImagePaste;
    pub struct TerminalTelemetry;
}
