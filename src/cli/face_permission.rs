//! Face permission confirm bridge: daemon `ServerEvent::PermissionRequest`
//! → ACP `Client::request_permission` → daemon `Request::PermissionResponse`.
//!
//! Builds Claude Code–style **tool-specific** card payloads (bash command /
//! cwd risk cues, edit/write path + diff summary) so Face `permission_view`
//! can render richer chrome than a generic "Approve".
//!
//! Distinct from AskUserQuestion (`question_view` / `x.ai/ask_user_question`).

use std::sync::Arc;

use agent_client_protocol as acp;
use agent_client_protocol::Client as _;
use anyhow::{Context, Result};
use xai_acp_lib::AcpGatewaySender;

use crate::protocol::{Request, ServerEvent};

use super::pager_agent::DaemonSession;

pub(crate) const OUTCOME_ALLOW_ONCE: &str = "allow-once";
pub(crate) const OUTCOME_ALLOW_ALWAYS: &str = "allow-always";
pub(crate) const OUTCOME_ALLOW_ALL: &str = "allow-all";
pub(crate) const OUTCOME_REJECT_ONCE: &str = "reject-once";
pub(crate) const OUTCOME_CANCELLED: &str = "cancelled";

/// Tool family for permission card chrome (Claude-style dispatch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PermissionCardKind {
    Bash,
    Edit,
    Write,
    Other,
}

/// Map an ACP permission response to a daemon outcome id.
pub(crate) fn outcome_from_acp_response(resp: &acp::RequestPermissionResponse) -> String {
    match &resp.outcome {
        acp::RequestPermissionOutcome::Cancelled => OUTCOME_CANCELLED.to_string(),
        acp::RequestPermissionOutcome::Selected(selected) => {
            selected.option_id.0.as_ref().to_string()
        }
        _ => OUTCOME_CANCELLED.to_string(),
    }
}

pub(crate) fn classify_permission_card(tool_name: &str) -> PermissionCardKind {
    match tool_name {
        "bash" | "shell" | "powershell" => PermissionCardKind::Bash,
        "edit" | "multiedit" | "str_replace" | "strreplace" | "apply_patch" => {
            PermissionCardKind::Edit
        }
        "write" | "propose_write" | "create_file" => PermissionCardKind::Write,
        other if other.ends_with("__bash") || other.contains("Bash") => PermissionCardKind::Bash,
        other if other.contains("edit") || other.contains("Edit") => PermissionCardKind::Edit,
        other if other.contains("write") || other.contains("Write") => PermissionCardKind::Write,
        _ => PermissionCardKind::Other,
    }
}

fn tool_kind_for_card(kind: PermissionCardKind) -> acp::ToolKind {
    match kind {
        PermissionCardKind::Bash => acp::ToolKind::Execute,
        PermissionCardKind::Edit | PermissionCardKind::Write => acp::ToolKind::Edit,
        PermissionCardKind::Other => acp::ToolKind::Other,
    }
}

fn permission_options(kind: PermissionCardKind) -> Vec<acp::PermissionOption> {
    let always_label = match kind {
        PermissionCardKind::Bash => "Always allow this command",
        PermissionCardKind::Edit | PermissionCardKind::Write => "Always allow edits",
        PermissionCardKind::Other => "Always allow this tool",
    };
    let allow_all_label = match kind {
        PermissionCardKind::Edit | PermissionCardKind::Write => "Allow all edits this session",
        _ => "Allow all tools this session",
    };
    vec![
        acp::PermissionOption::new(
            acp::PermissionOptionId::new(Arc::from(OUTCOME_ALLOW_ONCE)),
            "Allow once".to_string(),
            acp::PermissionOptionKind::AllowOnce,
        ),
        acp::PermissionOption::new(
            acp::PermissionOptionId::new(Arc::from(OUTCOME_ALLOW_ALWAYS)),
            always_label.to_string(),
            acp::PermissionOptionKind::AllowAlways,
        ),
        acp::PermissionOption::new(
            acp::PermissionOptionId::new(Arc::from(OUTCOME_ALLOW_ALL)),
            allow_all_label.to_string(),
            acp::PermissionOptionKind::AllowAlways,
        ),
        acp::PermissionOption::new(
            acp::PermissionOptionId::new(Arc::from(OUTCOME_REJECT_ONCE)),
            "Reject".to_string(),
            acp::PermissionOptionKind::RejectOnce,
        ),
    ]
}

fn extract_command(tool_input: Option<&serde_json::Value>, reason: &str) -> Option<String> {
    if let Some(v) = tool_input {
        if let Some(cmd) = v.get("command").and_then(|c| c.as_str()) {
            let trimmed = cmd.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    // Fallback: reason often embeds "… (command: …)" from bash.rs
    if let Some(idx) = reason.rfind("(command: ") {
        let rest = &reason[idx + "(command: ".len()..];
        let end = rest.rfind(')').unwrap_or(rest.len());
        let cmd = rest[..end].trim();
        if !cmd.is_empty() {
            return Some(cmd.to_string());
        }
    }
    None
}

fn extract_file_path(tool_input: Option<&serde_json::Value>) -> Option<&str> {
    tool_input
        .and_then(|v| v.get("file_path").or_else(|| v.get("path")))
        .and_then(|p| p.as_str())
        .map(str::trim)
        .filter(|p| !p.is_empty())
}

fn card_title(
    kind: PermissionCardKind,
    tool_name: &str,
    tool_input: Option<&serde_json::Value>,
    reason: &str,
) -> String {
    match kind {
        PermissionCardKind::Bash => {
            let desc = tool_input
                .and_then(|v| v.get("description"))
                .and_then(|d| d.as_str())
                .map(str::trim)
                .filter(|d| !d.is_empty());
            if let Some(d) = desc {
                return d.to_string();
            }
            if let Some(cmd) = extract_command(tool_input, reason) {
                let bin = cmd.split_whitespace().next().unwrap_or("command");
                return format!("Allow `{bin}`?");
            }
            "Allow Execute?".to_string()
        }
        PermissionCardKind::Edit => {
            if let Some(path) = extract_file_path(tool_input) {
                format!("Allow Edit to {path}?")
            } else {
                "Allow Edit?".to_string()
            }
        }
        PermissionCardKind::Write => {
            if let Some(path) = extract_file_path(tool_input) {
                format!("Allow Write to {path}?")
            } else {
                "Allow Write?".to_string()
            }
        }
        PermissionCardKind::Other => format!("Allow {tool_name}?"),
    }
}

/// Normalize daemon tool_input into shapes Face `build_permission_display` expects.
fn normalize_raw_input(
    kind: PermissionCardKind,
    tool_name: &str,
    reason: &str,
    tool_input: Option<serde_json::Value>,
) -> serde_json::Value {
    match kind {
        PermissionCardKind::Bash => {
            let mut obj = match tool_input {
                Some(serde_json::Value::Object(map)) => map,
                Some(other) => {
                    let mut m = serde_json::Map::new();
                    m.insert("value".to_string(), other);
                    m
                }
                None => serde_json::Map::new(),
            };
            if !obj.contains_key("command") {
                if let Some(cmd) = extract_command(None, reason) {
                    obj.insert("command".to_string(), serde_json::Value::String(cmd));
                }
            }
            if !obj.contains_key("description") {
                obj.insert(
                    "description".to_string(),
                    serde_json::Value::String(String::new()),
                );
            }
            // Surface reason as a risk/context cue when present.
            if !reason.is_empty() && !obj.contains_key("permission_reason") {
                obj.insert(
                    "permission_reason".to_string(),
                    serde_json::Value::String(reason.to_string()),
                );
            }
            serde_json::Value::Object(obj)
        }
        PermissionCardKind::Edit | PermissionCardKind::Write => {
            let mut obj = match tool_input {
                Some(serde_json::Value::Object(map)) => map,
                Some(other) => {
                    let mut m = serde_json::Map::new();
                    m.insert("value".to_string(), other);
                    m
                }
                None => serde_json::Map::new(),
            };
            if !reason.is_empty() && !obj.contains_key("permission_reason") {
                obj.insert(
                    "permission_reason".to_string(),
                    serde_json::Value::String(reason.to_string()),
                );
            }
            serde_json::Value::Object(obj)
        }
        PermissionCardKind::Other => {
            if let Some(raw) = tool_input {
                raw
            } else if !reason.is_empty() {
                serde_json::json!({
                    "reason": reason,
                    "tool_name": tool_name,
                })
            } else {
                serde_json::json!({ "tool_name": tool_name })
            }
        }
    }
}

/// Lightweight bash word highlights for Face scope selection (no tree-sitter).
fn bash_highlights_meta(command: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let words: Vec<String> = command
        .split_whitespace()
        .take(8)
        .map(|w| w.to_string())
        .collect();
    if words.is_empty() {
        return None;
    }
    let meta = serde_json::json!({
        "prefix": [],
        "highlighted_words": words,
        "suffix": [],
    });
    meta.as_object().cloned()
}

fn build_request(
    session_id: &str,
    tool_name: &str,
    reason: &str,
    tool_call_id: &str,
    tool_input: Option<serde_json::Value>,
) -> acp::RequestPermissionRequest {
    let kind = classify_permission_card(tool_name);
    let call_id = if tool_call_id.is_empty() {
        format!("perm-{tool_name}")
    } else {
        tool_call_id.to_string()
    };
    let title = card_title(kind, tool_name, tool_input.as_ref(), reason);
    let raw = normalize_raw_input(kind, tool_name, reason, tool_input);
    let fields = acp::ToolCallUpdateFields::new()
        .title(title)
        .kind(tool_kind_for_card(kind))
        .raw_input(raw.clone());

    let mut req = acp::RequestPermissionRequest::new(
        acp::SessionId::new(Arc::from(session_id)),
        acp::ToolCallUpdate::new(acp::ToolCallId::new(Arc::from(call_id.as_str())), fields),
        permission_options(kind),
    );

    if kind == PermissionCardKind::Bash
        && let Some(cmd) = raw.get("command").and_then(|c| c.as_str())
        && let Some(meta) = bash_highlights_meta(cmd)
    {
        req = req.meta(Some(meta));
    }

    req
}

pub(crate) async fn bridge_permission_request(
    gateway: &AcpGatewaySender<acp::AgentSide>,
    session: &DaemonSession,
    event: ServerEvent,
) -> Result<()> {
    let ServerEvent::PermissionRequest {
        request_id,
        session_id,
        tool_name,
        reason,
        allow_once_code,
        tool_input,
        tool_call_id,
        ..
    } = event
    else {
        anyhow::bail!("bridge_permission_request expected PermissionRequest event");
    };

    let args = build_request(
        &session_id,
        &tool_name,
        &reason,
        &tool_call_id,
        tool_input,
    );
    let reply_id = session.next_id();
    let outcome = match gateway.request_permission(args).await {
        Ok(resp) => outcome_from_acp_response(&resp),
        Err(err) => {
            crate::logging::warn(&format!(
                "Face request_permission failed; denying: {err}"
            ));
            OUTCOME_CANCELLED.to_string()
        }
    };

    session
        .send(&Request::PermissionResponse {
            id: reply_id,
            request_id,
            outcome,
            session_id,
            tool_name,
            allow_once_code,
        })
        .await
        .context("send PermissionResponse")?;
    wait_done(session, reply_id).await
}

async fn wait_done(session: &DaemonSession, request_id: u64) -> Result<()> {
    loop {
        match session.read_event().await? {
            ServerEvent::Done { id } if id == request_id => return Ok(()),
            ServerEvent::Error { id, message, .. } if id == request_id => {
                anyhow::bail!("permission_response failed: {message}");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_maps_cancelled() {
        let resp = acp::RequestPermissionResponse::new(acp::RequestPermissionOutcome::Cancelled);
        assert_eq!(outcome_from_acp_response(&resp), OUTCOME_CANCELLED);
    }

    #[test]
    fn outcome_maps_selected_option_id() {
        let resp = acp::RequestPermissionResponse::new(acp::RequestPermissionOutcome::Selected(
            acp::SelectedPermissionOutcome::new(acp::PermissionOptionId::new(Arc::from(
                OUTCOME_ALLOW_ONCE,
            ))),
        ));
        assert_eq!(outcome_from_acp_response(&resp), OUTCOME_ALLOW_ONCE);
    }

    #[test]
    fn build_request_includes_four_options() {
        let req = build_request("sess", "bash", "needs approval", "tc-1", None);
        assert_eq!(req.options.len(), 4);
        assert_eq!(req.session_id.0.as_ref(), "sess");
        let ids: Vec<_> = req.options.iter().map(|o| o.option_id.0.as_ref()).collect();
        assert_eq!(
            ids,
            [
                OUTCOME_ALLOW_ONCE,
                OUTCOME_ALLOW_ALWAYS,
                OUTCOME_ALLOW_ALL,
                OUTCOME_REJECT_ONCE
            ]
        );
    }

    #[test]
    fn classify_bash_edit_write() {
        assert_eq!(classify_permission_card("bash"), PermissionCardKind::Bash);
        assert_eq!(classify_permission_card("edit"), PermissionCardKind::Edit);
        assert_eq!(classify_permission_card("write"), PermissionCardKind::Write);
        assert_eq!(classify_permission_card("read"), PermissionCardKind::Other);
    }

    #[test]
    fn bash_card_sets_execute_kind_and_command() {
        let req = build_request(
            "sess",
            "bash",
            "risky",
            "tc-1",
            Some(serde_json::json!({
                "command": "rm -rf /tmp/x",
                "description": "Clean temp"
            })),
        );
        assert_eq!(req.tool_call.fields.kind, Some(acp::ToolKind::Execute));
        assert_eq!(
            req.tool_call.fields.title.as_deref(),
            Some("Clean temp")
        );
        let cmd = req
            .tool_call
            .fields
            .raw_input
            .as_ref()
            .and_then(|v| v.get("command"))
            .and_then(|c| c.as_str());
        assert_eq!(cmd, Some("rm -rf /tmp/x"));
        assert!(req.meta.is_some(), "bash highlights meta attached");
        let always = req
            .options
            .iter()
            .find(|o| o.option_id.0.as_ref() == OUTCOME_ALLOW_ALWAYS)
            .unwrap();
        assert!(always.name.to_lowercase().contains("command"));
    }

    #[test]
    fn bash_card_recovers_command_from_reason() {
        let req = build_request(
            "sess",
            "bash",
            "needs approval (command: git status)",
            "tc-1",
            None,
        );
        let cmd = req
            .tool_call
            .fields
            .raw_input
            .as_ref()
            .and_then(|v| v.get("command"))
            .and_then(|c| c.as_str());
        assert_eq!(cmd, Some("git status"));
        assert!(
            req.tool_call
                .fields
                .title
                .as_deref()
                .unwrap_or("")
                .contains("git")
        );
    }

    #[test]
    fn edit_card_sets_edit_kind_path_title_and_edit_options() {
        let req = build_request(
            "sess",
            "edit",
            "write gate",
            "tc-2",
            Some(serde_json::json!({
                "file_path": "src/main.rs",
                "old_string": "fn a() {}",
                "new_string": "fn a() { todo!() }"
            })),
        );
        assert_eq!(req.tool_call.fields.kind, Some(acp::ToolKind::Edit));
        assert_eq!(
            req.tool_call.fields.title.as_deref(),
            Some("Allow Edit to src/main.rs?")
        );
        let always = req
            .options
            .iter()
            .find(|o| o.option_id.0.as_ref() == OUTCOME_ALLOW_ALWAYS)
            .unwrap();
        assert!(
            always.name.to_lowercase().contains("edit"),
            "Face is_edit_permission keys off AllowAlways name containing 'edit'"
        );
    }

    #[test]
    fn write_card_title_includes_path() {
        let req = build_request(
            "sess",
            "write",
            "",
            "tc-3",
            Some(serde_json::json!({
                "file_path": "README.md",
                "content": "# hi\n"
            })),
        );
        assert_eq!(
            req.tool_call.fields.title.as_deref(),
            Some("Allow Write to README.md?")
        );
        assert_eq!(req.tool_call.fields.kind, Some(acp::ToolKind::Edit));
    }
}
