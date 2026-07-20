//! Smooth scrollbar widget with follow-mode awareness.
//!
//! Adapted for crates.io ratatui 0.28 (no `tui-scrollbar` / `ratatui-core`).
//! Thumb position uses proportional math equivalent to JumpToClick.

use std::sync::atomic::{AtomicBool, Ordering};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

/// When set, every scrollbar renders as a no-op. The pager toggles this on in
/// minimal (scrollback-native) mode, where lists/dropdowns show
/// no scrollbar bar at all — they scroll internally and the footer carries the
/// "↑/↓ navigate" hint. Off (default) everywhere else, so the full TUI is
/// unaffected.
static SCROLLBARS_HIDDEN: AtomicBool = AtomicBool::new(false);

/// Globally hide or show all scrollbars. See [`SCROLLBARS_HIDDEN`].
pub fn set_scrollbars_hidden(hidden: bool) {
    SCROLLBARS_HIDDEN.store(hidden, Ordering::Relaxed);
}

/// Whether scrollbars are currently globally hidden.
pub fn scrollbars_hidden() -> bool {
    SCROLLBARS_HIDDEN.load(Ordering::Relaxed)
}

/// Number of columns reserved between content and the scrollbar track.
/// This creates the "X" gap in the XSXBXX pattern (gap between selection_right and scrollbar).
const SCROLLBAR_GAP_COLS: u16 = 1;

/// Width of the scrollbar track itself (in terminal cells).
const SCROLLBAR_TRACK_COLS: u16 = 1;

/// Total columns reserved for scrollbar UI (gap + track).
pub const SCROLLBAR_TOTAL_COLS: u16 = SCROLLBAR_GAP_COLS + SCROLLBAR_TRACK_COLS;

/// Split an area into content + scrollbar regions.
///
/// Layout:
/// - `content_area`: original area minus [`SCROLLBAR_TOTAL_COLS`] on the right
/// - `scrollbar_area`: the last column of the original area (1 cell wide)
/// - The column between them is the "gap" (left intentionally blank)
///
/// Returns `(content_area, None)` when the terminal is too narrow.
///
/// **Note**: This always reserves space for scrollbar. Use [`maybe_split_for_scrollbar`]
/// to only reserve space when the scrollbar will actually be shown.
pub fn split_area_for_scrollbar(area: Rect) -> (Rect, Option<Rect>) {
    if area.width <= SCROLLBAR_TOTAL_COLS {
        return (area, None);
    }

    let content_width = area.width.saturating_sub(SCROLLBAR_TOTAL_COLS);
    let content_area = Rect {
        x: area.x,
        y: area.y,
        width: content_width,
        height: area.height,
    };
    let scrollbar_area = Rect {
        x: area.right().saturating_sub(1),
        y: area.y,
        width: SCROLLBAR_TRACK_COLS,
        height: area.height,
    };

    (content_area, Some(scrollbar_area))
}

/// Split an area only if scrollbar is actually needed.
///
/// Unlike [`split_area_for_scrollbar`], this gives full width to content
/// when scrollbar won't be shown (`total_lines <= viewport_lines`).
///
/// Use this when you know the content height before splitting.
pub fn maybe_split_for_scrollbar(area: Rect, total_lines: u16) -> (Rect, Option<Rect>) {
    // Only reserve space if scrollbar will actually be shown
    if needs_scrollbar(total_lines, area.height) {
        split_area_for_scrollbar(area)
    } else {
        // No scrollbar needed - give full width to content
        (area, None)
    }
}

/// Whether the scrollbar should be shown (content overflows viewport).
pub fn needs_scrollbar(total_lines: u16, viewport_lines: u16) -> bool {
    total_lines > viewport_lines
}

/// Whether the view is at the bottom (following mode position).
#[allow(dead_code)] // Useful helper, kept for future use
pub fn is_at_bottom(total_lines: u16, viewport_lines: u16, offset: u16) -> bool {
    let max_offset = total_lines.saturating_sub(viewport_lines);
    offset >= max_offset
}

/// Result of mapping a scrollbar click/drag position to a scroll offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarClickResult {
    /// Jump to the very top (click on first row of track).
    Top,
    /// Jump to the very bottom (click on last row of track).
    Bottom,
    /// Set scroll offset to this value (proportional position).
    Offset(usize),
}

/// Map a click on the scrollbar gutter to a scroll offset.
pub fn scrollbar_click_to_offset(
    cell_index: u16,
    track_cells: u16,
    total_lines: u16,
    viewport_lines: u16,
) -> ScrollbarClickResult {
    if track_cells == 0 {
        return ScrollbarClickResult::Top;
    }

    if cell_index == 0 {
        return ScrollbarClickResult::Top;
    }
    if cell_index >= track_cells.saturating_sub(1) {
        return ScrollbarClickResult::Bottom;
    }

    let max_offset = total_lines.saturating_sub(viewport_lines) as usize;
    if max_offset == 0 || track_cells <= 1 {
        return ScrollbarClickResult::Top;
    }
    let ratio = cell_index as f64 / (track_cells.saturating_sub(1) as f64);
    let offset = (ratio * max_offset as f64).round() as usize;
    ScrollbarClickResult::Offset(offset.min(max_offset))
}

fn paint_scrollbar_track(
    buf: &mut Buffer,
    scrollbar_area: Rect,
    total_lines: u16,
    viewport_lines: u16,
    offset: u16,
    track_style: Style,
    thumb_style: Style,
) {
    let track_h = scrollbar_area.height as usize;
    let content = total_lines.max(1) as usize;
    let viewport = viewport_lines.max(1) as usize;
    let thumb_len = ((viewport * track_h) / content).clamp(1, track_h);
    let max_offset = content.saturating_sub(viewport);
    let thumb_start = if max_offset == 0 {
        0
    } else {
        ((offset as usize) * (track_h - thumb_len)) / max_offset
    };
    let thumb_end = (thumb_start + thumb_len).min(track_h);

    for row in 0..scrollbar_area.height {
        let x = scrollbar_area.x;
        let y = scrollbar_area.y + row;
        let dst = &mut buf[(x, y)];
        let on_thumb = (row as usize) >= thumb_start && (row as usize) < thumb_end;
        if on_thumb {
            dst.set_symbol("\u{2588}");
            dst.set_style(thumb_style);
        } else {
            dst.set_symbol(" ");
            dst.set_style(track_style);
        }
    }
}

/// Render a scrollbar with follow-mode aware styling.
pub fn render_scrollbar(
    buf: &mut Buffer,
    scrollbar_area: Option<Rect>,
    total_lines: u16,
    viewport_lines: u16,
    offset: u16,
    is_following: bool,
) {
    if SCROLLBARS_HIDDEN.load(Ordering::Relaxed) {
        return;
    }

    let Some(scrollbar_area) = scrollbar_area else {
        return;
    };

    if scrollbar_area.width == 0 || scrollbar_area.height == 0 {
        return;
    }

    if !needs_scrollbar(total_lines, viewport_lines) {
        return;
    }

    let (track_style, thumb_style) = scrollbar_styles(is_following);
    paint_scrollbar_track(
        buf,
        scrollbar_area,
        total_lines,
        viewport_lines,
        offset,
        track_style,
        thumb_style,
    );
}

/// Get track and thumb styles based on follow mode.
///
/// Following mode: very dim colors (scrollbar recedes into background)
/// Not following: brighter colors (scrollbar "pops out")
fn scrollbar_styles(is_following: bool) -> (Style, Style) {
    let theme = crate::theme::Theme::current();
    if is_following {
        // Very dim - scrollbar is subtle when following
        let track_style = Style::new().bg(theme.scrollbar_bg);
        let thumb_style = Style::new().fg(theme.scrollbar_fg).bg(theme.scrollbar_bg);
        (track_style, thumb_style)
    } else {
        // Brighter - scrollbar stands out when scrolled up
        let track_style = Style::new().bg(theme.bg_highlight);
        let thumb_style = Style::new().fg(theme.gray).bg(theme.bg_highlight);
        (track_style, thumb_style)
    }
}

/// Render a scrollbar with custom track and thumb styles.
///
/// Like [`render_scrollbar`] but allows custom styling for theme integration.
pub fn render_scrollbar_styled(
    buf: &mut Buffer,
    scrollbar_area: Option<Rect>,
    total_lines: u16,
    viewport_lines: u16,
    offset: u16,
    track_style: Style,
    thumb_style: Style,
) {
    if SCROLLBARS_HIDDEN.load(Ordering::Relaxed) {
        return;
    }

    let Some(scrollbar_area) = scrollbar_area else {
        return;
    };

    if scrollbar_area.width == 0 || scrollbar_area.height == 0 {
        return;
    }

    if !needs_scrollbar(total_lines, viewport_lines) {
        return;
    }

    paint_scrollbar_track(
        buf,
        scrollbar_area,
        total_lines,
        viewport_lines,
        offset,
        track_style,
        thumb_style,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_split_area_normal() {
        let area = Rect::new(0, 0, 40, 10);
        let (content, scrollbar) = split_area_for_scrollbar(area);

        // Content should be 40 - 2 = 38 wide (gap + track)
        assert_eq!(content.width, 38);
        assert_eq!(content.height, 10);

        // Scrollbar should be at x=39, 1 column wide
        let sb = scrollbar.expect("scrollbar area");
        assert_eq!(sb.x, 39);
        assert_eq!(sb.width, 1);
        assert_eq!(sb.height, 10);
    }

    #[test]
    fn test_split_area_too_narrow() {
        let area = Rect::new(0, 0, 2, 10);
        let (content, scrollbar) = split_area_for_scrollbar(area);

        // Too narrow - return original area, no scrollbar
        assert_eq!(content, area);
        assert!(scrollbar.is_none());
    }

    #[test]
    fn test_maybe_split_reserves_when_needed() {
        let area = Rect::new(0, 0, 40, 10);

        // Content overflows (20 > 10) - should reserve scrollbar space
        let (content, scrollbar) = maybe_split_for_scrollbar(area, 20);
        assert_eq!(content.width, 38); // Reduced by 2 for gap + scrollbar track
        assert!(scrollbar.is_some());
    }

    #[test]
    fn test_maybe_split_full_width_when_not_needed() {
        let area = Rect::new(0, 0, 40, 10);

        // Content fits (5 <= 10) - should give full width to content
        let (content, scrollbar) = maybe_split_for_scrollbar(area, 5);
        assert_eq!(content.width, 40); // Full width
        assert!(scrollbar.is_none());
    }

    #[test]
    fn test_needs_scrollbar() {
        assert!(needs_scrollbar(100, 10)); // Content > viewport
        assert!(!needs_scrollbar(10, 10)); // Content == viewport
        assert!(!needs_scrollbar(5, 10)); // Content < viewport
    }

    #[test]
    fn test_is_at_bottom() {
        // total=100, viewport=10 -> max_offset=90
        assert!(is_at_bottom(100, 10, 90)); // At bottom
        assert!(is_at_bottom(100, 10, 95)); // Past bottom (clamped)
        assert!(!is_at_bottom(100, 10, 89)); // One line above bottom
        assert!(!is_at_bottom(100, 10, 0)); // At top
    }

    #[test]
    fn test_render_scrollbar_no_area() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 10));
        // Should not panic with None area
        render_scrollbar(&mut buf, None, 100, 10, 0, false);
    }

    #[test]
    fn test_render_scrollbar_no_overflow() {
        let area = Rect::new(0, 0, 10, 10);
        let (_, scrollbar_area) = split_area_for_scrollbar(area);
        let mut buf = Buffer::empty(area);

        // Content fits - should not render anything
        render_scrollbar(&mut buf, scrollbar_area, 5, 10, 0, false);

        // Check scrollbar column is empty (spaces with no custom background)
        let sb = scrollbar_area.unwrap();
        for y in 0..sb.height {
            let cell = &buf[(sb.x, sb.y + y)];
            assert_eq!(cell.symbol(), " ");
            // The cell should NOT have our scrollbar background colors
            // (i.e., it should be reset/default, not Color::Rgb)
            if let Some(Color::Rgb(_, _, _)) = cell.style().bg {
                panic!("Should not have RGB background when no scrollbar rendered");
            }
            // Otherwise - Reset, None, or other default-like value
        }
    }

    #[test]
    fn test_render_scrollbar_following_vs_not() {
        let area = Rect::new(0, 0, 10, 10);
        let (_, scrollbar_area) = split_area_for_scrollbar(area);

        // Render following
        let mut buf_following = Buffer::empty(area);
        render_scrollbar(&mut buf_following, scrollbar_area, 100, 10, 90, true);

        // Render not following
        let mut buf_not_following = Buffer::empty(area);
        render_scrollbar(&mut buf_not_following, scrollbar_area, 100, 10, 50, false);

        // The styles should differ - not following should be brighter
        let sb = scrollbar_area.unwrap();
        let following_style = buf_following[(sb.x, sb.y)].style();
        let not_following_style = buf_not_following[(sb.x, sb.y)].style();

        // Both should have backgrounds set (non-default)
        assert!(following_style.bg.is_some());
        assert!(not_following_style.bg.is_some());

        // At 256-color or truecolor, the backgrounds should be distinguishable.
        // At Basic (16-color) level, both dark grays map to Black — expected.
        if crate::theme::color_support::get().has_256() {
            assert_ne!(following_style.bg, not_following_style.bg);
        }
    }

    #[test]
    fn test_scrollbar_thumb_position() {
        let area = Rect::new(0, 0, 10, 10);
        let (_, scrollbar_area) = split_area_for_scrollbar(area);
        let sb = scrollbar_area.unwrap();

        // At top
        let mut buf_top = Buffer::empty(area);
        render_scrollbar(&mut buf_top, scrollbar_area, 100, 10, 0, false);

        // At bottom
        let mut buf_bottom = Buffer::empty(area);
        render_scrollbar(&mut buf_bottom, scrollbar_area, 100, 10, 90, false);

        // Count thumb cells (non-space)
        let count_thumb = |buf: &Buffer| -> usize {
            (0..sb.height)
                .filter(|&y| buf[(sb.x, sb.y + y)].symbol() != " ")
                .count()
        };

        // Both should have a thumb
        let top_thumb = count_thumb(&buf_top);
        let bottom_thumb = count_thumb(&buf_bottom);
        assert!(top_thumb > 0, "Should have thumb at top");
        assert!(bottom_thumb > 0, "Should have thumb at bottom");

        // Thumb size should be consistent
        assert_eq!(top_thumb, bottom_thumb, "Thumb size should be consistent");

        // Thumb position should differ (visual inspection would show top vs bottom)
        // We can check that the thumb cells are in different positions
        let thumb_positions = |buf: &Buffer| -> Vec<u16> {
            (0..sb.height)
                .filter(|&y| buf[(sb.x, sb.y + y)].symbol() != " ")
                .collect()
        };

        let top_pos = thumb_positions(&buf_top);
        let bottom_pos = thumb_positions(&buf_bottom);
        assert_ne!(
            top_pos, bottom_pos,
            "Thumb should be at different positions"
        );
    }
}
