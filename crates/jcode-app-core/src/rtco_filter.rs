/// Result of filtering a tool output through RTCO.
pub struct RtcoFilterResult {
    /// The filtered (compressed) text
    pub text: String,
    /// Original length in characters
    pub original_chars: usize,
    /// Filtered length in characters
    pub filtered_chars: usize,
    /// Token savings percentage (0.0 - 100.0)
    pub savings_percent: f64,
    /// Optional marker text describing what was compressed
    pub marker_text: Option<String>,
}

/// Filter a tool's output through RTCO.
///
/// Returns `Some(RtcoFilterResult)` if RTCO handled this tool and achieved
/// meaningful savings. Returns `None` if the tool isn't supported, the output
/// is too small, or savings are below the threshold.
///
/// When the `rtco` feature is not enabled, this function always returns `None`.
#[cfg(feature = "rtco")]
pub fn filter_tool_output(
    tool_name: &str,
    raw_output: &str,
    min_savings: f64,
) -> Option<RtcoFilterResult> {
    use rtco_core::{filter_output, has_filter};

    if !has_filter(tool_name) {
        return None;
    }

    if raw_output.len() < 512 {
        return None;
    }

    let result = filter_output(tool_name, raw_output);

    if result.savings_percent < min_savings {
        return None;
    }

    let marker_text = if result.markers.is_empty() {
        None
    } else {
        Some(format_markers(&result.markers))
    };

    Some(RtcoFilterResult {
        text: result.text,
        original_chars: result.original_tokens * 4,
        filtered_chars: result.filtered_tokens * 4,
        savings_percent: result.savings_percent,
        marker_text,
    })
}

/// No-op fallback when the `rtco` feature is disabled.
#[cfg(not(feature = "rtco"))]
pub fn filter_tool_output(
    _tool_name: &str,
    _raw_output: &str,
    _min_savings: f64,
) -> Option<RtcoFilterResult> {
    None
}

/// Get RTCO stats as a formatted string for display/telemetry.
#[cfg(feature = "rtco")]
pub fn format_rtco_summary(stats: &[RtcoFilterResult]) -> String {
    let total_original: usize = stats.iter().map(|s| s.original_chars).sum();
    let total_filtered: usize = stats.iter().map(|s| s.filtered_chars).sum();
    let avg_savings: f64 =
        stats.iter().map(|s| s.savings_percent).sum::<f64>() / stats.len().max(1) as f64;

    if total_original == 0 {
        return String::new();
    }

    let pct = 100.0 - (total_filtered as f64 / total_original as f64 * 100.0);
    format!(
        "rtco: filtered {} tool outputs, saved {:.0}% tokens ({:.1}k → {:.1}k chars, avg {:.0}%/tool)",
        stats.len(),
        pct,
        total_original as f64 / 1000.0,
        total_filtered as f64 / 1000.0,
        avg_savings,
    )
}

/// Default fallback when feature is disabled.
#[cfg(not(feature = "rtco"))]
pub fn format_rtco_summary(_stats: &[RtcoFilterResult]) -> String {
    String::new()
}

/// Format compression markers into a human-readable string.
#[cfg(feature = "rtco")]
fn format_markers(markers: &[rtco_core::CompressionMarker]) -> String {
    let parts: Vec<String> = markers
        .iter()
        .map(|m| {
            if m.count > 0 {
                format!("{} {}", m.count, m.details)
            } else {
                m.details.clone()
            }
        })
        .collect();

    if parts.is_empty() {
        return String::new();
    }

    format!("[rtco: {}]", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_noop_when_feature_disabled() {
        let result = filter_tool_output("git", "some long output", 0.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_summary_empty_for_no_data() {
        let result = format_rtco_summary(&[]);
        assert_eq!(result, "");
    }
}
