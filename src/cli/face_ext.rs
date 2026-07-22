//! Thin Face ACP `x.ai/*` handlers wired to next-code brain (PR10 gap close).
//!
//! Face chrome keeps calling stock method names; we implement the brain here
//! instead of returning empty `{}`.

use futures::StreamExt;
use serde_json::json;

use crate::message::{ContentBlock, Message, Role, StreamEvent};
use crate::provider::{MultiProvider, Provider};
use crate::session::Session;

/// Soft cap on transcript turns fed into `/btw` and `/recap` side calls.
const SIDE_CALL_MAX_TURNS: usize = 40;
/// Hard cap on recap body characters (stock grok-build uses 1200).
const RECAP_MAX_CHARS: usize = 1200;

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

/// Whether the session has at least one visible user message (for `/recap` gates).
pub fn session_has_user_messages(session_id: &str) -> bool {
    let Ok(session) = Session::load(session_id) else {
        return false;
    };
    session.visible_conversation_messages().iter().any(|m| {
        matches!(m.role, Role::User)
            && m.content.iter().any(|b| match b {
                ContentBlock::Text { text, .. } => {
                    let t = text.trim();
                    !t.is_empty() && !t.starts_with("<system-reminder>")
                }
                _ => false,
            })
    })
}

/// Extract plain-text user/assistant turns for a tool-free side model call.
fn text_only_transcript(session: &Session) -> Vec<Message> {
    let visible = session.visible_conversation_messages();
    let start = visible.len().saturating_sub(SIDE_CALL_MAX_TURNS);
    let mut out = Vec::new();
    for stored in &visible[start..] {
        let text = stored
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text, .. } => {
                    let t = text.trim();
                    if t.is_empty() || t.starts_with("<system-reminder>") {
                        None
                    } else {
                        Some(t)
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        if text.is_empty() {
            continue;
        }
        match stored.role {
            Role::User => out.push(Message::user(&text)),
            Role::Assistant => out.push(Message::assistant_text(&text)),
        }
    }
    out
}

async fn complete_text(messages: &[Message], system: &str) -> Result<String, String> {
    let provider = MultiProvider::new();
    let stream = provider
        .complete(messages, &[], system, None)
        .await
        .map_err(|e| format!("model call failed: {e}"))?;
    let mut text = String::new();
    let mut stream = stream;
    while let Some(event) = stream.next().await {
        if let Ok(StreamEvent::TextDelta(delta)) = event {
            text.push_str(&delta);
        }
    }
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        Err("No response from model".to_string())
    } else {
        Ok(trimmed)
    }
}

/// Stock-style tidy for recap body (UI adds the "Recap —" label).
pub fn clean_recap_text(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    for prefix in [
        "Recap —",
        "Recap -",
        "Recap:",
        "Session recap:",
        "Session Recap:",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim().to_string();
        }
    }
    // First paragraph / sentence block only.
    if let Some((first, _)) = s.split_once("\n\n") {
        s = first.trim().to_string();
    }
    s = s.replace('\n', " ").trim().to_string();
    while s.contains("  ") {
        s = s.replace("  ", " ");
    }
    if s.chars().count() > RECAP_MAX_CHARS {
        let mut cut = s.chars().take(RECAP_MAX_CHARS).collect::<String>();
        if let Some(idx) = cut.rfind(' ') {
            cut.truncate(idx);
        }
        s = cut;
    }
    s
}

/// `x.ai/btw` — tool-free side answer from session context (stock Face overlay).
pub async fn btw_payload(session_id: &str, question: &str) -> serde_json::Value {
    let q = question.trim();
    if q.is_empty() {
        return json!({ "result": { "answer": "Usage: /btw <question>" } });
    }
    if session_id.trim().is_empty() {
        return json!({ "error": "Missing sessionId for /btw" });
    }
    let Ok(session) = Session::load(session_id) else {
        return json!({ "error": format!("Unknown session id: {session_id}") });
    };

    let mut messages = text_only_transcript(&session);
    let wrapped = format!(
        "<system-reminder>This is a side question from the user. \
You must answer this question directly in a single response.\n\n\
IMPORTANT CONTEXT:\n\
- You are a separate, lightweight agent spawned to answer this one question\n\
- The main agent is NOT interrupted — it continues working independently\n\
- You share the conversation context but are a completely separate instance\n\
- Do NOT reference being interrupted or what you were \"previously doing\"\n\n\
CRITICAL CONSTRAINTS:\n\
- You have NO tools available — you cannot read files, run commands, search, or take any actions\n\
- This is a one-off response — there will be no follow-up turns\n\
- You can ONLY provide information based on what you already know from the conversation context\n\
- NEVER say things like \"Let me try...\", \"I'll now...\", or promise to take any action\n\
- If you don't know the answer, say so — do not offer to look it up or investigate\n\n\
Simply answer the question with the information you have.</system-reminder>\n\n\
{q}"
    );
    messages.push(Message::user(&wrapped));

    let system = "You answer brief side questions about the current coding session using only the conversation context. Be concise. No tools.";
    match complete_text(&messages, system).await {
        Ok(answer) => json!({ "result": { "answer": answer } }),
        Err(err) => json!({ "result": { "answer": format!("Couldn't answer side question: {err}") } }),
    }
}

/// Generate a one-line session recap from transcript (does not mutate session).
pub async fn generate_recap_summary(session_id: &str) -> Result<String, String> {
    if session_id.trim().is_empty() {
        return Err("Missing sessionId".to_string());
    }
    let session = Session::load(session_id).map_err(|_| format!("Unknown session id: {session_id}"))?;
    if !session_has_user_messages(session_id) {
        return Err("empty".to_string());
    }

    let mut messages = text_only_transcript(&session);
    let instruction = "\
<system-reminder>Write ONE sentence recap body for a user returning from idle. \
Output ONLY the body (the UI adds the \"Recap —\" label). \
Do NOT call any tools — respond with plain text only.\n\n\
Lead with agency:\n\
- \"You asked …\" if the session was mainly questions, walkthroughs, or review with no landed change.\n\
- \"We …\" if the agent implemented, fixed, merged, or changed code/config/docs.\n\
- If almost nothing happened: \"You had just begun this session.\"\n\n\
Shape: ~25–40 words. No bullets, markdown, or extra labels.</system-reminder>";
    messages.push(Message::user(instruction));

    let system = "You write short session recaps. One sentence. Plain text only.";
    let raw = complete_text(&messages, system).await?;
    let summary = clean_recap_text(&raw);
    if summary.is_empty() {
        Err("empty summary".to_string())
    } else {
        Ok(summary)
    }
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
            let sid = params
                .get("sessionId")
                .or_else(|| params.get("session_id"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let q = params
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            btw_payload(sid, q).await
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
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let v = rt.block_on(btw_payload("sess", "  "));
        assert!(
            v["result"]["answer"]
                .as_str()
                .unwrap_or_default()
                .contains("Usage:")
        );
    }

    #[test]
    fn clean_recap_strips_label_and_caps() {
        let cleaned = clean_recap_text("Recap — We wired the Face /btw side path.\n\nExtra");
        assert_eq!(cleaned, "We wired the Face /btw side path.");
        assert!(!cleaned.to_lowercase().starts_with("recap"));
    }
}
