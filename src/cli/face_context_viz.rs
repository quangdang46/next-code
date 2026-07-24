//! Face `/context` API-true aggregation.
//!
//! Mirrors Claude Code's `/context` path: analyze the same view the model
//! sees (compact boundary + system/tools/skills), not raw billing totals.
//!
//! Claude refs (local plugin):
//! - `src/commands/context/context.tsx` (`toApiView` + microcompact)
//! - `src/components/ContextVisualization.tsx`
//! - `src/utils/analyzeContext.ts`

use std::path::Path;

use serde_json::json;

use crate::compaction::content_char_count;
use crate::message::{ContentBlock, Role};
use crate::prompt::{self, SkillInfo};
use crate::provider::models::context_limit_for_model;
use crate::session::{Session, StoredMessage};
use crate::skill::SkillRegistry;

/// Default auto-compact threshold shown in Face `/context` chrome.
pub const AUTO_COMPACT_THRESHOLD_PERCENT: u8 = 85;

/// Fallback context window when the model catalog has no entry.
const DEFAULT_CONTEXT_WINDOW: u64 = 128_000;

/// Cold-path tool-definition estimate (matches TUI `context_snapshot` fallback).
const FALLBACK_TOOL_DEFS_COUNT: usize = 25;
const FALLBACK_CHARS_PER_TOOL_DEF: usize = 500;

/// Aggregated Face `/context` snapshot (API-true view model).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiViewContext {
    pub model: String,
    pub used: u64,
    pub total: u64,
    pub free_tokens: u64,
    pub message_tokens: u64,
    pub system_prompt_tokens: u64,
    pub tool_definitions_tokens: u64,
    pub tool_definitions_count: usize,
    pub turn_count: u64,
    pub tool_call_count: u64,
    pub compaction_count: u64,
    pub usage_pct: u8,
    pub message_count: u64,
    pub auto_compact_threshold_percent: u8,
    pub usage_categories: Vec<UsageCategory>,
    /// Messages counted after compact boundary (includes synthetic summary).
    pub api_view_message_count: u64,
    pub compacted_skipped: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageCategory {
    pub label: String,
    pub tokens: u64,
    pub detail: Option<String>,
}

/// Build an API-true context snapshot for Face `/context` / `x.ai/session/info`.
pub fn aggregate_api_view_context(session: &Session) -> ApiViewContext {
    let model = session
        .model
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let working_dir = session.working_dir.as_deref().map(Path::new);

    let skill_infos = load_skill_infos(working_dir);
    let (_prompt, prompt_info) =
        prompt::build_system_prompt_full(None, &skill_infos, false, None, working_dir, None, None);

    // System bucket = full prompt minus the skills listing (skills shown separately
    // as a usage category, Claude-style). Skills chars still count toward `used`
    // via the skills category tokens.
    let system_chars = prompt_info
        .total_chars
        .saturating_sub(prompt_info.skills_chars);
    let skills_chars = prompt_info.skills_chars;
    let skills_count = skill_infos.len();

    let compacted_skip = session
        .compaction
        .as_ref()
        .map(|c| c.compacted_count.min(session.messages.len()))
        .unwrap_or(0);
    let summary_chars = session
        .compaction
        .as_ref()
        .map(|c| c.summary_text.len())
        .filter(|&n| n > 0)
        .unwrap_or(0);

    let active = &session.messages[compacted_skip..];
    let mut message_chars = summary_chars;
    let mut turn_count = 0u64;
    let mut tool_call_count = 0u64;
    for msg in active {
        message_chars += content_char_count(&msg.content);
        if matches!(msg.role, Role::User) && !is_session_context_reminder(msg) {
            turn_count += 1;
        }
        for block in &msg.content {
            if matches!(block, ContentBlock::ToolUse { .. }) {
                tool_call_count += 1;
            }
        }
    }
    // Compact summary counts as one synthetic user turn for the API view.
    let api_view_message_count =
        active.len() as u64 + if summary_chars > 0 { 1 } else { 0 };

    let tool_definitions_count = FALLBACK_TOOL_DEFS_COUNT;
    let tool_defs_chars = tool_definitions_count * FALLBACK_CHARS_PER_TOOL_DEF;

    let system_prompt_tokens = estimate_tokens_u64(system_chars);
    let message_tokens = estimate_tokens_u64(message_chars);
    let tool_definitions_tokens = estimate_tokens_u64(tool_defs_chars);
    let skills_tokens = estimate_tokens_u64(skills_chars);

    let estimated_used = system_prompt_tokens
        .saturating_add(message_tokens)
        .saturating_add(tool_definitions_tokens)
        .saturating_add(skills_tokens);

    // Prefer observed last-input tokens when they exceed the estimate (provider
    // truth), but never under-count the reconstructed view.
    let observed_input = latest_observed_input_tokens(session);
    let used = estimated_used.max(observed_input);

    let total = context_limit_for_model(&model)
        .map(|n| n as u64)
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_CONTEXT_WINDOW)
        .max(used.max(1));

    let free_tokens = total.saturating_sub(used);
    let usage_pct = if total == 0 {
        0
    } else {
        ((used as f64 / total as f64) * 100.0).round() as u8
    };

    let mut usage_categories = Vec::new();
    if skills_tokens > 0 || skills_count > 0 {
        usage_categories.push(UsageCategory {
            label: "Skills".to_string(),
            tokens: skills_tokens,
            detail: Some(count_detail(skills_count, "skill")),
        });
    }
    if summary_chars > 0 {
        usage_categories.push(UsageCategory {
            label: "Compact summary".to_string(),
            tokens: estimate_tokens_u64(summary_chars),
            detail: Some(format!("{compacted_skip} msgs compacted")),
        });
    }

    ApiViewContext {
        model,
        used,
        total,
        free_tokens,
        message_tokens,
        system_prompt_tokens,
        tool_definitions_tokens,
        tool_definitions_count,
        turn_count,
        tool_call_count,
        compaction_count: if compacted_skip > 0 { 1 } else { 0 },
        usage_pct,
        message_count: session.messages.len() as u64,
        auto_compact_threshold_percent: AUTO_COMPACT_THRESHOLD_PERCENT,
        usage_categories,
        api_view_message_count,
        compacted_skipped: compacted_skip as u64,
    }
}

/// Serialize into the `x.ai/session/info` envelope Face already parses.
pub fn session_info_json(session: &Session) -> serde_json::Value {
    let snap = aggregate_api_view_context(session);
    let cwd = session.working_dir.clone().unwrap_or_default();
    let categories: Vec<serde_json::Value> = snap
        .usage_categories
        .iter()
        .map(|c| {
            json!({
                "label": c.label,
                "tokens": c.tokens,
                "detail": c.detail,
            })
        })
        .collect();

    json!({
        "result": {
            "sessionId": session.id,
            "cwd": cwd,
            "data": {
                "model": snap.model,
                "modelDisplayName": session.display_title_or_name(),
                "turns": snap.turn_count,
                "turnIndex": snap.turn_count,
                "context": {
                    "used": snap.used,
                    "total": snap.total,
                    "freeTokens": snap.free_tokens,
                    "messageTokens": snap.message_tokens,
                    "systemPromptTokens": snap.system_prompt_tokens,
                    "turnCount": snap.turn_count,
                    "toolCallCount": snap.tool_call_count,
                    "compactionCount": snap.compaction_count,
                    "usagePct": snap.usage_pct,
                    "usageCategories": categories,
                    "toolDefinitionsTokens": snap.tool_definitions_tokens,
                    "toolDefinitionsCount": snap.tool_definitions_count,
                    "messageCount": snap.message_count,
                    "autoCompactThresholdPercent": snap.auto_compact_threshold_percent,
                }
            }
        }
    })
}

fn estimate_tokens_u64(chars: usize) -> u64 {
    // Same bytes/4 heuristic as `util::estimate_tokens` / xai-token-estimation.
    (chars as u64) / 4
}

fn count_detail(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

fn load_skill_infos(working_dir: Option<&Path>) -> Vec<SkillInfo> {
    let registry = SkillRegistry::load_for_working_dir(working_dir).unwrap_or_default();
    registry
        .list()
        .into_iter()
        .map(|s| SkillInfo {
            name: s.name.clone(),
            description: s.description.clone(),
        })
        .collect()
}

fn is_session_context_reminder(msg: &StoredMessage) -> bool {
    msg.content.iter().any(|b| match b {
        ContentBlock::Text { text, .. } => {
            text.starts_with("<system-reminder>\n# Session Context")
        }
        _ => false,
    })
}

fn latest_observed_input_tokens(session: &Session) -> u64 {
    session
        .messages
        .iter()
        .rev()
        .find_map(|m| m.token_usage.as_ref().map(|u| u.input_tokens))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ContentBlock;
    use crate::session::{StoredCompactionState, StoredMessage, StoredTokenUsage};
    use chrono::Utc;

    fn msg(role: Role, text: &str) -> StoredMessage {
        StoredMessage {
            id: format!("m-{}", text.chars().take(8).collect::<String>()),
            role,
            content: vec![ContentBlock::Text {
                text: text.to_string(),
                cache_control: None,
            }],
            display_role: None,
            timestamp: Some(Utc::now()),
            tool_duration_ms: None,
            token_usage: None,
        }
    }

    fn session_with(messages: Vec<StoredMessage>) -> Session {
        let mut session = Session::create_with_id("session_face_ctx_viz_test".into(), None, None);
        session.model = Some("gpt-4o".into());
        session.working_dir = Some(std::env::temp_dir().display().to_string());
        session.replace_messages(messages);
        session
    }

    #[test]
    fn aggregate_counts_system_and_messages_not_billing_totals() {
        let session = session_with(vec![
            msg(Role::User, &"u".repeat(400)),
            msg(Role::Assistant, &"a".repeat(400)),
        ]);
        let snap = aggregate_api_view_context(&session);
        assert!(snap.system_prompt_tokens > 0, "system prompt should contribute");
        assert!(snap.message_tokens > 0, "messages should contribute");
        assert!(snap.tool_definitions_tokens > 0);
        assert_eq!(snap.tool_definitions_count, FALLBACK_TOOL_DEFS_COUNT);
        // Window must be a real context limit, not used==total.
        assert!(snap.total >= DEFAULT_CONTEXT_WINDOW.min(128_000) || snap.total > snap.used);
        assert_eq!(snap.free_tokens, snap.total.saturating_sub(snap.used));
        assert!(snap.used < snap.total, "fresh short chat should not fill window");
    }

    #[test]
    fn compact_boundary_excludes_pre_compact_messages() {
        let long = "x".repeat(4_000);
        let mut session = session_with(vec![
            msg(Role::User, &long),
            msg(Role::Assistant, &long),
            msg(Role::User, "hi"),
            msg(Role::Assistant, "hello"),
        ]);
        let before = aggregate_api_view_context(&session).message_tokens;

        session.compaction = Some(StoredCompactionState {
            summary_text: "summary of earlier turns".into(),
            openai_encrypted_content: None,
            covers_up_to_turn: 2,
            original_turn_count: 4,
            compacted_count: 2,
        });
        let after = aggregate_api_view_context(&session);
        assert!(
            after.message_tokens < before,
            "compacted view should drop pre-boundary message chars ({after:?} vs before={before})"
        );
        assert_eq!(after.compacted_skipped, 2);
        assert_eq!(after.compaction_count, 1);
        assert!(
            after
                .usage_categories
                .iter()
                .any(|c| c.label == "Compact summary")
        );
    }

    #[test]
    fn observed_input_tokens_floor_used() {
        let mut session = session_with(vec![msg(Role::User, "hi")]);
        session.messages[0].token_usage = Some(StoredTokenUsage {
            input_tokens: 50_000,
            output_tokens: 10,
            cache_read_input_tokens: None,
            cache_creation_input_tokens: None,
        });
        let snap = aggregate_api_view_context(&session);
        assert!(snap.used >= 50_000);
    }

    #[test]
    fn session_info_json_wire_shape_is_camel_case() {
        let session = session_with(vec![msg(Role::User, "hello world")]);
        let v = session_info_json(&session);
        let ctx = &v["result"]["data"]["context"];
        assert!(ctx["used"].as_u64().unwrap() > 0);
        assert!(ctx["total"].as_u64().unwrap() > ctx["used"].as_u64().unwrap());
        assert!(ctx.get("freeTokens").is_some());
        assert!(ctx.get("systemPromptTokens").is_some());
        assert!(ctx.get("messageTokens").is_some());
        assert!(ctx.get("toolDefinitionsTokens").is_some());
        assert!(ctx.get("toolDefinitionsCount").is_some());
        assert!(ctx.get("usageCategories").is_some());
        assert!(ctx.get("autoCompactThresholdPercent").is_some());
        assert_eq!(
            ctx["autoCompactThresholdPercent"].as_u64(),
            Some(AUTO_COMPACT_THRESHOLD_PERCENT as u64)
        );
    }
}
