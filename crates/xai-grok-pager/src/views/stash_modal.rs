//! Multi-entry prompt stash modal (OpenCode DialogStash).
//!
//! Displays a list of stashed prompt entries showing the first line
//! (truncated to ~50 chars), a relative timestamp, and the line count.
//! Navigation with up/down (and j/k in vim mode). Enter restores the
//! selected stash into the prompt; `d` arms a delete confirmation
//! (second `d` confirms, any other key cancels).
//!
//! Rendered via `ModalWindow` chrome (same look as CommandPalette /
//! ShortcutsHelp), using the shared `PickerState` / `PickerEntry` /
//! `render_picker_in_modal` infrastructure.

use crate::views::picker::{PickerConfig, PickerOutcome, PickerState, handle_picker_input};
use unicode_width::UnicodeWidthChar;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single entry in the stash list.
#[derive(Debug, Clone)]
pub struct StashEntry {
    /// Display preview: first line, truncated to ~50 chars.
    pub preview: String,
    /// Full text stored in this entry.
    pub full_text: String,
    /// Number of lines in the stashed text.
    pub line_count: u16,
    /// Monotonic index (stable for delete).
    pub index: usize,
}

/// Summary data shown in the picker row (computed from a `StashEntry`).
#[derive(Debug, Clone)]
pub struct StashRowData {
    /// Preview text (first line, truncated).
    pub preview: String,
    /// Right-aligned label, e.g. "13m  5 lines". Empty when there are no entries.
    pub right_label: String,
    /// Original index into the source stash list.
    pub index: usize,
}

/// Persistent state for the stash browser modal.
#[derive(Debug, Clone)]
pub struct StashModalState {
    /// The list of stash entries to display.
    pub entries: Vec<StashRowData>,
    /// Unified picker state (selection, search query, scroll, hit areas).
    pub state: PickerState,
    /// Shared modal window chrome state.
    pub window: crate::views::modal_window::ModalWindowState,
    /// Index (into `entries`) armed for delete; `None` when not armed.
    /// Double-press `d` confirms; any other key cancels.
    pub pending_delete: Option<usize>,
}

impl StashModalState {
    /// Create a new stash modal from the raw stash entries.
    pub fn new(entries: Vec<StashEntry>) -> Self {
        let row_data: Vec<StashRowData> = entries
            .iter()
            .map(|e| {
                let preview = truncate_preview(&e.preview, 50);
                let right_label = format!("  {} lines", e.line_count);
                StashRowData {
                    preview,
                    right_label,
                    index: e.index,
                }
            })
            .collect();

        let state = PickerState::default();
        Self {
            entries: row_data,
            state,
            window: crate::views::modal_window::ModalWindowState::new(),
            pending_delete: None,
        }
    }
}

/// Truncate a string to at most `max_width` visible columns, appending "..." when cut.
fn truncate_preview(s: &str, max_width: usize) -> String {
    use unicode_width::UnicodeWidthStr;
    let first_line = s.lines().next().unwrap_or("");
    if first_line.width() <= max_width {
        first_line.to_string()
    } else {
        let mut w = 0usize;
        let mut cut = 0usize;
        for (byte_idx, ch) in first_line.char_indices() {
            let cw = ch.width().unwrap_or(0);
            if w + cw > max_width.saturating_sub(1) {
                break;
            }
            w += cw;
            cut = byte_idx + ch.len_utf8();
        }
        let mut result: String = first_line[..cut].into();
        result.push('\u{2026}');
        result
    }
}

// ---------------------------------------------------------------------------
// Footer shortcuts
// ---------------------------------------------------------------------------

/// Footer hints painted along the bottom border of the stash modal.
pub fn modal_footer(pending_delete: bool) -> Vec<crate::views::modal_window::Shortcut<'static>> {
    use crate::views::modal_window::Shortcut;
    if pending_delete {
        vec![
            Shortcut {
                label: "d confirm delete",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "other cancel",
                clickable: false,
                id: 0,
            },
        ]
    } else {
        let mut shortcuts = vec![
            Shortcut {
                label: "\u{2191}/\u{2193} nav",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "Enter restore",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "d delete",
                clickable: false,
                id: 0,
            },
            Shortcut {
                label: "Esc close",
                clickable: false,
                id: 0,
            },
        ];
        crate::views::modal_window::push_vim_nav_search_hint(&mut shortcuts, false);
        shortcuts
    }
}

// ---------------------------------------------------------------------------
// Sizing
// ---------------------------------------------------------------------------

pub fn modal_sizing(compact: bool) -> crate::views::modal_window::ModalSizing {
    crate::views::modal_window::ModalSizing {
        width_pct: 0.55,
        max_width: 60,
        min_width: 36,
        v_margin: 5,
        h_pad: 2,
        v_pad: 1,
        footer_lines: 2,
    }
    .with_compact(compact)
}

// ---------------------------------------------------------------------------
// Input dispatch
// ---------------------------------------------------------------------------

/// Outcome of processing an input event for the stash modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StashOutcome {
    /// User chose to restore the stash entry at this index (maps to the original stash index).
    Restore(usize),
    /// User confirmed deletion of the stash entry at this index.
    Delete(usize),
    /// User pressed Esc / clicked close — dismiss.
    Close,
    /// Visual state changed.
    Changed,
    /// Nothing changed.
    Unchanged,
}

/// Handle a key event in the stash modal.
///
/// When `pending_delete` is `Some`, the next `d` confirms deletion;
/// any other key cancels the arm.
pub fn handle_key(
    key: &crossterm::event::KeyEvent,
    state: &mut PickerState,
    entry_count: usize,
    pending_delete: &mut Option<usize>,
) -> StashOutcome {
    use crossterm::event::{Event, KeyCode};

    // Ctrl+./Ctrl+X close the modal.
    if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('.') | KeyCode::Char('x'))
    {
        return StashOutcome::Close;
    }

    // Delete confirmation phase: `d` confirms, any other key cancels.
    if let Some(armed_idx) = *pending_delete {
        if key.code == KeyCode::Char('d') && key.modifiers.is_empty() {
            *pending_delete = None;
            return StashOutcome::Delete(armed_idx);
        }
        // Any other key cancels.
        *pending_delete = None;
        // Fall through so the key's normal action can still apply (e.g.
        // Down after cancelling navigates).
    }

    // Esc closes.
    if key.code == KeyCode::Esc {
        return StashOutcome::Close;
    }

    // Enter restores the selected entry.
    if key.code == KeyCode::Enter && entry_count > 0 {
        // Clamp the selected index
        let sel = state.selected.min(entry_count.saturating_sub(1));
        return StashOutcome::Restore(sel);
    }

    // `d` arms delete on the selected entry.
    if key.code == KeyCode::Char('d') && key.modifiers.is_empty() && entry_count > 0 {
        let sel = state.selected.min(entry_count.saturating_sub(1));
        *pending_delete = Some(sel);
        return StashOutcome::Changed;
    }

    // Fall through to the generic picker handler for navigation, search, etc.
    let config = PickerConfig {
        title: None,
        show_search_hint: true,
        expandable: false,
        esc_clears_query: true,
        shortcuts: None,
        pending_hint: None,
        non_selectable: &[],
        non_selectable_clickable: &[],
        shortcuts_area: None,
        tabs: None,
        active_tab: 0,
        filter_label: None,
        filter_key_hint: None,
        filter_active: false,
        action_keys: &[],
        disable_search: false,
        compact_bottom_bar: false,
        search_only_on_slash: false,
        vim_normal_first: crate::appearance::cache::load_vim_mode(),
    };

    let ev = Event::Key(*key);
    match handle_picker_input(&ev, state, entry_count, &config) {
        PickerOutcome::Selected(idx) => StashOutcome::Restore(idx),
        PickerOutcome::Closed => StashOutcome::Close,
        PickerOutcome::Changed | PickerOutcome::QueryChanged => StashOutcome::Changed,
        _ => StashOutcome::Unchanged,
    }
}

/// Handle a mouse event in the stash modal.
pub fn handle_mouse(
    mouse: &crossterm::event::MouseEvent,
    state: &mut PickerState,
    entry_count: usize,
) -> StashOutcome {
    let config = PickerConfig {
        title: None,
        show_search_hint: true,
        expandable: false,
        esc_clears_query: true,
        shortcuts: None,
        pending_hint: None,
        non_selectable: &[],
        non_selectable_clickable: &[],
        shortcuts_area: None,
        tabs: None,
        active_tab: 0,
        filter_label: None,
        filter_key_hint: None,
        filter_active: false,
        action_keys: &[],
        disable_search: false,
        compact_bottom_bar: false,
        search_only_on_slash: false,
        vim_normal_first: crate::appearance::cache::load_vim_mode(),
    };

    let ev = crossterm::event::Event::Mouse(*mouse);
    match handle_picker_input(&ev, state, entry_count, &config) {
        PickerOutcome::Selected(idx) => StashOutcome::Restore(idx),
        PickerOutcome::Closed => StashOutcome::Close,
        PickerOutcome::Changed | PickerOutcome::QueryChanged => StashOutcome::Changed,
        _ => StashOutcome::Unchanged,
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Render the stash modal in full (chrome + picker content).
#[allow(clippy::too_many_arguments)]
pub fn render_modal(
    buf: &mut ratatui::buffer::Buffer,
    area: ratatui::layout::Rect,
    state: &mut PickerState,
    window: &mut crate::views::modal_window::ModalWindowState,
    row_data: &[StashRowData],
    pending_delete: bool,
    theme: &crate::theme::Theme,
    compact: bool,
) {
    use crate::views::modal_window as mw;
    use crate::views::picker::{self, PickerEntry, PickerRow};

    let footer = modal_footer(pending_delete);
    let modal_config = mw::ModalWindowConfig {
        title: "Stash",
        tabs: None,
        shortcuts: &footer,
        sizing: modal_sizing(compact),
        fold_info: None,
    };

    let Some(mca) = mw::render_modal_window(buf, area, window, &modal_config, theme) else {
        return;
    };

    let content_area = mca.content;

    // Build picker entries from stash rows.
    let picker_entries: Vec<PickerEntry> = row_data
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let selected = state.hovered == Some(i)
                || (state.hovered.is_none() && i == state.selected);
            PickerEntry::Row(PickerRow {
                label: row.preview.as_str(),
                right_label: row.right_label.as_str(),
                selected,
                expanded: false,
                fields: &[],
                description_lines: &[],
                summary_lines: &[],
                dimmed: false,
                indent: 0,
                badge: "",
                badge_color: None,
                collapsible: false,
                underline_last_desc: false,
            })
        })
        .collect();

    let non_sel: Vec<bool> = vec![false; picker_entries.len()];

    picker::render_picker_in_modal(
        buf,
        content_area,
        mca.inner_x,
        mca.inner_width,
        theme,
        state,
        &picker_entries,
        &non_sel,
        false,
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    fn make_entry(text: &str, index: usize) -> StashEntry {
        let preview = text.lines().next().unwrap_or("").to_string();
        let line_count = text.lines().count() as u16;
        StashEntry {
            preview,
            full_text: text.to_string(),
            line_count,
            index,
        }
    }

    #[test]
    fn truncate_preview_short_text() {
        assert_eq!(truncate_preview("hello", 50), "hello");
    }

    #[test]
    fn truncate_preview_long_text() {
        let long = "a".repeat(100);
        let result = truncate_preview(&long, 50);
        assert!(result.ends_with('\u{2026}'));
        assert!(result.width() <= 50);
    }

    #[test]
    fn truncate_preview_multi_line_takes_first_line() {
        assert_eq!(truncate_preview("first line\nsecond line", 50), "first line");
    }

    #[test]
    fn new_modal_with_entries() {
        let entries = vec![
            make_entry("stash one", 0),
            make_entry("stash two\nmore lines\nand more", 1),
        ];
        let modal = StashModalState::new(entries);
        assert_eq!(modal.entries.len(), 2);
        assert_eq!(modal.entries[0].preview, "stash one");
        assert_eq!(modal.entries[1].right_label, "  3 lines");
    }

    #[test]
    fn key_enter_restores_selected() {
        let mut state = PickerState::default();
        let mut pending = None;
        let outcome = handle_key(
            &crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::NONE,
            ),
            &mut state,
            3,
            &mut pending,
        );
        assert!(matches!(outcome, StashOutcome::Restore(0)));
    }

    #[test]
    fn key_esc_closes() {
        let mut state = PickerState::default();
        let mut pending = None;
        let outcome = handle_key(
            &crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Esc,
                crossterm::event::KeyModifiers::NONE,
            ),
            &mut state,
            3,
            &mut pending,
        );
        assert!(matches!(outcome, StashOutcome::Close));
    }

    #[test]
    fn key_d_arms_then_d_confirms_delete() {
        let mut state = PickerState::default();
        let mut pending = None;

        // First `d` arms deletion on selected (0).
        let outcome = handle_key(
            &crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('d'),
                crossterm::event::KeyModifiers::NONE,
            ),
            &mut state,
            3,
            &mut pending,
        );
        assert!(matches!(outcome, StashOutcome::Changed));
        assert_eq!(pending, Some(0));

        // Second `d` confirms deletion.
        let outcome = handle_key(
            &crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('d'),
                crossterm::event::KeyModifiers::NONE,
            ),
            &mut state,
            3,
            &mut pending,
        );
        assert!(matches!(outcome, StashOutcome::Delete(0)));
        assert_eq!(pending, None);
    }

    #[test]
    fn key_other_after_d_cancels_delete() {
        let mut state = PickerState::default();
        let mut pending = Some(0);

        // A navigation key cancels the armed deletion.
        let outcome = handle_key(
            &crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Down,
                crossterm::event::KeyModifiers::NONE,
            ),
            &mut state,
            3,
            &mut pending,
        );
        assert_eq!(pending, None, "non-d key cancels the arm");
        // outcome should be Changed (because it fell through to picker handler)
        assert!(matches!(outcome, StashOutcome::Changed));
    }

    #[test]
    fn ctrl_dot_closes() {
        let mut state = PickerState::default();
        let mut pending = None;
        let outcome = handle_key(
            &crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('.'),
                crossterm::event::KeyModifiers::CONTROL,
            ),
            &mut state,
            3,
            &mut pending,
        );
        assert!(matches!(outcome, StashOutcome::Close));
    }

    #[test]
    fn down_navigates() {
        let mut state = PickerState::default();
        let mut pending = None;
        let outcome = handle_key(
            &crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Down,
                crossterm::event::KeyModifiers::NONE,
            ),
            &mut state,
            3,
            &mut pending,
        );
        assert!(matches!(outcome, StashOutcome::Changed));
        assert_eq!(state.selected, 1);
    }
}
