//! Which-key overlay — compact, context-aware binding preview shown at the
//! bottom of the screen on Ctrl+K.
//!
//! Unlike the full `ShortcutsHelp` modal (Ctrl+./Ctrl+X), which-key is a
//! lightweight inline overlay that shows currently active bindings grouped
//! by category. It auto-dismisses on Esc or action dispatch.
//!
//! Ported conceptually from OpenCode's which-key plugin (`which-key.tsx`):
//! groups bindings by category, uses a compact multi-column layout, and
//! overlays the bottom chrome so you can see which keys do what without
//! leaving the current view.

use crate::actions::{ActionDef, ActionRegistry, Category, When};
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Active which-key overlay state.
#[derive(Debug, Clone)]
pub struct WhichKeyState {
    /// Grouped entries built from the registry.
    pub groups: Vec<WhichKeyGroup>,
    /// Index into `groups` of the currently selected/active group tab.
    pub active_group: usize,
    /// Vertical scroll offset within the active group's items.
    pub scroll: u16,
}

/// One category group in the which-key overlay.
#[derive(Debug, Clone)]
pub struct WhichKeyGroup {
    pub label: &'static str,
    pub items: Vec<WhichKeyItem>,
}

/// One binding row in a which-key group.
#[derive(Debug, Clone)]
pub struct WhichKeyItem {
    /// Human-friendly key display (e.g. "Ctrl+C", "j/↓", "Enter").
    pub key_display: String,
    /// Short label matching the shortcuts bar (e.g. "cancel", "nav", "send").
    pub label: &'static str,
    /// Longer description shown when space permits.
    pub description: &'static str,
    /// Whether this binding continues a chord (prefix key).
    pub continues: bool,
}

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

/// Height of the which-key panel in lines.
pub const PANEL_HEIGHT: u16 = 8;
/// Maximum column width for binding items.
const MAX_COL_WIDTH: usize = 42;
/// Gap between columns in cells.
const COL_GAP: u16 = 3;
/// Left/right padding inside the panel.
const HPAD: u16 = 2;

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Build which-key groups from the action registry for the given active contexts.
///
/// Only includes actions that have a visible hint (non‑confirmation, non‑null key)
/// and are relevant to the current set of contexts.
pub fn build_groups(
    active_contexts: &[When],
    registry: &ActionRegistry,
    vim_mode: bool,
) -> Vec<WhichKeyGroup> {
    // Use category order from shortcuts_help.
    let category_order = &[
        (Category::GettingStarted, "Essentials"),
        (Category::Input, "Input"),
        (Category::ConversationNav, "Navigation"),
        (Category::ConversationAction, "View"),
        (Category::Panels, "Panels"),
        (Category::Session, "Session"),
        (Category::Dashboard, "Dashboard"),
    ];

    let mut groups: Vec<WhichKeyGroup> = Vec::new();

    for &(cat, label) in category_order {
        let defs: Vec<&ActionDef> = registry
            .all()
            .iter()
            .filter(|d| d.category == cat && d.hint_priority.is_some())
            .filter(|d| {
                // Skip actions with no real keybinding.
                d.default_key != crate::key!(Null) || !d.alt_keys.is_empty()
            })
            .filter(|d| {
                // In non-vim mode, suppress bare-letter scrollback bindings
                // (they'd show as usable but aren't).
                if !vim_mode && d.context == When::ScrollbackFocused {
                    let is_letter = d.default_key.is_letter_or_shift_letter()
                        && d.alt_keys.iter().all(|k| k.is_letter_or_shift_letter());
                    return !is_letter;
                }
                true
            })
            .filter(|d| {
                // Only show actions whose context is currently active.
                active_contexts.contains(&d.context)
                    || d.context == When::Always
            })
            .collect();

        if defs.is_empty() {
            continue;
        }

        let items: Vec<WhichKeyItem> = defs
            .iter()
            .map(|def| {
                // Build key display: prefer hint_key_display, else join keys.
                let key_display = if let Some(display) = def.hint_key_display {
                    display.to_string()
                } else {
                    let mut keys = vec![def.default_key];
                    keys.extend_from_slice(&def.alt_keys);
                    // Dedup identical displays.
                    let mut seen = std::collections::HashSet::new();
                    keys.retain(|k| seen.insert(k.display_pretty()));
                    keys.iter()
                        .map(|k| k.display_pretty())
                        .collect::<Vec<_>>()
                        .join(" / ")
                };
                WhichKeyItem {
                    key_display,
                    label: def.label,
                    description: def.description,
                    continues: false,
                }
            })
            .collect();

        groups.push(WhichKeyGroup {
            label,
            items,
        });
    }

    groups
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Render the which-key overlay into a rectangle at the bottom of the screen.
///
/// `area` is the full width of the terminal at the Y position where the overlay
/// should appear. The overlay draws `PANEL_HEIGHT` lines starting from `area.y`.
pub fn render_which_key(
    buf: &mut Buffer,
    area: Rect,
    state: &WhichKeyState,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    // We need at least PANEL_HEIGHT lines.
    let height = PANEL_HEIGHT.min(area.height);
    let panel_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height,
    };

    // Draw backdrop.
    let bg = theme.bg_dark;
    let border_style = Style::default().fg(theme.gray_dim).bg(bg);
    let muted_style = Style::default().fg(theme.gray).bg(bg);

    // Draw backdrop and top separator line.
    for y in panel_area.y..panel_area.y + panel_area.height {
        let is_sep = y == panel_area.y;
        let style = if is_sep {
            Style::default().fg(theme.gray_dim).bg(bg)
        } else {
            border_style
        };
        for x in panel_area.x..panel_area.x + panel_area.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_style(style);
                cell.set_symbol(if is_sep { "─" } else { " " });
            }
        }
    }

    // Group tab bar (row 1).
    if panel_area.y + 1 < panel_area.y + panel_area.height {
        let mut tab_x = panel_area.x + HPAD;
        for (gi, group) in state.groups.iter().enumerate() {
            if tab_x >= panel_area.x + panel_area.width - HPAD {
                break;
            }
            let selected = gi == state.active_group;
            let tab_fg = if selected {
                theme.accent_user
            } else {
                theme.gray
            };
            let tab_bg = if selected {
                theme.bg_visual
            } else {
                bg
            };
            let tab_style = Style::default()
                .fg(tab_fg)
                .bg(tab_bg)
                .add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                });
            let tab_text = if selected {
                format!(" ▎{} ", group.label)
            } else {
                format!("  {}  ", group.label)
            };
            // Ensure we handle a possible non-ascii separator.
            let tab_w = tab_text.width() as u16;
            if tab_x + tab_w > panel_area.x + panel_area.width - HPAD {
                // Truncate / skip this tab.
                break;
            }
            buf.set_span(tab_x, panel_area.y + 1, &Span::styled(&tab_text, tab_style), tab_w);
            tab_x += tab_w;
            // Draw a subtle separator between tabs.
            if gi + 1 < state.groups.len() && tab_x < panel_area.x + panel_area.width {
                if let Some(cell) = buf.cell_mut((tab_x, panel_area.y + 1)) {
                    cell.set_symbol("│");
                    cell.set_style(Style::default().fg(theme.gray_dim).bg(bg));
                }
                tab_x += 1;
            }
        }
    }

    // Items area (rows 2+).
    let items_y = panel_area.y + 1 + 1;
    let items_h = height.saturating_sub(2).min( // header row 1 + items
        (panel_area.y + height).saturating_sub(items_y)
    );
    if items_h == 0 {
        return;
    }

    let inner_w = panel_area.width.saturating_sub(HPAD * 2);
    if inner_w == 0 {
        return;
    }

    let active_group = state.groups.get(state.active_group);
    let Some(group) = active_group else { return };

    if group.items.is_empty() {
        let empty_text = Span::styled("No active bindings", muted_style);
        buf.set_span(
            panel_area.x + HPAD,
            items_y,
            &empty_text,
            inner_w,
        );
        return;
    }

    // Compute column layout.
    let col_w = MAX_COL_WIDTH.min(inner_w as usize);
    let n_cols = ((inner_w as usize + COL_GAP as usize) / (col_w + COL_GAP as usize))
        .max(1);
    let actual_col_w = (inner_w as usize - (n_cols - 1) * COL_GAP as usize) / n_cols;
    let col_w_u16 = actual_col_w as u16;

    let items_per_page = items_h as usize * n_cols;
    let total_items = group.items.len();
    let max_scroll = total_items.saturating_sub(items_per_page);
    let scroll = (state.scroll as usize).min(max_scroll);

    // Render items as a grid: fill columns first, then rows.
    for item_idx in 0..items_per_page {
        let flat_idx = scroll + item_idx;
        if flat_idx >= total_items {
            break;
        }
        let item = &group.items[flat_idx];

        let col = item_idx % n_cols;
        let row = item_idx / n_cols;
        let x = panel_area.x + HPAD + (col as u16) * (col_w_u16 + COL_GAP);
        let y = items_y + row as u16;

        if y >= panel_area.y + height {
            break;
        }

        // Determine available key width: prefer up to ~40% of column.
        let key_budget = (actual_col_w / 2).max(8);
        let label_budget = actual_col_w.saturating_sub(key_budget + 1);

        let key_text = truncate_to_width(&item.key_display, key_budget);
        let label_text = truncate_to_width(item.label, label_budget);

        let key_style = if item.continues {
            Style::default()
                .fg(theme.warning)
                .bg(bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.accent_user)
                .bg(bg)
                .add_modifier(Modifier::BOLD)
        };
        let label_style = muted_style;

        buf.set_span(x, y, &Span::styled(&key_text, key_style), key_budget as u16);
        if key_text.len() < key_budget {
            let sep_x = x + key_text.width() as u16;
            buf.set_span(
                sep_x,
                y,
                &Span::styled(" ", label_style),
                1,
            );
        }
        let label_x = x + key_budget as u16 + 1;
        buf.set_span(label_x, y, &Span::styled(&label_text, label_style), label_budget as u16);
    }

    // Footer line: hint text.
    let footer_y = panel_area.y + height - 1;
    if footer_y > items_y {
        let footer_text = format!(
            " h/l prev/next group  ↑/↓ scroll  Esc close  · {} items · {} groups",
            total_items,
            state.groups.len(),
        );
        let footer_span = Span::styled(&footer_text, muted_style);
        buf.set_span(
            panel_area.x + HPAD,
            footer_y,
            &footer_span,
            inner_w,
        );
    }
}

/// Truncate text to fit `max_width` columns, adding "…" when truncated.
fn truncate_to_width(text: &str, max_width: usize) -> String {
    let w = text.width();
    if w <= max_width {
        return text.to_string();
    }
    // Reserve 1 column for the ellipsis character.
    let budget = max_width.saturating_sub(1);
    let mut result = String::new();
    let mut cur_w = 0;
    for ch in text.chars() {
        let cw = ch.width().unwrap_or(1);
        if cur_w + cw > budget {
            result.push('…');
            break;
        }
        result.push(ch);
        cur_w += cw;
    }
    result
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// Outcome of handling a key press while the which-key overlay is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichKeyOutcome {
    /// Close the overlay (Esc, Ctrl+K again).
    Close,
    /// Selection/scroll changed — redraw.
    Changed,
    /// Key was not handled by which-key — let it fall through.
    Unchanged,
}

/// Handle a key event for the which-key overlay.
pub fn handle_input(
    key: &crossterm::event::KeyEvent,
    state: &mut WhichKeyState,
) -> WhichKeyOutcome {
    use crossterm::event::{KeyCode, KeyModifiers};

    // Close on Ctrl+K, Ctrl+., Ctrl+X, or Esc.
    if key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('k') | KeyCode::Char('.') | KeyCode::Char('x')))
    {
        return WhichKeyOutcome::Close;
    }

    if state.groups.is_empty() {
        return WhichKeyOutcome::Unchanged;
    }

    let current_group = &state.groups[state.active_group];
    let items_total = current_group.items.len();
    let n_cols = 3.max(1); // rough column estimate

    match key.code {
        // Next group (right tab)
        KeyCode::Right | KeyCode::Char('l') if key.modifiers.is_empty() => {
            if state.active_group + 1 < state.groups.len() {
                state.active_group += 1;
                state.scroll = 0;
                WhichKeyOutcome::Changed
            } else {
                WhichKeyOutcome::Unchanged
            }
        }
        // Previous group (left tab)
        KeyCode::Left | KeyCode::Char('h') if key.modifiers.is_empty() => {
            if state.active_group > 0 {
                state.active_group -= 1;
                state.scroll = 0;
                WhichKeyOutcome::Changed
            } else {
                WhichKeyOutcome::Unchanged
            }
        }
        // Scroll down
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
            let page = items_per_page(items_total, n_cols);
            if state.scroll as usize + page < items_total {
                state.scroll += 1;
                WhichKeyOutcome::Changed
            } else {
                WhichKeyOutcome::Unchanged
            }
        }
        // Scroll up
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => {
            if state.scroll > 0 {
                state.scroll -= 1;
                WhichKeyOutcome::Changed
            } else {
                WhichKeyOutcome::Unchanged
            }
        }
        // Page down
        KeyCode::PageDown => {
            let page = items_per_page(items_total, n_cols);
            let max_scroll = items_total.saturating_sub(page);
            if (state.scroll as usize) < max_scroll {
                state.scroll = (max_scroll.min(state.scroll as usize + page)) as u16;
                WhichKeyOutcome::Changed
            } else {
                WhichKeyOutcome::Unchanged
            }
        }
        // Page up
        KeyCode::PageUp => {
            let page = items_per_page(items_total, n_cols);
            state.scroll = (state.scroll as usize).saturating_sub(page) as u16;
            WhichKeyOutcome::Changed
        }
        // Home
        KeyCode::Home => {
            if state.scroll > 0 {
                state.scroll = 0;
                WhichKeyOutcome::Changed
            } else {
                WhichKeyOutcome::Unchanged
            }
        }
        // End
        KeyCode::End => {
            let max_scroll = items_total.saturating_sub(items_per_page(items_total, n_cols));
            if (state.scroll as usize) < max_scroll {
                state.scroll = max_scroll as u16;
                WhichKeyOutcome::Changed
            } else {
                WhichKeyOutcome::Unchanged
            }
        }
        _ => WhichKeyOutcome::Unchanged,
    }
}

fn items_per_page(_total: usize, n_cols: usize) -> usize {
    let rows = (PANEL_HEIGHT as usize).saturating_sub(3).max(1); // header + footer + top border
    rows * n_cols
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key;

    #[test]
    fn build_groups_returns_groups() {
        let registry = crate::actions::ActionRegistry::defaults();
        let contexts = &[When::AgentScreen, When::Always];
        let groups = build_groups(contexts, &registry, true);
        // At minimum we should have Essentials (GettingStarted), Input, Panels, Session, Navigation, View, Dashboard
        assert!(!groups.is_empty(), "should have at least one group");
        assert!(
            groups.iter().any(|g| g.label == "Essentials"),
            "should include Essentials group"
        );
        assert!(
            groups.iter().any(|g| g.label == "Panels"),
            "should include Panels group"
        );
    }

    #[test]
    fn build_groups_with_scrollback_context() {
        let registry = crate::actions::ActionRegistry::defaults();
        let contexts = &[When::ScrollbackFocused, When::AgentScreen, When::Always];
        let groups = build_groups(contexts, &registry, true);
        let nav = groups.iter().find(|g| g.label == "Navigation");
        assert!(nav.is_some(), "Navigation group should exist");
        assert!(!nav.unwrap().items.is_empty(), "Navigation should have items");
    }

    #[test]
    fn build_groups_suppresses_purely_letter_bindings_in_non_vim() {
        // In non-vim mode, bindings whose ONLY keys are bare letters
        // (e.g. GotoTop 'g' with no alt) should be suppressed.
        let registry = crate::actions::ActionRegistry::defaults();
        let contexts = &[When::ScrollbackFocused, When::Always];
        let groups = build_groups(contexts, &registry, false);
        // Navigation items that DO appear (like j/k via hint_key_display)
        // have at least one non-letter alt key (arrow), so they survive.
        // Items like GotoTop 'g' (letter-only, no arrow alt) should be suppressed.
        // Verify no lone-letter items appear.
        for group in &groups {
            for item in &group.items {
                let trimmed = item.key_display.trim();
                let is_lone_letter = trimmed.len() == 1
                    && trimmed.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
                assert!(
                    !is_lone_letter,
                    "lone letter binding should be suppressed in non-vim: '{}'",
                    trimmed
                );
            }
        }
        // Verify Navigation group still exists (j/k with ↓ alt survives).
        let nav = groups.iter().find(|g| g.label == "Navigation");
        assert!(
            nav.is_some(),
            "Navigation group should still exist in non-vim"
        );
        assert!(
            !nav.unwrap().items.is_empty(),
            "Navigation should have arrow-supported items"
        );
    }

    #[test]
    fn handle_input_closes_on_esc() {
        let registry = crate::actions::ActionRegistry::defaults();
        let contexts = &[When::AgentScreen, When::Always];
        let groups = build_groups(contexts, &registry, true);
        let mut state = WhichKeyState {
            groups,
            active_group: 0,
            scroll: 0,
        };
        let esc = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        );
        assert_eq!(handle_input(&esc, &mut state), WhichKeyOutcome::Close);
    }

    #[test]
    fn handle_input_closes_on_ctrl_k() {
        let registry = crate::actions::ActionRegistry::defaults();
        let contexts = &[When::AgentScreen, When::Always];
        let groups = build_groups(contexts, &registry, true);
        let mut state = WhichKeyState {
            groups,
            active_group: 0,
            scroll: 0,
        };
        let ctrl_k = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('k'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        assert_eq!(handle_input(&ctrl_k, &mut state), WhichKeyOutcome::Close);
    }

    #[test]
    fn handle_input_navigates_groups() {
        let registry = crate::actions::ActionRegistry::defaults();
        let contexts = &[When::AgentScreen, When::Always];
        let groups = build_groups(contexts, &registry, true);
        if groups.len() < 2 {
            return; // need at least 2 groups to test navigation
        }
        let mut state = WhichKeyState {
            groups,
            active_group: 0,
            scroll: 0,
        };
        let right = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Right,
            crossterm::event::KeyModifiers::NONE,
        );
        assert_eq!(
            handle_input(&right, &mut state),
            WhichKeyOutcome::Changed
        );
        assert_eq!(state.active_group, 1);

        let left = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Left,
            crossterm::event::KeyModifiers::NONE,
        );
        assert_eq!(
            handle_input(&left, &mut state),
            WhichKeyOutcome::Changed
        );
        assert_eq!(state.active_group, 0);
    }

    #[test]
    fn truncate_fits_within_width() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
        assert_eq!(truncate_to_width("hello world", 5), "hell…");
        assert_eq!(truncate_to_width("Ctrl+Shift+P", 6), "Ctrl+…");
    }
}
