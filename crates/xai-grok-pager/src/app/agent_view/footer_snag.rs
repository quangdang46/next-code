//! Claude-style footer indicator keyboard snag.
//!
//! Ref: Claude `footerSelection` / `FooterItem` + `context: 'Footer'` bindings
//! (`.tmp/claude-code/src/state/AppStateStore.ts`, `PromptInput.tsx` navigateFooter,
//! `keybindings/defaultBindings.ts` Footer context).
//!
//! Face already paints clickable tasks / `@agent` pills (#133). This module
//! adds arrow-key focus + Enter open without requiring the mouse.

use std::fmt;

/// One focusable footer chrome item (left→right visual order for agents,
/// then tasks on the right — Claude lists tasks among left pills; Face keeps
/// tasks on the right but keyboard order is still stable: agents then tasks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FooterSnagItem {
    /// `@agent` pill at index into the current roster-for-pills slice.
    Agent { index: usize },
    /// Background-task SummaryPill (opens Tasks hub).
    Tasks,
}

impl fmt::Display for FooterSnagItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Agent { index } => write!(f, "agent[{index}]"),
            Self::Tasks => write!(f, "tasks"),
        }
    }
}

/// Build the navigable list for the current frame.
///
/// `agent_count` = number of `@agent` pills currently painted.
/// `tasks_visible` = whether the tasks SummaryPill is shown.
pub fn footer_items(agent_count: usize, tasks_visible: bool) -> Vec<FooterSnagItem> {
    let mut items = Vec::with_capacity(agent_count + usize::from(tasks_visible));
    for index in 0..agent_count {
        items.push(FooterSnagItem::Agent { index });
    }
    if tasks_visible {
        items.push(FooterSnagItem::Tasks);
    }
    items
}

/// delta +1 = next (right/down), -1 = previous (left/up).
///
/// When `selection` is `None` and delta is +1, selects the first item.
/// When at the start and delta is -1 with `exit_at_start`, clears selection.
/// Returns whether the selection changed (including clear).
pub fn navigate(
    selection: &mut Option<FooterSnagItem>,
    items: &[FooterSnagItem],
    delta: i8,
    exit_at_start: bool,
) -> bool {
    if items.is_empty() {
        let cleared = selection.take().is_some();
        return cleared;
    }
    let idx = selection
        .as_ref()
        .and_then(|s| items.iter().position(|i| i == s));
    match (idx, delta.signum()) {
        (None, 1) => {
            *selection = Some(items[0].clone());
            true
        }
        (None, _) => false,
        (Some(i), 1) => {
            if let Some(next) = items.get(i + 1) {
                *selection = Some(next.clone());
                true
            } else {
                false
            }
        }
        (Some(i), -1) => {
            if i == 0 {
                if exit_at_start {
                    *selection = None;
                    true
                } else {
                    false
                }
            } else {
                *selection = Some(items[i - 1].clone());
                true
            }
        }
        _ => false,
    }
}

/// Drop selection if the item is no longer visible (task finished, agents gone).
pub fn prune(selection: &mut Option<FooterSnagItem>, items: &[FooterSnagItem]) {
    if let Some(ref sel) = *selection
        && !items.iter().any(|i| i == sel)
    {
        *selection = None;
    }
}

use crate::app::agent_view::AgentView;
use crate::app::app_view::InputOutcome;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl AgentView {
    /// Visible footer snag items for the current frame (agents then tasks).
    pub(crate) fn footer_snag_items(&self) -> Vec<FooterSnagItem> {
        let roster = self.agent_team_roster();
        let agent_count = if roster.len() > 1 { roster.len() } else { 0 };
        let tasks_visible = self.watchers().shows_ambient();
        footer_items(agent_count, tasks_visible)
    }

    pub(crate) fn footer_snag_active(&self) -> bool {
        self.footer_snag.is_some()
    }

    /// Enter footer focus on the first visible pill (Claude Down→footer).
    pub(crate) fn footer_snag_enter(&mut self) -> bool {
        let items = self.footer_snag_items();
        if items.is_empty() {
            return false;
        }
        self.footer_snag = Some(items[0].clone());
        true
    }

    pub(crate) fn footer_snag_clear(&mut self) {
        self.footer_snag = None;
    }

    /// Handle keys while footer snag is focused. Returns `None` if the key
    /// should fall through (e.g. typing clears snag then edits).
    pub(crate) fn handle_footer_snag_key(&mut self, key: &KeyEvent) -> Option<InputOutcome> {
        let items = self.footer_snag_items();
        prune(&mut self.footer_snag, &items);
        if self.footer_snag.is_none() {
            return None;
        }

        // Esc / Ctrl+C: clear selection back to prompt.
        if matches!(key.code, KeyCode::Esc)
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.footer_snag_clear();
            return Some(InputOutcome::Changed);
        }

        // Enter: open selected pill.
        if key.code == KeyCode::Enter && key.modifiers.is_empty() {
            return Some(self.footer_snag_open_selected());
        }

        // Left / Ctrl+P / Up: previous (Up exits at start → prompt).
        if key.code == KeyCode::Left
            || key.code == KeyCode::Up
            || (key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            let exit = matches!(key.code, KeyCode::Up);
            let _ = navigate(&mut self.footer_snag, &items, -1, exit);
            return Some(InputOutcome::Changed);
        }

        // Right / Down / Ctrl+N: next.
        if key.code == KeyCode::Right
            || key.code == KeyCode::Down
            || (key.code == KeyCode::Char('n') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            let _ = navigate(&mut self.footer_snag, &items, 1, false);
            return Some(InputOutcome::Changed);
        }

        // Any other key: clear snag and let the prompt handle it.
        self.footer_snag_clear();
        None
    }

    fn footer_snag_open_selected(&mut self) -> InputOutcome {
        let sel = self.footer_snag.take();
        match sel {
            Some(FooterSnagItem::Tasks) => {
                self.tasks.open_hub();
                self.set_active_pane(super::AgentPane::Tasks, false);
                InputOutcome::Changed
            }
            Some(FooterSnagItem::Agent { index }) => {
                let roster = self.agent_team_roster();
                if let Some(row) = roster.get(index) {
                    let id = row.id.clone();
                    let _ = self.enter_agent_by_id(&id);
                }
                InputOutcome::Changed
            }
            None => InputOutcome::Unchanged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footer_items_agents_then_tasks() {
        assert_eq!(
            footer_items(2, true),
            vec![
                FooterSnagItem::Agent { index: 0 },
                FooterSnagItem::Agent { index: 1 },
                FooterSnagItem::Tasks,
            ]
        );
        assert!(footer_items(0, false).is_empty());
    }

    #[test]
    fn navigate_enters_and_exits() {
        let items = footer_items(1, true);
        let mut sel = None;
        assert!(navigate(&mut sel, &items, 1, true));
        assert_eq!(sel, Some(FooterSnagItem::Agent { index: 0 }));
        assert!(navigate(&mut sel, &items, 1, true));
        assert_eq!(sel, Some(FooterSnagItem::Tasks));
        assert!(!navigate(&mut sel, &items, 1, true));
        assert!(navigate(&mut sel, &items, -1, true));
        assert_eq!(sel, Some(FooterSnagItem::Agent { index: 0 }));
        assert!(navigate(&mut sel, &items, -1, true));
        assert_eq!(sel, None);
    }

    #[test]
    fn prune_drops_stale_tasks() {
        let mut sel = Some(FooterSnagItem::Tasks);
        prune(&mut sel, &footer_items(1, false));
        assert_eq!(sel, None);
    }
}
