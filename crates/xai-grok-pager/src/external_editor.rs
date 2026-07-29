//! Resolve and spawn `$VISUAL` / `$EDITOR` the way Claude Code does.
//!
//! Face previously defaulted to bare `vi` and ran `Command::new(editor)` with
//! no shell / wait flags. On Windows that leaves the alternate screen (black
//! main buffer) and then fails to start an editor — the reported `/memory`
//! edit black-screen.
//!
//! References:
//! - `.tmp/claude-code/src/utils/editor.ts` (`getExternalEditor`, GUI list,
//!   win32 `start /wait notepad`)
//! - `.tmp/claude-code/src/utils/promptEditor.ts` (`EDITOR_OVERRIDES`, skip
//!   alt-screen handoff for GUI editors)

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

/// GUI editors that open a separate window and must not kick Face off the
/// alternate screen (Claude `classifyGuiEditor` / `GUI_EDITORS`).
const GUI_EDITORS: &[&str] = &[
    "code",
    "cursor",
    "windsurf",
    "codium",
    "subl",
    "atom",
    "gedit",
    "notepad++",
    "notepad",
];

/// Resolve the editor command string (may contain spaces / flags).
pub fn resolve_editor() -> String {
    if let Ok(v) = std::env::var("VISUAL") {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    if let Ok(v) = std::env::var("EDITOR") {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    #[cfg(windows)]
    {
        // Claude: `start /wait notepad` — Face uses cmd /C so CreateProcess
        // does not need to resolve the `start` builtin itself.
        return "notepad".to_string();
    }
    #[cfg(not(windows))]
    {
        "vi".to_string()
    }
}

/// Basename of the first token (handles absolute paths and `code -w`).
fn editor_basename(editor: &str) -> String {
    let first = editor.split_whitespace().next().unwrap_or(editor);
    Path::new(first)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(first)
        .to_ascii_lowercase()
}

/// Whether this editor is a separate-window GUI (do not LeaveAlternateScreen).
pub fn is_gui_editor(editor: &str) -> bool {
    let base = editor_basename(editor);
    GUI_EDITORS.iter().any(|g| base.contains(g))
}

/// Apply Claude-style wait overrides so GUI editors block until the tab closes.
fn with_wait_flags(editor: &str) -> String {
    let base = editor_basename(editor);
    let lower = editor.to_ascii_lowercase();
    if base.contains("code")
        || base.contains("cursor")
        || base.contains("windsurf")
        || base.contains("codium")
    {
        if lower.split_whitespace().any(|t| t == "-w" || t == "--wait") {
            return editor.to_string();
        }
        return format!("{editor} -w");
    }
    if base.contains("subl") {
        if lower
            .split_whitespace()
            .any(|t| t == "--wait" || t == "-w")
        {
            return editor.to_string();
        }
        return format!("{editor} --wait");
    }
    editor.to_string()
}

/// Quote a path for a Windows `cmd /C` command line.
fn win_quote(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.contains(' ') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.into_owned()
    }
}

/// Spawn the resolved editor and wait for it to exit.
pub fn spawn_editor_blocking(path: &Path) -> std::io::Result<ExitStatus> {
    let editor = with_wait_flags(&resolve_editor());
    tracing::debug!(%editor, path = %path.display(), "spawning external editor");

    #[cfg(windows)]
    {
        // `notepad` alone → `start /WAIT notepad <path>` so we wait for the
        // GUI window (matching Claude's default). Multi-word $EDITOR values
        // go through `cmd /C` so `.cmd` shims (`code.cmd`) resolve.
        let base = editor_basename(&editor);
        let cmdline = if base == "notepad" || base == "notepad.exe" {
            format!("start /WAIT notepad {}", win_quote(path))
        } else {
            format!("{} {}", editor, win_quote(path))
        };
        Command::new("cmd").args(["/C", &cmdline]).status()
    }

    #[cfg(not(windows))]
    {
        let mut parts = editor.split_whitespace();
        let prog = parts.next().unwrap_or("vi");
        let mut cmd = Command::new(prog);
        cmd.args(parts).arg(path);
        cmd.status()
    }
}

/// Convenience for tests / callers that need the path as owned.
#[cfg(test)]
pub fn resolve_editor_for_path(_path: &PathBuf) -> String {
    resolve_editor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gui_detection_matches_claude_list() {
        assert!(is_gui_editor("code"));
        assert!(is_gui_editor("code -w"));
        assert!(is_gui_editor("/usr/local/bin/cursor"));
        assert!(is_gui_editor("notepad"));
        assert!(is_gui_editor("subl --wait"));
        assert!(!is_gui_editor("vi"));
        assert!(!is_gui_editor("nvim"));
        assert!(!is_gui_editor("nano"));
    }

    #[test]
    fn wait_flags_added_for_vscode_family() {
        assert_eq!(with_wait_flags("code"), "code -w");
        assert_eq!(with_wait_flags("code -w"), "code -w");
        assert_eq!(with_wait_flags("cursor"), "cursor -w");
        assert_eq!(with_wait_flags("subl"), "subl --wait");
        assert_eq!(with_wait_flags("vi"), "vi");
    }

    #[test]
    fn resolve_prefers_visual_then_editor() {
        // Isolation: only assert the preference order when vars are set by us.
        // SAFETY: test process; we restore afterward.
        let old_visual = std::env::var_os("VISUAL");
        let old_editor = std::env::var_os("EDITOR");
        unsafe {
            std::env::remove_var("VISUAL");
            std::env::set_var("EDITOR", "nano");
        }
        assert_eq!(resolve_editor(), "nano");
        unsafe {
            std::env::set_var("VISUAL", "nvim");
        }
        assert_eq!(resolve_editor(), "nvim");
        unsafe {
            match old_visual {
                Some(v) => std::env::set_var("VISUAL", v),
                None => std::env::remove_var("VISUAL"),
            }
            match old_editor {
                Some(v) => std::env::set_var("EDITOR", v),
                None => std::env::remove_var("EDITOR"),
            }
        }
    }
}
