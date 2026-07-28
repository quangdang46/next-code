//! Face `/experimental` checklist — Codex-parity experiment mode UX.
//!
//! Lists `Stage::Experimental` flags from `next-code-experiment-flags`,
//! Space toggles enablement, Enter persists `[experiments]` to config.toml.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use next_code_experiment_flags::{Experiments, ExperimentalMenuItem};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::app::app_view::InputOutcome;
use crate::render::SafeBuf;
use crate::theme::Theme;
use crate::views::modal_window::{
    self, ModalSizing, ModalWindowConfig, ModalWindowState, Shortcut,
};

/// Outcome from key/mouse handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExperimentalModalOutcome {
    Changed,
    Unchanged,
    /// Persist current toggles and close.
    SaveAndClose,
    /// Close without writing config.
    Cancel,
}

#[derive(Debug, Clone)]
pub struct ExperimentalModalState {
    pub window: ModalWindowState,
    pub items: Vec<ExperimentalMenuItem>,
    pub selected: usize,
    pub scroll_offset: usize,
    list_area: Rect,
}

impl ExperimentalModalState {
    pub fn from_experiments(experiments: &Experiments) -> Self {
        let items = experiments.experimental_menu_items();
        Self {
            window: ModalWindowState::new(),
            items,
            selected: 0,
            scroll_offset: 0,
            list_area: Rect::default(),
        }
    }

    pub fn load_from_config() -> Self {
        let experiments = load_experiments_from_config();
        Self::from_experiments(&experiments)
    }

    fn visible_len(&self) -> usize {
        self.items.len()
    }

    fn clamp_selection(&mut self) {
        if self.items.is_empty() {
            self.selected = 0;
            self.scroll_offset = 0;
            return;
        }
        if self.selected >= self.items.len() {
            self.selected = self.items.len() - 1;
        }
    }

    fn ensure_visible(&mut self, viewport: usize) {
        if self.items.is_empty() || viewport == 0 {
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + viewport {
            self.scroll_offset = self.selected + 1 - viewport;
        }
    }

    fn toggle_selected(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected) {
            item.enabled = !item.enabled;
        }
    }

    /// Key/value pairs to write under `[experiments]`.
    pub fn overrides(&self) -> Vec<(String, bool)> {
        self.items
            .iter()
            .map(|i| (i.key.to_string(), i.enabled))
            .collect()
    }
}

fn load_experiments_from_config() -> Experiments {
    let path = xai_grok_config::grok_home().join("config.toml");
    let mut entries = std::collections::BTreeMap::new();
    if let Ok(content) = std::fs::read_to_string(&path)
        && let Ok(value) = content.parse::<toml::Value>()
        && let Some(table) = value.get("experiments").and_then(|v| v.as_table())
    {
        for (k, v) in table {
            if let Some(b) = v.as_bool() {
                entries.insert(k.clone(), b);
            }
        }
    }
    Experiments::from_config(&entries)
}

/// Persist checklist overrides into `[experiments]` without clobbering other config.
pub fn persist_experiment_overrides(updates: &[(String, bool)]) -> Result<(), String> {
    let path = xai_grok_config::grok_home().join("config.toml");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    let mut doc = if raw.trim().is_empty() {
        toml_edit::DocumentMut::new()
    } else {
        raw.parse::<toml_edit::DocumentMut>()
            .map_err(|e| format!("Could not parse config.toml: {e}"))?
    };
    let experiments = doc
        .entry("experiments")
        .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
        .as_table_mut()
        .ok_or_else(|| "experiments section is not a table".to_string())?;
    for (key, enabled) in updates {
        experiments.insert(key, toml_edit::value(*enabled));
    }
    std::fs::write(&path, doc.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn handle_experimental_key(
    state: &mut ExperimentalModalState,
    key: &KeyEvent,
) -> ExperimentalModalOutcome {
    if key.kind == KeyEventKind::Release {
        return ExperimentalModalOutcome::Unchanged;
    }
    match key.code {
        KeyCode::Esc => ExperimentalModalOutcome::Cancel,
        KeyCode::Enter => ExperimentalModalOutcome::SaveAndClose,
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => {
            if state.selected > 0 {
                state.selected -= 1;
            }
            ExperimentalModalOutcome::Changed
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
            if state.selected + 1 < state.visible_len() {
                state.selected += 1;
            }
            ExperimentalModalOutcome::Changed
        }
        KeyCode::Char(' ') if key.modifiers.is_empty() => {
            state.toggle_selected();
            ExperimentalModalOutcome::Changed
        }
        KeyCode::Home => {
            state.selected = 0;
            ExperimentalModalOutcome::Changed
        }
        KeyCode::End => {
            if !state.items.is_empty() {
                state.selected = state.items.len() - 1;
            }
            ExperimentalModalOutcome::Changed
        }
        _ => ExperimentalModalOutcome::Unchanged,
    }
}

pub fn handle_experimental_mouse(
    state: &mut ExperimentalModalState,
    kind: MouseEventKind,
    column: u16,
    row: u16,
) -> ExperimentalModalOutcome {
    let list = state.list_area;
    if list.width == 0 || list.height == 0 {
        return ExperimentalModalOutcome::Unchanged;
    }
    match kind {
        MouseEventKind::ScrollUp => {
            if state.selected > 0 {
                state.selected -= 1;
            }
            ExperimentalModalOutcome::Changed
        }
        MouseEventKind::ScrollDown => {
            if state.selected + 1 < state.visible_len() {
                state.selected += 1;
            }
            ExperimentalModalOutcome::Changed
        }
        MouseEventKind::Down(_) => {
            if column < list.x
                || column >= list.x.saturating_add(list.width)
                || row < list.y
                || row >= list.y.saturating_add(list.height)
            {
                return ExperimentalModalOutcome::Unchanged;
            }
            let rel = (row - list.y) as usize;
            let idx = state.scroll_offset + rel;
            if idx < state.items.len() {
                state.selected = idx;
                return ExperimentalModalOutcome::Changed;
            }
            ExperimentalModalOutcome::Unchanged
        }
        _ => ExperimentalModalOutcome::Unchanged,
    }
}

pub fn render_experimental_modal(
    buf: &mut Buffer,
    area: Rect,
    state: &mut ExperimentalModalState,
    compact: bool,
) {
    let theme = Theme::current();
    let shortcuts = [
        Shortcut {
            label: "space toggle",
            clickable: false,
            id: 0,
        },
        Shortcut {
            label: "enter save",
            clickable: false,
            id: 0,
        },
        Shortcut {
            label: "esc cancel",
            clickable: false,
            id: 0,
        },
    ];
    let sizing = if compact {
        ModalSizing {
            width_pct: 0.70,
            max_width: 72,
            min_width: 44,
            v_margin: 4,
            h_pad: 2,
            v_pad: 1,
            footer_lines: 2,
        }
    } else {
        ModalSizing {
            width_pct: 0.75,
            max_width: 84,
            min_width: 48,
            v_margin: 4,
            h_pad: 2,
            v_pad: 1,
            footer_lines: 2,
        }
    };
    let cfg = ModalWindowConfig {
        title: "Experimental features",
        tabs: None,
        shortcuts: &shortcuts,
        sizing,
        fold_info: None,
    };
    let Some(mca) = modal_window::render_modal_window(buf, area, &mut state.window, &cfg, &theme)
    else {
        return;
    };

    let content = mca.content;
    if content.height == 0 || content.width == 0 {
        return;
    }

    let mut y = content.y;
    let hint = "Toggle experimental features. Changes are saved to config.toml.";
    write_line(buf, content.x, y, content.width, hint, theme.muted());
    y = y.saturating_add(1);

    if y >= content.y.saturating_add(content.height) {
        return;
    }

    let list_height = content
        .y
        .saturating_add(content.height)
        .saturating_sub(y)
        .saturating_sub(1) as usize;
    state.list_area = Rect {
        x: content.x,
        y,
        width: content.width,
        height: list_height as u16,
    };
    state.clamp_selection();
    state.ensure_visible(list_height.max(1));

    if state.items.is_empty() {
        write_line(
            buf,
            content.x,
            y,
            content.width,
            "  No experimental features available for now",
            theme.muted(),
        );
        return;
    }

    let end = (state.scroll_offset + list_height).min(state.items.len());
    for (row_i, item) in state.items[state.scroll_offset..end].iter().enumerate() {
        let idx = state.scroll_offset + row_i;
        let selected = idx == state.selected;
        let marker = if item.enabled { 'x' } else { ' ' };
        let prefix = if selected { '›' } else { ' ' };
        let label = format!("{prefix} [{marker}] {}", item.name);
        let mut style = Style::default().fg(theme.text_primary);
        if selected {
            style = style.add_modifier(Modifier::BOLD).fg(theme.fuzzy_accent);
        }
        let row_y = y.saturating_add(row_i as u16);
        write_line(buf, content.x, row_y, content.width, &label, style);
        if content.width > 24 {
            let name_w = label.width() as u16;
            if name_w + 2 < content.width {
                let desc_x = content.x.saturating_add(name_w).saturating_add(2);
                let desc_w = content.width.saturating_sub(name_w.saturating_add(2));
                write_line(
                    buf,
                    desc_x,
                    row_y,
                    desc_w,
                    item.description,
                    Style::default().fg(theme.text_secondary),
                );
            }
        }
    }

    let footer_y = content.y.saturating_add(content.height.saturating_sub(1));
    write_line(
        buf,
        content.x,
        footer_y,
        content.width,
        "Press space to toggle · enter to save for next conversation",
        theme.muted(),
    );
}

fn write_line(buf: &mut Buffer, x: u16, y: u16, width: u16, text: &str, style: Style) {
    if width == 0 {
        return;
    }
    let clipped: String = text.chars().take(width as usize).collect();
    let line = Line::from(Span::styled(clipped, style));
    buf.set_line_safe(x, y, &line, width);
}

/// Apply modal outcome into an [`InputOutcome`] (caller owns modal close / toast).
pub fn outcome_to_input(outcome: ExperimentalModalOutcome) -> InputOutcome {
    match outcome {
        ExperimentalModalOutcome::Changed => InputOutcome::Changed,
        ExperimentalModalOutcome::Unchanged => InputOutcome::Unchanged,
        ExperimentalModalOutcome::SaveAndClose | ExperimentalModalOutcome::Cancel => {
            InputOutcome::Changed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;
    use next_code_experiment_flags::{ExperimentFlag, Stage, EXPERIMENT_FLAGS};

    #[test]
    fn load_defaults_includes_js_plugins_only_experimental() {
        let state = ExperimentalModalState::from_experiments(&Experiments::with_defaults());
        assert!(state.items.iter().any(|i| i.key == "js_plugins"));
        for item in &state.items {
            let spec = EXPERIMENT_FLAGS.iter().find(|s| s.id == item.flag).unwrap();
            assert!(matches!(spec.stage, Stage::Experimental { .. }));
        }
    }

    #[test]
    fn space_toggles_and_overrides_persist_shape() {
        let mut state = ExperimentalModalState::from_experiments(&Experiments::with_defaults());
        assert!(!state.items.is_empty());
        let before = state.items[0].enabled;
        state.selected = 0;
        assert_eq!(
            handle_experimental_key(
                &mut state,
                &KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)
            ),
            ExperimentalModalOutcome::Changed
        );
        assert_eq!(state.items[0].enabled, !before);
        let overrides = state.overrides();
        assert!(overrides.iter().any(|(k, v)| k == "js_plugins" && *v == state.items.iter().find(|i| i.key == "js_plugins").unwrap().enabled));
    }

    #[test]
    fn enter_requests_save() {
        let mut state = ExperimentalModalState::from_experiments(&Experiments::with_defaults());
        assert_eq!(
            handle_experimental_key(&mut state, &KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            ExperimentalModalOutcome::SaveAndClose
        );
    }

    #[test]
    fn register_recipe_experimental_appears_without_face_changes() {
        // Documented contract: any Stage::Experimental row in EXPERIMENT_FLAGS
        // appears in the Face menu automatically.
        let menu_keys: Vec<_> = Experiments::with_defaults()
            .experimental_menu_items()
            .into_iter()
            .map(|i| i.key)
            .collect();
        for spec in EXPERIMENT_FLAGS {
            if matches!(spec.stage, Stage::Experimental { .. }) {
                assert!(
                    menu_keys.contains(&spec.key),
                    "Experimental flag {} missing from menu",
                    spec.key
                );
            } else {
                assert!(
                    !menu_keys.contains(&spec.key),
                    "non-Experimental flag {} leaked into menu",
                    spec.key
                );
            }
        }
        let _ = ExperimentFlag::JsPlugins;
    }
}
