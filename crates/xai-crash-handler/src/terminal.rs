//! Terminal restore sequences for signal handler context.
//!
//! Constants match upstream `xai-crash-handler::terminal` exactly so the
//! pager's wrap-restore ordering tests stay valid.

/// Raw CSI sequences to disable every mouse-tracking mode the pager enables
/// (`?1000/?1002/?1003/?1015/?1006`) — the mouse subset of [`MOUSE_PASTE_RESET`],
/// without the bracketed-paste (`?2004l`) reset.
pub const MOUSE_TRACKING_RESET: &[u8] = b"\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1015l\x1b[?1006l";

/// Raw CSI sequences to disable mouse tracking and bracketed paste.
pub const MOUSE_PASTE_RESET: &[u8] =
    b"\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1015l\x1b[?1006l\x1b[?2004l";

/// Full escape sequence to restore the terminal to a sane state.
///
/// The kitty CSI-u pop precedes `?1049l` per spec (the protocol stack
/// is per-screen).
pub const RESTORE_SEQ: &[u8] =
    b"\x1b[?2026l\x1b[?25h\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1015l\x1b[?1006l\x1b[?2004l\x1b[?1004l\x1b[<u\x1b[?1049l";
