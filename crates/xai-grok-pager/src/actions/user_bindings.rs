//! User keybindings file — Claude-shaped JSON merged onto Face `ActionRegistry`.
//!
//! Path: `{grok_home()}/keybindings.json` (typically `~/.next-code/keybindings.json`).
//!
//! Schema (Claude-compatible shape):
//! ```json
//! {
//!   "bindings": [
//!     {
//!       "context": "AgentScreen",
//!       "bindings": {
//!         "ctrl+h": "send_to_background",
//!         "ctrl+g": null
//!       }
//!     }
//!   ]
//! }
//! ```
//!
//! Defaults live in [`super::defaults`]; user entries override by key within a
//! context (`null` unbinds). See `docs/plans/PLAN-20260724-face-keybindings-remap.md`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::input::key::{parse_shortcut_str, KeyShortcut};
use xai_grok_pager_render::util::grok_home;

use super::{ActionDef, ActionId, When};

/// Relative path under [`grok_home()`].
pub const KEYBINDINGS_FILE_NAME: &str = "keybindings.json";

/// Absolute path to the user keybindings file.
pub fn keybindings_path() -> PathBuf {
    grok_home().join(KEYBINDINGS_FILE_NAME)
}

/// User-facing display path (e.g. `~/.next-code/keybindings.json`).
pub fn keybindings_display_path() -> String {
    xai_grok_pager_render::util::display_user_grok_path(KEYBINDINGS_FILE_NAME)
}

#[derive(Debug, Clone, Deserialize)]
struct KeybindingsFile {
    #[serde(default)]
    bindings: Vec<KeybindingBlock>,
}

#[derive(Debug, Clone, Deserialize)]
struct KeybindingBlock {
    context: String,
    #[serde(default)]
    bindings: BTreeMap<String, Value>,
}

/// A single parsed user override (after schema validation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedOverride {
    pub context: When,
    pub key: KeyShortcut,
    /// `None` means unbind this key in `context`.
    pub action: Option<ActionId>,
}

/// Load / parse / merge result.
#[derive(Debug, Clone, Default)]
pub struct LoadResult {
    pub overrides: Vec<ParsedOverride>,
    pub warnings: Vec<String>,
    /// True when a file existed on disk (even if empty / invalid).
    pub file_present: bool,
}

impl ActionId {
    /// Snake_case name used in `keybindings.json`.
    pub fn binding_name(self) -> &'static str {
        match self {
            ActionId::SendPrompt => "send_prompt",
            ActionId::InterjectPrompt => "interject_prompt",
            ActionId::EnableVoiceMode => "enable_voice_mode",
            ActionId::VoiceToggle => "voice_toggle",
            ActionId::ScrollUp => "scroll_up",
            ActionId::ScrollDown => "scroll_down",
            ActionId::PageUp => "page_up",
            ActionId::PageDown => "page_down",
            ActionId::HalfPageUp => "half_page_up",
            ActionId::HalfPageDown => "half_page_down",
            ActionId::GotoTop => "goto_top",
            ActionId::GotoBottom => "goto_bottom",
            ActionId::SelectNext => "select_next",
            ActionId::SelectPrev => "select_prev",
            ActionId::NextTurn => "next_turn",
            ActionId::PrevTurn => "prev_turn",
            ActionId::NextResponse => "next_response",
            ActionId::PrevResponse => "prev_response",
            ActionId::Collapse => "collapse",
            ActionId::Expand => "expand",
            ActionId::ToggleFold => "toggle_fold",
            ActionId::ToggleExpandAll => "toggle_expand_all",
            ActionId::ExpandAllThinking => "expand_all_thinking",
            ActionId::ToggleRaw => "toggle_raw",
            ActionId::ToggleMouseCapture => "toggle_mouse_capture",
            ActionId::NextModel => "next_model",
            ActionId::CancelTurn => "cancel_turn",
            ActionId::ToggleYolo => "toggle_yolo",
            ActionId::ToggleMultiline => "toggle_multiline",
            ActionId::FocusPrompt => "focus_prompt",
            ActionId::FocusScrollback => "focus_scrollback",
            ActionId::CopyBlockContent => "copy_block_content",
            ActionId::CopyBlockMeta => "copy_block_meta",
            ActionId::OpenBlockViewer => "open_block_viewer",
            ActionId::OpenNextLink => "open_next_link",
            ActionId::OpenPrevLink => "open_prev_link",
            ActionId::ToggleTodos => "toggle_todos",
            ActionId::ToggleTasks => "toggle_tasks",
            ActionId::ToggleQueue => "toggle_queue",
            ActionId::OpenSessions => "open_sessions",
            ActionId::OpenExtensions => "open_extensions",
            ActionId::SendToBackground => "send_to_background",
            ActionId::CycleMode => "cycle_mode",
            ActionId::BashMode => "bash_mode",
            ActionId::Rewind => "rewind",
            ActionId::KillBgTask => "kill_bg_task",
            ActionId::DumpInputLog => "dump_input_log",
            ActionId::Quit => "quit",
            ActionId::NewSession => "new_session",
            ActionId::NewSessionInWorktree => "new_session_in_worktree",
            ActionId::ExitSession => "exit_session",
            ActionId::CommandPalette => "command_palette",
            ActionId::ModelPicker => "model_picker",
            ActionId::ShortcutsHelp => "shortcuts_help",
            ActionId::OpenSettings => "open_settings",
            ActionId::OpenDashboard => "open_dashboard",
            ActionId::DashboardSelectNext => "dashboard_select_next",
            ActionId::DashboardSelectPrev => "dashboard_select_prev",
            ActionId::DashboardTogglePin => "dashboard_toggle_pin",
            ActionId::DashboardBeginRename => "dashboard_begin_rename",
            ActionId::DashboardStop => "dashboard_stop",
            ActionId::DashboardCycleMode => "dashboard_cycle_mode",
            ActionId::DashboardToggleGrouping => "dashboard_toggle_grouping",
            ActionId::DashboardReorderUp => "dashboard_reorder_up",
            ActionId::DashboardReorderDown => "dashboard_reorder_down",
            ActionId::DashboardShortcutsHelp => "dashboard_shortcuts_help",
            ActionId::DashboardExit => "dashboard_exit",
            ActionId::DashboardOverlayExit => "dashboard_overlay_exit",
            ActionId::DashboardOverlayPrev => "dashboard_overlay_prev",
            ActionId::DashboardOverlayNext => "dashboard_overlay_next",
            ActionId::DashboardOverlayStop => "dashboard_overlay_stop",
            ActionId::DashboardToggleAutoApprove => "dashboard_toggle_auto_approve",
            ActionId::DashboardOpenLocationPicker => "dashboard_open_location_picker",
            ActionId::DashboardToggleWorktree => "dashboard_toggle_worktree",
        }
    }

    /// Parse a snake_case binding name.
    pub fn from_binding_name(name: &str) -> Option<Self> {
        let n = name.trim();
        // Fast path: linear match keeps the table in one place with binding_name.
        const ALL: &[ActionId] = &[
            ActionId::SendPrompt,
            ActionId::InterjectPrompt,
            ActionId::EnableVoiceMode,
            ActionId::VoiceToggle,
            ActionId::ScrollUp,
            ActionId::ScrollDown,
            ActionId::PageUp,
            ActionId::PageDown,
            ActionId::HalfPageUp,
            ActionId::HalfPageDown,
            ActionId::GotoTop,
            ActionId::GotoBottom,
            ActionId::SelectNext,
            ActionId::SelectPrev,
            ActionId::NextTurn,
            ActionId::PrevTurn,
            ActionId::NextResponse,
            ActionId::PrevResponse,
            ActionId::Collapse,
            ActionId::Expand,
            ActionId::ToggleFold,
            ActionId::ToggleExpandAll,
            ActionId::ExpandAllThinking,
            ActionId::ToggleRaw,
            ActionId::ToggleMouseCapture,
            ActionId::NextModel,
            ActionId::CancelTurn,
            ActionId::ToggleYolo,
            ActionId::ToggleMultiline,
            ActionId::FocusPrompt,
            ActionId::FocusScrollback,
            ActionId::CopyBlockContent,
            ActionId::CopyBlockMeta,
            ActionId::OpenBlockViewer,
            ActionId::OpenNextLink,
            ActionId::OpenPrevLink,
            ActionId::ToggleTodos,
            ActionId::ToggleTasks,
            ActionId::ToggleQueue,
            ActionId::OpenSessions,
            ActionId::OpenExtensions,
            ActionId::SendToBackground,
            ActionId::CycleMode,
            ActionId::BashMode,
            ActionId::Rewind,
            ActionId::KillBgTask,
            ActionId::DumpInputLog,
            ActionId::Quit,
            ActionId::NewSession,
            ActionId::NewSessionInWorktree,
            ActionId::ExitSession,
            ActionId::CommandPalette,
            ActionId::ModelPicker,
            ActionId::ShortcutsHelp,
            ActionId::OpenSettings,
            ActionId::OpenDashboard,
            ActionId::DashboardSelectNext,
            ActionId::DashboardSelectPrev,
            ActionId::DashboardTogglePin,
            ActionId::DashboardBeginRename,
            ActionId::DashboardStop,
            ActionId::DashboardCycleMode,
            ActionId::DashboardToggleGrouping,
            ActionId::DashboardReorderUp,
            ActionId::DashboardReorderDown,
            ActionId::DashboardShortcutsHelp,
            ActionId::DashboardExit,
            ActionId::DashboardOverlayExit,
            ActionId::DashboardOverlayPrev,
            ActionId::DashboardOverlayNext,
            ActionId::DashboardOverlayStop,
            ActionId::DashboardToggleAutoApprove,
            ActionId::DashboardOpenLocationPicker,
            ActionId::DashboardToggleWorktree,
        ];
        ALL.iter()
            .copied()
            .find(|id| id.binding_name().eq_ignore_ascii_case(n))
    }
}

impl When {
    /// Context name used in `keybindings.json`.
    pub fn binding_name(self) -> &'static str {
        match self {
            When::Always => "Always",
            When::PromptFocused => "PromptFocused",
            When::ScrollbackFocused => "ScrollbackFocused",
            When::AgentScreen => "AgentScreen",
            When::WelcomeScreen => "WelcomeScreen",
            When::DashboardFocused => "DashboardFocused",
            When::DashboardOverlay => "DashboardOverlay",
        }
    }

    pub fn from_binding_name(name: &str) -> Option<Self> {
        match name.trim() {
            "Always" | "Global" => Some(When::Always),
            "PromptFocused" | "Chat" => Some(When::PromptFocused),
            "ScrollbackFocused" | "Transcript" => Some(When::ScrollbackFocused),
            "AgentScreen" => Some(When::AgentScreen),
            "WelcomeScreen" => Some(When::WelcomeScreen),
            "DashboardFocused" => Some(When::DashboardFocused),
            "DashboardOverlay" => Some(When::DashboardOverlay),
            _ => None,
        }
    }
}

/// Parse JSON text into overrides + warnings (does not touch disk).
pub fn parse_keybindings_json(text: &str) -> LoadResult {
    let mut result = LoadResult {
        file_present: true,
        ..LoadResult::default()
    };
    let parsed: KeybindingsFile = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            result
                .warnings
                .push(format!("invalid keybindings.json: {e}"));
            return result;
        }
    };

    for (block_idx, block) in parsed.bindings.iter().enumerate() {
        let Some(context) = When::from_binding_name(&block.context) else {
            result.warnings.push(format!(
                "bindings[{block_idx}]: unknown context {:?}",
                block.context
            ));
            continue;
        };
        for (key_raw, value) in &block.bindings {
            let Some(key) = parse_shortcut_str(key_raw) else {
                result.warnings.push(format!(
                    "bindings[{block_idx}].{}: invalid keystroke {:?}",
                    context.binding_name(),
                    key_raw
                ));
                continue;
            };
            let action = match value {
                Value::Null => None,
                Value::String(s) => {
                    if let Some(id) = ActionId::from_binding_name(s) {
                        Some(id)
                    } else {
                        result.warnings.push(format!(
                            "bindings[{block_idx}].{}: unknown action {:?}",
                            context.binding_name(),
                            s
                        ));
                        continue;
                    }
                }
                other => {
                    result.warnings.push(format!(
                        "bindings[{block_idx}].{}: expected action string or null, got {}",
                        context.binding_name(),
                        other
                    ));
                    continue;
                }
            };
            result.overrides.push(ParsedOverride {
                context,
                key,
                action,
            });
        }
    }
    result
}

/// Load from `path` if present. Missing file → empty overrides, no warning.
pub fn load_keybindings_file(path: &Path) -> LoadResult {
    match fs::read_to_string(path) {
        Ok(text) => parse_keybindings_json(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => LoadResult::default(),
        Err(e) => LoadResult {
            file_present: true,
            warnings: vec![format!("could not read {}: {e}", path.display())],
            ..LoadResult::default()
        },
    }
}

/// Load from the default user path.
pub fn load_user_keybindings() -> LoadResult {
    load_keybindings_file(&keybindings_path())
}

fn shortcut_eq(a: KeyShortcut, b: KeyShortcut) -> bool {
    a == b
}

fn strip_key_from_def(def: &mut ActionDef, key: KeyShortcut) {
    if shortcut_eq(def.default_key, key) {
        // Prefer promoting first alt; else Null (unbound primary).
        if let Some(next) = def.alt_keys.first().copied() {
            def.default_key = next;
            def.alt_keys.remove(0);
        } else {
            def.default_key = KeyShortcut::key(crossterm::event::KeyCode::Null);
        }
    }
    def.alt_keys.retain(|k| !shortcut_eq(*k, key));
}

fn strip_key_in_context(actions: &mut [ActionDef], context: When, key: KeyShortcut) {
    for def in actions.iter_mut() {
        if def.context == context {
            strip_key_from_def(def, key);
        }
    }
}

fn assign_key(def: &mut ActionDef, key: KeyShortcut) {
    if shortcut_eq(def.default_key, key) || def.alt_keys.iter().any(|k| shortcut_eq(*k, key)) {
        return;
    }
    let null = KeyShortcut::key(crossterm::event::KeyCode::Null);
    if shortcut_eq(def.default_key, null) {
        def.default_key = key;
    } else {
        // New binding becomes primary; keep previous primary as alt.
        def.alt_keys.insert(0, def.default_key);
        def.default_key = key;
    }
}

/// Apply parsed overrides onto a mutable action list (defaults first).
pub fn apply_overrides(actions: &mut [ActionDef], overrides: &[ParsedOverride]) {
    for ov in overrides {
        strip_key_in_context(actions, ov.context, ov.key);
        let Some(action_id) = ov.action else {
            continue;
        };
        if let Some(def) = actions
            .iter_mut()
            .find(|d| d.id == action_id && d.context == ov.context)
        {
            assign_key(def, ov.key);
        }
    }
}

/// Load user file (if any) and merge into `actions`. Returns warnings.
pub fn apply_user_keybindings(actions: &mut [ActionDef]) -> Vec<String> {
    let loaded = load_user_keybindings();
    apply_overrides(actions, &loaded.overrides);
    loaded.warnings
}

/// Minimal documented template (Claude-shaped). Full defaults stay in code.
pub fn generate_keybindings_template() -> String {
    let body = serde_json::json!({
        "$docs": "https://github.com/quangdang46/next-code — Face keybindings. Contexts: Always, PromptFocused, ScrollbackFocused, AgentScreen, WelcomeScreen, DashboardFocused, DashboardOverlay. Actions: snake_case ActionId (send_to_background, toggle_tasks, …). Set a key to null to unbind. Run /keybindings reload after editing if the editor did not reopen Face.",
        "bindings": [
            {
                "context": "AgentScreen",
                "bindings": {
                    "ctrl+g": "send_to_background",
                    "ctrl+b": "toggle_tasks",
                    "ctrl+t": "toggle_todos"
                }
            }
        ]
    });
    format!("{}\n", serde_json::to_string_pretty(&body).unwrap_or_else(|_| "{}".into()))
}

/// Create the template if missing (`wx`). Returns whether the file was created.
pub fn ensure_keybindings_file() -> std::io::Result<(PathBuf, bool)> {
    let path = keybindings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(generate_keybindings_template().as_bytes())?;
            Ok((path, true))
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok((path, false)),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::default_actions;
    use crate::key;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn action_binding_name_round_trip() {
        assert_eq!(
            ActionId::from_binding_name("send_to_background"),
            Some(ActionId::SendToBackground)
        );
        assert_eq!(
            ActionId::SendToBackground.binding_name(),
            "send_to_background"
        );
        assert_eq!(
            ActionId::from_binding_name("toggle_tasks"),
            Some(ActionId::ToggleTasks)
        );
    }

    #[test]
    fn when_binding_name_accepts_claude_aliases() {
        assert_eq!(When::from_binding_name("Global"), Some(When::Always));
        assert_eq!(
            When::from_binding_name("Chat"),
            Some(When::PromptFocused)
        );
    }

    #[test]
    fn parse_and_merge_override_changes_lookup() {
        let json = r#"{
          "bindings": [
            {
              "context": "AgentScreen",
              "bindings": {
                "ctrl+h": "send_to_background",
                "ctrl+g": null
              }
            }
          ]
        }"#;
        let loaded = parse_keybindings_json(json);
        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        assert_eq!(loaded.overrides.len(), 2);

        let mut actions = default_actions(false);
        apply_overrides(&mut actions, &loaded.overrides);
        let reg = crate::actions::ActionRegistry::new(actions);

        let ctrl_h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL);
        let ctrl_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert_eq!(
            reg.lookup(&ctrl_h, When::AgentScreen),
            Some(ActionId::SendToBackground)
        );
        assert_eq!(reg.lookup(&ctrl_g, When::AgentScreen), None);
        assert!(reg.matches_id(ActionId::SendToBackground, &ctrl_h));
        assert!(!reg.matches_id(ActionId::SendToBackground, &ctrl_g));
    }

    #[test]
    fn null_unbind_only_strips_key() {
        let json = r#"{
          "bindings": [{
            "context": "AgentScreen",
            "bindings": { "ctrl+b": null }
          }]
        }"#;
        let loaded = parse_keybindings_json(json);
        let mut actions = default_actions(false);
        apply_overrides(&mut actions, &loaded.overrides);
        let reg = crate::actions::ActionRegistry::new(actions);
        let ctrl_b = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert_eq!(reg.lookup(&ctrl_b, When::AgentScreen), None);
    }

    #[test]
    fn invalid_json_yields_warning_not_panic() {
        let loaded = parse_keybindings_json("{ not json");
        assert!(loaded.overrides.is_empty());
        assert!(!loaded.warnings.is_empty());
    }

    #[test]
    fn parse_shortcut_ctrl_g() {
        assert_eq!(parse_shortcut_str("ctrl+g"), Some(key!('g', CONTROL)));
        assert_eq!(parse_shortcut_str("shift+tab"), Some(key!(Tab, SHIFT)));
    }

    #[test]
    fn template_is_valid_json() {
        let t = generate_keybindings_template();
        let loaded = parse_keybindings_json(&t);
        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        assert!(!loaded.overrides.is_empty());
    }
}
