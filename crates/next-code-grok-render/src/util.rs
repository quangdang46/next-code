//! `util` module — Grok-compatible utility functions.

/// Truncate a string to a maximum width.
pub fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else {
        format!("{}…", &s[..max_width.saturating_sub(1)])
    }
}

/// Format a duration in milliseconds for display.
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        let secs = ms / 1000;
        if secs < 60 {
            format!("{secs}s")
        } else {
            let mins = secs / 60;
            let rem = secs % 60;
            format!("{mins}m {rem}s")
        }
    }
}
