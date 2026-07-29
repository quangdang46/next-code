//! Build ACP `ToolCallContent::Diff` for Face edit/write/hashline tools.
//!
//! Mirrors grok-build `acp_conversion` for `SearchReplaceOutput::EditsApplied`:
//! Face `extract_edit_hunks` prefers Diff content (strategy 2/3) over Bash-shaped
//! raw_output.

use agent_client_protocol as acp;
use serde_json::Value;

/// Max bytes of old+new text embedded in a Face Diff (full-file for hashline).
const MAX_DIFF_BYTES: usize = 512_000;

/// Whether this tool name should render as Face `ToolKind::Edit`.
pub fn is_face_edit_tool(name: &str) -> bool {
    next_code_tui_tool_display::is_edit_tool_name(name)
}

/// Prefer structured `metadata` from the tool (`old_text` / `new_text` /
/// `file_path`), else reconstruct from `raw_input` (classic edit/write args).
pub fn build_edit_diff_content(
    name: &str,
    raw_input: Option<&Value>,
    metadata: Option<&Value>,
) -> Option<acp::ToolCallContent> {
    if !is_face_edit_tool(name) {
        return None;
    }

    if let Some(diff) = diff_from_metadata(metadata) {
        return Some(diff);
    }
    diff_from_raw_input(raw_input)
}

fn path_from_value(v: &Value) -> Option<&str> {
    v.get("file_path")
        .or_else(|| v.get("filePath"))
        .or_else(|| v.get("target_file"))
        .or_else(|| v.get("path"))
        .and_then(|p| p.as_str())
        .map(str::trim)
        .filter(|p| !p.is_empty())
}

fn diff_from_metadata(metadata: Option<&Value>) -> Option<acp::ToolCallContent> {
    let meta = metadata?;
    let path = path_from_value(meta)?;
    let new_text = meta.get("new_text").and_then(|v| v.as_str())?;
    let old_text = meta
        .get("old_text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if old_text.len().saturating_add(new_text.len()) > MAX_DIFF_BYTES {
        return None;
    }
    Some(acp::ToolCallContent::Diff(
        acp::Diff::new(path, new_text.to_string()).old_text(Some(old_text.to_string())),
    ))
}

fn diff_from_raw_input(raw_input: Option<&Value>) -> Option<acp::ToolCallContent> {
    let input = raw_input?;
    let path = path_from_value(input)?;

    // Classic str-replace / structured hashline edit.
    if let (Some(old), Some(new)) = (
        input.get("old_string").and_then(|v| v.as_str()),
        input.get("new_string").and_then(|v| v.as_str()),
    ) {
        if old.len().saturating_add(new.len()) > MAX_DIFF_BYTES {
            return None;
        }
        return Some(acp::ToolCallContent::Diff(
            acp::Diff::new(path, new.to_string()).old_text(Some(old.to_string())),
        ));
    }

    // Write / create: new file body only (old empty → all inserts).
    if let Some(content) = input.get("content").and_then(|v| v.as_str()) {
        if content.len() > MAX_DIFF_BYTES {
            return None;
        }
        return Some(acp::ToolCallContent::Diff(
            acp::Diff::new(path, content.to_string()).old_text(Some(String::new())),
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classic_edit_from_raw_input() {
        let input = json!({
            "file_path": "src/main.rs",
            "old_string": "let x = 1;",
            "new_string": "let x = 2;",
        });
        let content = build_edit_diff_content("edit", Some(&input), None).expect("diff");
        match content {
            acp::ToolCallContent::Diff(d) => {
                assert_eq!(d.path, std::path::PathBuf::from("src/main.rs"));
                assert_eq!(d.new_text, "let x = 2;");
                assert_eq!(d.old_text.as_deref(), Some("let x = 1;"));
            }
            other => panic!("expected Diff, got {other:?}"),
        }
    }

    #[test]
    fn hashline_from_metadata() {
        let meta = json!({
            "file_path": "a.rs",
            "old_text": "a\n",
            "new_text": "b\n",
        });
        let content =
            build_edit_diff_content("hashline_edit", None, Some(&meta)).expect("diff");
        assert!(matches!(content, acp::ToolCallContent::Diff(_)));
    }

    #[test]
    fn write_from_content() {
        let input = json!({
            "file_path": "new.txt",
            "content": "hello\n",
        });
        let content = build_edit_diff_content("write", Some(&input), None).expect("diff");
        match content {
            acp::ToolCallContent::Diff(d) => {
                assert_eq!(d.old_text.as_deref(), Some(""));
                assert_eq!(d.new_text, "hello\n");
            }
            other => panic!("expected Diff, got {other:?}"),
        }
    }

    #[test]
    fn non_edit_returns_none() {
        assert!(build_edit_diff_content("bash", Some(&json!({})), None).is_none());
    }
}
