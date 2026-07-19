//! `clipboard` module — Grok-compatible clipboard types.
//!
//! Wraps next-code's clipboard/system types for Grok compatibility.

/// Placeholder image data type for clipboard operations.
#[derive(Debug, Clone)]
pub struct ImageData {
    pub data: Vec<u8>,
    pub mime_type: String,
}

/// Check if OSC 52 sink is active (stub).
pub fn osc52_sink_active() -> bool {
    false
}

/// Get clipboard image (stub).
pub fn system_clipboard_get_image() -> Option<ImageData> {
    None
}
