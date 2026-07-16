//! Teammate view — Claude Code invariant:
//!
//! **Each subagent has a Message-level transcript; view = swap to that
//! transcript** (or empty). Never show lead messages while viewing.
//!
//! jcode multi-session mapping:
//! - **Hard attach** (`resume_session`) = full child session history (primary Enter)
//! - **Soft buffer** (`teammate_transcripts` / `SwarmMemberMessage`) = live mirror
//!   while staying on the lead socket (only when Message-level content exists)
//!
//! Soft body is **not** a SwarmStatus status-dump. Empty until stream/seed has
//! user|assistant|tool content (CC: never fall through to leader).

use crate::protocol::SwarmMemberStatus;
use crate::tui::color_support::rgb as rgb_color;
use jcode_tui_messages::DisplayMessage;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

const MAX_TASK_LABEL_CHARS: usize = 160;

/// True when `msgs` looks like a real agent conversation (CC `task.messages`),
/// not a pure status/chrome novel.
pub fn is_message_level_transcript(msgs: &[DisplayMessage]) -> bool {
    msgs.iter().any(|m| {
        let t = m.content.trim();
        if t.is_empty() {
            return false;
        }
        match m.role.as_str() {
            "user" | "assistant" | "tool" => true,
            "system" => {
                // Tool rows may be system-shaped with [tool] prefix; count them.
                t.starts_with('[')
                    && !t.starts_with("status:")
                    && !t.contains("no stream yet")
                    && !t.contains("No live stream")
            }
            _ => false,
        }
    })
}

/// Seed lead-side transcript lines from a SwarmStatus member snapshot.
///
/// Message-level only: task (user), tools, assistant tail.
/// Returns **empty** when nothing real yet (CC empty-until-bootstrap).
pub fn seed_messages_from_member(member: &SwarmMemberStatus) -> Vec<DisplayMessage> {
    let mut out = Vec::new();

    if let Some(task) = member
        .task_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !looks_like_spawn_meta_prompt(s))
    {
        out.push(
            DisplayMessage::user(truncate_chars(task, MAX_TASK_LABEL_CHARS)).with_title("task"),
        );
    }

    for t in &member.todo_items {
        for tool in &t.tool_intents {
            let st = tool.status.to_ascii_lowercase();
            let line = format!(
                "{} ({st}): {}",
                tool.tool_name,
                truncate_chars(&tool.intent, 120)
            );
            let key = if tool.tool_call_id.is_empty() {
                format!("{}:tool:{}", member.session_id, tool.tool_name)
            } else {
                format!("{}:tool:{}", member.session_id, tool.tool_call_id)
            };
            out.push(
                DisplayMessage::system(format!("[{}] {line}", tool.tool_name)).with_title(key),
            );
        }
    }

    if let Some(tail) = member
        .output_tail
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        out.push(DisplayMessage::assistant(truncate_chars(tail, 4000)).with_title("stream"));
    }
    out
}

/// Last non-empty display line for tree preview under a child row.
pub fn preview_line_from_messages(msgs: &[DisplayMessage]) -> Option<String> {
    msgs.iter()
        .rev()
        .find(|m| {
            let t = m.content.trim();
            !t.is_empty()
                && (m.role == "assistant"
                    || m.role == "tool"
                    || m.title.as_deref() == Some("stream")
                    || m.title.as_deref() == Some("task"))
        })
        .map(|m| {
            let line = m
                .content
                .lines()
                .last()
                .unwrap_or(m.content.as_str())
                .trim();
            truncate_chars(line, 80)
        })
        .filter(|s| !s.is_empty())
}

/// Message-level snapshot for soft buffer / tree seed.
///
/// **Empty when nothing real** — CC never fills view with status novels.
/// Prefer live `SwarmMemberMessage` buffer; this is the SwarmStatus fallback.
pub fn build_view_messages(member: &SwarmMemberStatus) -> Vec<DisplayMessage> {
    // Same path as seed: task + tools + stream only.
    seed_messages_from_member(member)
}

/// True if text looks like the coordinator's long spawn / UI-test instruction
/// rather than a short worker task.
fn looks_like_spawn_meta_prompt(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    let long = s.chars().count() > 160;
    let markers = [
        "dùng tool swarm",
        "action=spawn",
        "shift+",
        "enter soft-view",
        "hard-attach",
        "mục tiêu ui",
        "giữ nó chạy",
        "spawn đúng",
        "team-lead",
        "shift+up/down",
    ];
    long || markers.iter().any(|m| lower.contains(m))
}

pub(crate) fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Public alias for tree preview callers.
pub(crate) fn truncate_chars_public(s: &str, max: usize) -> String {
    truncate_chars(s, max)
}

fn short_model(model: &str) -> String {
    let short = model.rsplit('/').next().unwrap_or(model);
    if short.len() > 16 {
        format!("{}…", &short[..14])
    } else {
        short.to_string()
    }
}

fn format_elapsed(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

pub fn member_tree_stats(member: &SwarmMemberStatus) -> Option<String> {
    let mut parts = Vec::new();
    let tool_n: usize = member.todo_items.iter().map(|t| t.tool_intents.len()).sum();
    if tool_n > 0 {
        parts.push(format!(
            "{tool_n} tool {}",
            if tool_n == 1 { "use" } else { "uses" }
        ));
    }
    if let Some(secs) = member.runtime.elapsed_secs {
        if secs > 0 {
            parts.push(format_elapsed(secs));
        }
    }
    if let Some(model) = member
        .runtime
        .model
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        parts.push(short_model(model));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

pub fn find_member<'a>(
    members: &'a [SwarmMemberStatus],
    session_id: &str,
) -> Option<&'a SwarmMemberStatus> {
    members.iter().find(|m| m.session_id == session_id)
}

pub fn member_is_running(member: &SwarmMemberStatus) -> bool {
    matches!(
        member.status.trim().to_ascii_lowercase().as_str(),
        "running" | "processing" | "active" | "starting" | "ready"
    )
}

pub fn member_is_terminal(member: &SwarmMemberStatus) -> bool {
    matches!(
        member.status.trim().to_ascii_lowercase().as_str(),
        "completed"
            | "done"
            | "ok"
            | "success"
            | "failed"
            | "error"
            | "crashed"
            | "stopped"
            | "cancelled"
            | "canceled"
            | "interrupted"
            | "killed"
    )
}

/// Claude Code `TeammateViewHeader` — sticky above transcript while viewing.
///
/// Exact CC layout (`TeammateViewHeader.tsx`):
///   Viewing @{name} · esc return
///   {task.prompt}          // dim, second line only
///
/// Do **not** pile return hints here, in the tree, status bar, separator, and
/// input hint all at once — that is what made the UI noisy vs CC.
pub fn draw_viewing_chrome(
    frame: &mut Frame,
    area: Rect,
    agent_name: &str,
    hard_attached: bool,
    member: Option<&SwarmMemberStatus>,
) {
    if area.height == 0 || area.width < 8 {
        return;
    }
    let name = agent_name.trim().trim_start_matches('@');
    let name = if name.is_empty() { "agent" } else { name };
    let name_color = rgb_color(80, 220, 100);
    let dim = Style::default().fg(rgb_color(140, 140, 150));
    let accent = Style::default()
        .fg(rgb_color(200, 200, 210))
        .add_modifier(Modifier::BOLD);

    // Line 1 — CC: "Viewing @name · esc return" (same wording soft + hard).
    let mut lines: Vec<Line<'static>> = vec![Line::from(vec![
        Span::styled("Viewing ", Style::default().fg(rgb_color(220, 220, 230))),
        Span::styled(
            format!("@{name}"),
            Style::default().fg(name_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" · ", dim),
        Span::styled("esc", accent),
        Span::styled(" return", dim),
    ])];

    // Line 2 — CC: task prompt only (dim). No extra keybinding spam.
    if area.height >= 2 {
        let second = if let Some(m) = member {
            m.task_label
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && !looks_like_spawn_meta_prompt(s))
                .map(|s| truncate_chars(s, 120))
                .or_else(|| {
                    // Soft without a clean task: one short status word, not a novel.
                    if !hard_attached {
                        Some(m.status.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        } else {
            String::new()
        };
        if !second.is_empty() {
            lines.push(Line::from(Span::styled(second, dim)));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}

/// CC puts esc only in `TeammateViewHeader` — no status-bar / footer fragment.
/// Kept as empty so any leftover call site stays silent.
pub fn viewing_footer_hint_spans() -> Vec<Span<'static>> {
    Vec::new()
}

/// @deprecated — no dedicated return bar (header owns esc).
pub fn hard_attach_status_line(_agent_name: &str) -> Line<'static> {
    Line::from("")
}

/// @deprecated — status bar no longer hijacks with a full replacement line.
pub fn viewing_status_spans(_agent_name: &str, _hard_attached: bool) -> Vec<Span<'static>> {
    Vec::new()
}

/// @deprecated separator stays plain ─── (CC has no hint in the rule).
pub fn viewing_separator_label(_agent_name: &str, _hard_attached: bool) -> String {
    String::new()
}

pub fn header_height(viewing: bool, available: u16) -> u16 {
    if !viewing {
        0
    } else if available >= 3 {
        2
    } else if available >= 2 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{SwarmTodoItem, SwarmToolIntent};

    fn sample_member() -> SwarmMemberStatus {
        SwarmMemberStatus {
            session_id: "ses_duck".into(),
            friendly_name: Some("duck".into()),
            status: "running".into(),
            detail: Some("startup queued".into()),
            task_label: Some("print ticks".into()),
            role: None,
            is_headless: Some(true),
            live_attachments: None,
            status_age_secs: Some(3),
            output_tail: Some("tick 1\ntick 2".into()),
            report_back_to_session_id: None,
            todo_progress: Some((1, 3)),
            todo_items: vec![SwarmTodoItem {
                content: "step one".into(),
                status: "in_progress".into(),
                tool_intents: vec![SwarmToolIntent {
                    tool_call_id: String::new(),
                    tool_name: "bash".into(),
                    intent: "hostname".into(),
                    status: "running".into(),
                    progress: None,
                }],
            }],
            runtime: crate::protocol::SwarmMemberRuntime {
                model: Some("gpt-4.1".into()),
                provider: None,
                auth_method: None,
                effort: None,
                elapsed_secs: Some(42),
            },
        }
    }

    #[test]
    fn build_view_does_not_spam_meta_spawn_prompt() {
        let mut m = sample_member();
        m.detail = Some(
            "Dùng tool swarm với action=spawn, spawn ĐÚNG 1 agent con và giữ nó chạy ít nhất 25–40 giây. Prompt cho agent..."
                .into(),
        );
        m.task_label = m.detail.clone();
        m.output_tail = None;
        let msgs = build_view_messages(&m);
        let blob: String = msgs.iter().map(|x| x.content.clone()).collect();
        assert!(
            !blob.contains("Dùng tool swarm"),
            "must not dump spawn meta-prompt: {blob}"
        );
        // Tools still seed; no status novel.
        assert!(
            blob.contains("bash") || blob.contains("hostname"),
            "expected tool rows without stream: {blob}"
        );
        assert!(!blob.contains("status:"), "no status dump in body: {blob}");
    }

    #[test]
    fn build_view_empty_when_nothing_real() {
        let mut m = sample_member();
        m.detail = None;
        m.task_label = None;
        m.output_tail = None;
        m.todo_items.clear();
        m.todo_progress = None;
        let msgs = build_view_messages(&m);
        // CC: empty until bootstrap — never fall through to lead with a novel.
        assert!(msgs.is_empty(), "expected empty transcript: {msgs:?}");
        assert!(!is_message_level_transcript(&msgs));
    }

    #[test]
    fn build_view_messages_includes_stream_and_tools() {
        let msgs = build_view_messages(&sample_member());
        assert!(!msgs.iter().any(|m| m.content.contains("esc return")));
        assert!(
            msgs.iter()
                .any(|m| m.role == "assistant" && m.content.contains("tick"))
        );
        assert!(msgs.iter().any(|m| m.content.contains("bash")));
        assert!(is_message_level_transcript(&msgs));
    }

    #[test]
    fn message_level_rejects_status_only() {
        let msgs = vec![DisplayMessage::system("status: running")];
        assert!(!is_message_level_transcript(&msgs));
        let msgs = vec![DisplayMessage::assistant("hello")];
        assert!(is_message_level_transcript(&msgs));
    }

    #[test]
    fn member_tree_stats_shows_tools_and_elapsed() {
        let s = member_tree_stats(&sample_member()).expect("stats");
        assert!(s.contains("tool"));
    }

    #[test]
    fn member_is_terminal_maps_cancelled() {
        let mut m = sample_member();
        m.status = "cancelled".into();
        assert!(member_is_terminal(&m));
        m.status = "running".into();
        assert!(member_is_running(&m));
    }

    #[test]
    fn seed_messages_from_member_uses_tail_and_tools() {
        let m = sample_member();
        let msgs = seed_messages_from_member(&m);
        assert!(
            msgs.iter()
                .any(|x| x.role == "assistant" && x.content.contains("tick")),
            "{msgs:?}"
        );
        assert!(
            msgs.iter().any(|x| x.content.contains("bash")),
            "tools: {msgs:?}"
        );
        let preview = preview_line_from_messages(&msgs).expect("preview");
        assert!(
            preview.contains("tick") || preview.contains("bash"),
            "{preview}"
        );
    }

    #[test]
    fn chrome_copy_matches_claude_code_contract() {
        // TeammateViewHeader.tsx: "Viewing @name · esc return" only.
        // No footer/status esc fragment (CC has none).
        assert!(viewing_footer_hint_spans().is_empty());
        assert!(viewing_status_spans("duck", true).is_empty());
        // Separator is plain (no embedded hint).
        assert!(viewing_separator_label("duck", true).is_empty());
        assert_eq!(header_height(true, 10), 2);
        assert_eq!(header_height(true, 2), 1);
        assert_eq!(header_height(false, 10), 0);
    }
}
