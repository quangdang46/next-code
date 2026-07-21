//! Thin Face ACP `x.ai/*` handlers wired to next-code brain (PR10 gap close).
//!
//! Face chrome keeps calling stock method names; we implement the brain here
//! instead of returning empty `{}`.

use serde_json::json;

use crate::session::Session;

const FEEDBACK_ISSUE_URL: &str = "https://github.com/quangdang46/next-code/issues/new";

/// Format connected-provider usage/cost for Face `/usage` / `/cost`.
pub async fn usage_text_payload() -> serde_json::Value {
    let providers = crate::usage::fetch_all_provider_usage().await;
    let text = format_usage_text(&providers);
    json!({ "result": { "text": text } })
}

fn format_usage_text(providers: &[crate::usage::ProviderUsage]) -> String {
    if providers.is_empty() {
        return "No connected providers\n\nNext steps:\n- Use /connect <provider> to add credentials\n- Then /usage show again"
            .to_string();
    }
    let mut out = String::from("Usage / cost (connected providers)\n");
    for (idx, provider) in providers.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(&provider.provider_name);
        out.push('\n');
        out.push_str(&"-".repeat(provider.provider_name.chars().count().max(3)));
        out.push('\n');
        if let Some(error) = &provider.error {
            out.push_str(&format!("error: {error}\n"));
            continue;
        }
        if provider.limits.is_empty() && provider.extra_info.is_empty() {
            out.push_str("No usage data available.\n");
            continue;
        }
        for limit in &provider.limits {
            match limit
                .resets_at
                .as_deref()
                .map(crate::usage::format_reset_time)
            {
                Some(reset_in) => out.push_str(&format!(
                    "{}: {} (resets in {})\n",
                    limit.name,
                    crate::usage::format_usage_bar(limit.usage_percent, 15),
                    reset_in
                )),
                None => out.push_str(&format!(
                    "{}: {}\n",
                    limit.name,
                    crate::usage::format_usage_bar(limit.usage_percent, 15)
                )),
            }
        }
        for (key, value) in &provider.extra_info {
            out.push_str(&format!("{key}: {value}\n"));
        }
    }
    out
}

/// `x.ai/session/info` — context + session snapshot for `/context` / session info.
pub fn session_info_payload(session_id: &str) -> serde_json::Value {
    let Ok(session) = Session::load(session_id) else {
        return json!({
            "error": format!("Unknown session id: {session_id}")
        });
    };
    let totals = session.token_usage_totals();
    let used = totals.input_tokens.saturating_add(totals.output_tokens);
    let total = used.max(1);
    let usage_pct = ((used as f64 / total as f64) * 100.0).round() as u8;
    let turn_count = session
        .messages
        .iter()
        .filter(|m| matches!(m.role, crate::message::Role::User))
        .count() as u64;
    let cwd = session.working_dir.clone().unwrap_or_default();
    let model = session.model.clone().unwrap_or_else(|| "unknown".into());
    json!({
        "result": {
            "sessionId": session.id,
            "cwd": cwd,
            "data": {
                "model": model,
                "modelDisplayName": session.display_title_or_name(),
                "turns": turn_count,
                "turnIndex": turn_count,
                "context": {
                    "used": used,
                    "total": total,
                    "freeTokens": 0,
                    "messageTokens": used,
                    "systemPromptTokens": 0,
                    "turnCount": turn_count,
                    "toolCallCount": 0,
                    "compactionCount": 0,
                    "usagePct": usage_pct,
                    "messageCount": session.messages.len() as u64,
                    "autoCompactThresholdPercent": 85,
                }
            }
        }
    })
}

/// `x.ai/session/rename`
pub fn session_rename_payload(session_id: &str, title: &str) -> serde_json::Value {
    let Ok(mut session) = Session::load(session_id) else {
        return json!({ "error": format!("Unknown session id: {session_id}") });
    };
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return json!({ "error": "Title must not be empty" });
    }
    session.rename_title(Some(trimmed.to_string()));
    if let Err(e) = session.save() {
        return json!({ "error": format!("Failed to save session: {e}") });
    }
    crate::tui::session_picker::invalidate_session_list_cache();
    json!({ "result": { "ok": true, "title": trimmed } })
}

/// `x.ai/feedback` — record locally; point users at GitHub (not x.ai/feedback).
pub fn feedback_payload(feedback_text: Option<&str>) -> serde_json::Value {
    let text = feedback_text.unwrap_or("").trim();
    if !text.is_empty() {
        // Persist is intentionally fire-and-forget for Face; logs via stderr
        // so operators can collect feedback without xAI telemetry.
        eprintln!("[nextcode.feedback] {text}");
    }
    json!({
        "result": {
            "ok": true,
            "issueUrl": FEEDBACK_ISSUE_URL,
            "message": format!(
                "Thanks — feedback recorded. Open an issue if you want follow-up: {FEEDBACK_ISSUE_URL}"
            ),
        }
    })
}

/// `x.ai/btw` — side question acknowledgment (Face chrome; next-code brain note).
pub fn btw_payload(question: &str) -> serde_json::Value {
    let q = question.trim();
    let answer = if q.is_empty() {
        "Usage: /btw <question>".to_string()
    } else {
        format!(
            "Side question noted: {q}\n\n\
             next-code Face will answer side questions from session context on a later turn; \
             for an immediate reply, ask in the main composer."
        )
    };
    json!({ "result": { "answer": answer } })
}

/// `x.ai/rewind/points` — user turns as rewind targets.
pub fn rewind_points_payload(session_id: &str) -> serde_json::Value {
    let Ok(session) = Session::load(session_id) else {
        return json!({ "error": format!("Unknown session id: {session_id}") });
    };
    let mut rewind_points = Vec::new();
    let mut prompt_index = 0usize;
    for msg in &session.messages {
        if !matches!(msg.role, crate::message::Role::User) {
            continue;
        }
        let preview = msg
            .content
            .iter()
            .find_map(|b| match b {
                crate::message::ContentBlock::Text { text, .. } => {
                    let t = text.trim();
                    if t.is_empty() || t.starts_with("<system-reminder>") {
                        None
                    } else {
                        Some(t.chars().take(120).collect::<String>())
                    }
                }
                _ => None,
            });
        if preview.is_none() {
            continue;
        }
        rewind_points.push(json!({
            "promptIndex": prompt_index,
            "createdAt": session.updated_at.to_rfc3339(),
            "numFileSnapshots": 0,
            "promptPreview": preview,
            "hasFileChanges": false,
        }));
        prompt_index += 1;
    }
    json!({ "result": { "rewindPoints": rewind_points } })
}

/// `x.ai/rewind/execute` — truncate transcript after the selected user turn.
pub fn rewind_execute_payload(
    session_id: &str,
    target_prompt_index: usize,
    force: bool,
) -> serde_json::Value {
    let Ok(mut session) = Session::load(session_id) else {
        return json!({ "error": format!("Unknown session id: {session_id}") });
    };
    let mut user_idx = 0usize;
    let mut cut_at = None;
    for (i, msg) in session.messages.iter().enumerate() {
        if !matches!(msg.role, crate::message::Role::User) {
            continue;
        }
        let is_visible = msg.content.iter().any(|b| match b {
            crate::message::ContentBlock::Text { text, .. } => {
                let t = text.trim();
                !t.is_empty() && !t.starts_with("<system-reminder>")
            }
            _ => false,
        });
        if !is_visible {
            continue;
        }
        if user_idx == target_prompt_index {
            // Keep through this user message; drop everything after.
            cut_at = Some(i + 1);
            break;
        }
        user_idx += 1;
    }
    let Some(cut) = cut_at else {
        return json!({
            "error": format!("No rewind point at index {target_prompt_index}")
        });
    };
    if !force && cut >= session.messages.len() {
        return json!({
            "result": {
                "success": true,
                "targetPromptIndex": target_prompt_index,
                "revertedFiles": [],
                "cleanFiles": [],
                "conflicts": [],
                "mode": "conversation",
            }
        });
    }
    session.messages.truncate(cut);
    if let Err(e) = session.save() {
        return json!({ "error": format!("Failed to save rewind: {e}") });
    }
    json!({
        "result": {
            "success": true,
            "targetPromptIndex": target_prompt_index,
            "revertedFiles": [],
            "cleanFiles": [],
            "conflicts": [],
            "mode": "conversation",
        }
    })
}

/// `x.ai/session/fork` — clone transcript into a child next-code session.
pub fn session_fork_payload(
    source_session_id: &str,
    new_session_id: Option<&str>,
) -> serde_json::Value {
    let Ok(parent) = Session::load(source_session_id) else {
        return json!({ "error": format!("Unknown session id: {source_session_id}") });
    };
    let mut child = if let Some(nid) = new_session_id.filter(|s| !s.is_empty()) {
        Session::create_with_id(nid.to_string(), Some(parent.id.clone()), parent.title.clone())
    } else {
        Session::create(Some(parent.id.clone()), parent.title.clone())
    };
    child.messages = parent.messages.clone();
    child.working_dir = parent.working_dir.clone();
    child.model = parent.model.clone();
    child.append_fork_notice(&parent.id, parent.display_title_or_name());
    if let Err(e) = child.save() {
        return json!({ "error": format!("Failed to save forked session: {e}") });
    }
    crate::tui::session_picker::invalidate_session_list_cache();
    json!({ "result": { "newSessionId": child.id } })
}

/// Dispatch Face ext methods that map to next-code brain.
pub async fn handle_ext_method(
    method: &str,
    params: &serde_json::Value,
) -> Option<serde_json::Value> {
    Some(match method {
        "x.ai/usage" | "x.ai/billing" => {
            // billing: never return xAI credits; usage text only.
            usage_text_payload().await
        }
        "x.ai/session/info" => {
            let sid = params
                .get("sessionId")
                .or_else(|| params.get("session_id"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            session_info_payload(sid)
        }
        "x.ai/session/rename" => {
            let sid = params
                .get("sessionId")
                .or_else(|| params.get("session_id"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let title = params
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            session_rename_payload(sid, title)
        }
        "x.ai/feedback" => {
            let text = params
                .get("feedbackText")
                .or_else(|| params.get("feedback_text"))
                .and_then(|v| v.as_str());
            feedback_payload(text)
        }
        "x.ai/btw" => {
            let q = params
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            btw_payload(q)
        }
        "x.ai/rewind/points" => {
            let sid = params
                .get("sessionId")
                .or_else(|| params.get("session_id"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            rewind_points_payload(sid)
        }
        "x.ai/rewind/execute" => {
            let sid = params
                .get("sessionId")
                .or_else(|| params.get("session_id"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let idx = params
                .get("targetPromptIndex")
                .or_else(|| params.get("target_prompt_index"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            let force = params
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            rewind_execute_payload(sid, idx, force)
        }
        "x.ai/session/fork" => {
            let sid = params
                .get("sourceSessionId")
                .or_else(|| params.get("sessionId"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let nid = params
                .get("newSessionId")
                .and_then(|v| v.as_str());
            session_fork_payload(sid, nid)
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_usage_empty_mentions_connect() {
        let text = format_usage_text(&[]);
        assert!(text.contains("No connected providers"));
        assert!(text.contains("/connect"));
        assert!(!text.to_lowercase().contains("grok.com"));
    }

    #[test]
    fn feedback_points_at_github_not_xai() {
        let v = feedback_payload(Some("nice"));
        let s = v.to_string();
        assert!(s.contains("github.com/quangdang46/next-code"));
        assert!(!s.contains("x.ai/feedback"));
        assert!(!s.contains("Grok Build"));
    }

    #[test]
    fn btw_empty_usage() {
        let v = btw_payload("  ");
        assert!(
            v["result"]["answer"]
                .as_str()
                .unwrap_or_default()
                .contains("Usage:")
        );
    }
}
