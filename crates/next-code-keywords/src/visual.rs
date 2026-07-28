//! Visual effects — keyword highlight spans for TUI rendering.

use crate::detector::detect_keywords_with;
use crate::options::DetectOptions;

/// A highlight span for a detected keyword in the input.
#[derive(Debug, Clone)]
pub struct KeywordHighlight {
    /// Byte offset start in the input string.
    pub start: usize,
    /// Byte offset end in the input string.
    pub end: usize,
    /// RGB color for rainbow effect.
    pub color: (u8, u8, u8),
    /// The keyword label (e.g. "$ultrawork").
    pub label: String,
    /// Priority of the matched keyword.
    pub priority: u8,
}

/// Compute highlight spans using Strict defaults.
pub fn compute_highlights(input: &str) -> Vec<KeywordHighlight> {
    compute_highlights_with(input, &DetectOptions::default())
}

/// Compute highlight spans with the same detect options as activation.
pub fn compute_highlights_with(input: &str, opts: &DetectOptions) -> Vec<KeywordHighlight> {
    let detections = detect_keywords_with(input, opts);
    let mut results: Vec<KeywordHighlight> = Vec::with_capacity(detections.len());
    let mut cursor = 0usize;
    for (i, det) in detections.into_iter().enumerate() {
        let (start, end) = remap_to_raw_span(input, &det.matched_text, det.position);

        // Skip highlights that start before the current cursor (overlap with
        // a previous, higher-priority match).
        if start < cursor {
            continue;
        }

        let color = rainbow_color(i, det.entry.priority);
        results.push(KeywordHighlight {
            start,
            end,
            color,
            label: det.matched_text.clone(),
            priority: det.entry.priority,
        });
        cursor = end;
    }
    results
}

/// Map a sanitized detection span back onto the raw input when whitespace /
/// ANSI sanitization shifted offsets. Falls back to the sanitized span.
fn remap_to_raw_span(raw: &str, matched: &str, sanitized_pos: (usize, usize)) -> (usize, usize) {
    let sanitized = crate::sanitizer::sanitize(raw);
    if sanitized.as_str() == raw {
        let start = sanitized_pos.0.min(raw.len());
        let end = sanitized_pos.1.min(raw.len());
        return (start, end);
    }
    let lower = crate::sanitizer::to_lower(raw);
    let needle = matched.to_lowercase();
    if let Some(pos) = crate::detector::find_word_boundary(&lower, &needle) {
        // Prefer the raw slice length so multi-byte / case fold stays aligned.
        let end = pos + matched.len();
        if end <= raw.len() {
            return (pos, end);
        }
    }
    (
        sanitized_pos.0.min(raw.len()),
        sanitized_pos.1.min(raw.len()),
    )
}

/// Generate a rainbow RGB color based on index and priority.
///
/// Higher priority → warmer colors (red/orange).
/// Lower priority → cooler colors (blue/purple).
fn rainbow_color(index: usize, priority: u8) -> (u8, u8, u8) {
    // Base hue from priority: 0 (red) to 270 (purple)
    let base_hue = ((11 - priority) as f32 / 11.0) * 270.0;
    // Offset by index for variety
    let hue = (base_hue + (index as f32 * 30.0)) % 360.0;
    hsv_to_rgb(hue, 0.8, 0.95)
}

/// Convert HSV to RGB.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Format a highlight as a display string for status notices.
pub fn format_highlight_notice(highlights: &[KeywordHighlight]) -> Option<String> {
    if highlights.is_empty() {
        return None;
    }

    let labels: Vec<&str> = highlights.iter().map(|h| h.label.as_str()).collect();
    Some(format!("✨ Keywords: {}", labels.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_highlights_empty_input() {
        assert!(compute_highlights("").is_empty());
    }

    #[test]
    fn compute_highlights_detects_keyword() {
        let highlights = compute_highlights("$ultrawork fix the bug");
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].label, "$ultrawork");
    }

    #[test]
    fn compute_highlights_bare_ultrawork() {
        let highlights = compute_highlights("ultrawork fix the bug");
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].label, "ultrawork");
        assert_eq!(highlights[0].start, 0);
        assert_eq!(highlights[0].end, "ultrawork".len());
    }

    #[test]
    fn compute_highlights_bare_set_smoke() {
        for token in [
            "ultrawork",
            "ultrathink",
            "ultragoal",
            "ultraqa",
            "ralplan",
            "hyperplan",
            "ultraplan",
            "tdd",
            "deepsearch",
            "analyze",
            "code-review",
            "ultrareview",
            "security-review",
            "bestofn",
            "teammode",
            "team-mode",
        ] {
            let highlights = compute_highlights(token);
            assert_eq!(
                highlights.len(),
                1,
                "expected highlight for bare token {token}"
            );
            assert_eq!(highlights[0].label.to_lowercase(), token.to_lowercase());
        }
    }

    #[test]
    fn rainbow_color_varies() {
        let c1 = rainbow_color(0, 10);
        let c2 = rainbow_color(1, 10);
        assert_ne!(c1, c2);
    }

    #[test]
    fn hsv_to_rgb_pure_red() {
        let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn format_highlight_notice_empty() {
        assert!(format_highlight_notice(&[]).is_none());
    }

    #[test]
    fn format_highlight_notice_with_keywords() {
        let highlights = vec![KeywordHighlight {
            start: 0,
            end: 10,
            color: (255, 0, 0),
            label: "$ultrawork".to_string(),
            priority: 10,
        }];
        let notice = format_highlight_notice(&highlights);
        assert!(notice.unwrap().contains("$ultrawork"));
    }
}
