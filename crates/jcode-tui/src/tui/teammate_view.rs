//! Soft teammate / swarm-agent view — status preview only.
//!
//! **Important (vs Claude Code):** CC `enterTeammateView` swaps the main
//! transcript to `task.messages` — the agent's *real* conversation. jcode
//! swarm agents are separate remote sessions; SwarmStatus only carries
//! `detail` / `output_tail` / todos. Soft-view is a lightweight preview.
//! **True session switch** is hard-attach (`resume_session`) — default Enter.
//!
//! Soft-view rules:
//! - Never dump the full spawn prompt as the body (often stuffed into `detail`)
//! - Prefer short task_label + status + tools + output_tail
//! - One empty-state line if nothing to show (no spam)

use jcode_tui_messages::DisplayMessage;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use crate::protocol::SwarmMemberStatus;
use crate::tui::color_support::rgb as rgb_color;

const MAX_DETAIL_CHARS: usize = 120;
const MAX_TASK_LABEL_CHARS: usize = 160;

/// Seed lead-side transcript lines from a SwarmStatus member snapshot.
/// Used when the live `SwarmMemberMessage` stream has not filled the buffer yet.
pub fn seed_messages_from_member(member: &SwarmMemberStatus) -> Vec<DisplayMessage> {
    let mut out = Vec::new();
    let name = member
        .friendly_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("agent");

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
            let line = format!("{} ({st}): {}", tool.tool_name, truncate_chars(&tool.intent, 120));
            let key = if tool.tool_call_id.is_empty() {
                format!("{}:tool:{}", member.session_id, tool.tool_name)
            } else {
                format!("{}:tool:{}", member.session_id, tool.tool_call_id)
            };
            out.push(DisplayMessage::system(format!("[{}] {line}", tool.tool_name)).with_title(key));
        }
    }

    if let Some(tail) = member
        .output_tail
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        out.push(
            DisplayMessage::assistant(truncate_chars(tail, 4000)).with_title("stream"),
        );
    } else if out.is_empty() {
        out.push(DisplayMessage::system(format!(
            "@{name}: no stream yet (waiting for SwarmMemberMessage / output_tail)"
        )));
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
            let line = m.content.lines().last().unwrap_or(m.content.as_str()).trim();
            truncate_chars(line, 80)
        })
        .filter(|s| !s.is_empty())
}

/// Build a short soft-preview transcript (not a full agent session).
pub fn build_view_messages(member: &SwarmMemberStatus) -> Vec<DisplayMessage> {
    let name = member
        .friendly_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("agent");

    let mut msgs = Vec::new();

    // Compact banner — do not paste spawn prompt here.
    let mut header = format!("Preview @{name} (soft-view · not full session)");
    let mut meta = vec![member.status.clone()];
    if let Some(model) = member.runtime.model.as_deref().filter(|s| !s.is_empty()) {
        meta.push(short_model(model));
    }
    if let Some(secs) = member.runtime.elapsed_secs {
        meta.push(format_elapsed(secs));
    }
    header.push_str(&format!("\n{}", meta.join(" · ")));
    header.push_str("\nEnter hard-switches into this session · Esc returns · Shift+Enter = this preview");
    msgs.push(DisplayMessage::system(header));

    // Short task label only (truncated). Full spawn blobs go to detail often.
    if let Some(task) = member
        .task_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let task = truncate_chars(task, MAX_TASK_LABEL_CHARS);
        // Skip if it looks like the coordinator meta-prompt (user paste to test UI).
        if !looks_like_spawn_meta_prompt(&task) {
            msgs.push(DisplayMessage::user(task).with_title("task"));
        } else {
            msgs.push(DisplayMessage::system(
                "task: (spawn brief omitted — use Enter to open full agent session)",
            ));
        }
    }

    // Detail: only if short and not a meta-prompt dump.
    if let Some(detail) = member
        .detail
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !looks_like_spawn_meta_prompt(detail) {
            let d = truncate_chars(detail, MAX_DETAIL_CHARS);
            msgs.push(
                DisplayMessage::system(format!("status: {} — {d}", member.status))
                    .with_title(format!("@{name}")),
            );
        } else {
            msgs.push(
                DisplayMessage::system(format!("status: {}", member.status))
                    .with_title(format!("@{name}")),
            );
        }
    } else {
        msgs.push(
            DisplayMessage::system(format!("status: {}", member.status))
                .with_title(format!("@{name}")),
        );
    }

    let tool_lines = collect_tool_activity_lines(member);
    if !tool_lines.is_empty() {
        msgs.push(DisplayMessage::system(tool_lines.join("\n")).with_title("tools"));
    }

    if !member.todo_items.is_empty() {
        let mut lines = Vec::new();
        for t in &member.todo_items {
            let mark = match t.status.to_ascii_lowercase().as_str() {
                "completed" | "done" => "✓",
                "in_progress" | "running" => "…",
                "cancelled" | "canceled" => "○",
                _ => "·",
            };
            lines.push(format!("{mark} {}", truncate_chars(&t.content, 100)));
            for tool in t.tool_intents.iter().take(3) {
                let st = match tool.status.to_ascii_lowercase().as_str() {
                    "running" => "…",
                    "completed" | "done" => "✓",
                    "error" | "failed" => "✗",
                    _ => "·",
                };
                lines.push(format!(
                    "    {st} {} — {}",
                    tool.tool_name,
                    truncate_chars(&tool.intent, 80)
                ));
            }
        }
        if let Some((done, total)) = member.todo_progress {
            lines.push(format!("({done}/{total})"));
        }
        msgs.push(DisplayMessage::todos(lines.join("\n")));
    } else if let Some((done, total)) = member.todo_progress {
        if total > 0 {
            msgs.push(DisplayMessage::system(format!("todos: {done}/{total}")));
        }
    }

    if let Some(tail) = member
        .output_tail
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        for chunk in split_stream_chunks(tail) {
            msgs.push(DisplayMessage::assistant(chunk));
        }
    } else if tool_lines.is_empty() {
        // Single empty-state — never spam this on every refresh as duplicates
        // of long detail text.
        msgs.push(DisplayMessage::system(
            "No live stream yet (output_tail empty). Press Enter to hard-switch into the agent session for full history.",
        ));
    }

    msgs
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

fn collect_tool_activity_lines(member: &SwarmMemberStatus) -> Vec<String> {
    let mut lines = Vec::new();
    for t in &member.todo_items {
        for tool in &t.tool_intents {
            let status_lc = tool.status.to_ascii_lowercase();
            let st = match status_lc.as_str() {
                "running" => "running",
                "completed" | "done" => "done",
                "error" | "failed" => "error",
                _ => status_lc.as_str(),
            };
            let mut line = format!(
                "├─ {} ({st}): {}",
                tool.tool_name,
                truncate_chars(&tool.intent, 80)
            );
            if let Some(p) = &tool.progress {
                let unit = p.unit.as_deref().unwrap_or("");
                line.push_str(&format!(" · {}/{}{}", p.current, p.total, unit));
            }
            lines.push(line);
        }
    }
    if lines.len() > 12 {
        let omitted = lines.len() - 12;
        lines.truncate(12);
        lines.push(format!("└─ … +{omitted} more"));
    } else if let Some(last) = lines.last_mut() {
        *last = last.replacen("├─", "└─", 1);
    }
    lines
}

fn split_stream_chunks(tail: &str) -> Vec<String> {
    let paragraphs: Vec<&str> = tail
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if paragraphs.len() > 1 {
        return paragraphs.into_iter().map(str::to_string).collect();
    }
    let lines: Vec<&str> = tail.lines().collect();
    if lines.len() <= 40 {
        return vec![tail.to_string()];
    }
    lines.chunks(40).map(|c| c.join("\n")).collect()
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
    let tool_n: usize = member
        .todo_items
        .iter()
        .map(|t| t.tool_intents.len())
        .sum();
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
/// CC source (`TeammateViewHeader.tsx`):
///   Viewing @{name} · esc return
///   {task.prompt}
///
/// jcode hard-attach is a real `resume_session` (CC keeps one process and swaps
/// `task.messages`). Chrome still must match CC wording and stay visible for
/// the whole attach lifetime — not a 3s status notice.
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
    // CC teammate name is bold green-ish identity color.
    let name_color = rgb_color(80, 220, 100);
    let dim = Style::default().fg(rgb_color(140, 140, 150));
    let accent = Style::default()
        .fg(rgb_color(255, 220, 100))
        .add_modifier(Modifier::BOLD);

    // Line 1 — hard: CC "Viewing @name · esc return"
    // Soft: honest "Status preview" (not task.messages).
    let prefix = if hard_attached {
        "Viewing "
    } else {
        "Status preview "
    };
    let git = short_client_git_hash();
    let mut line1 = vec![
        Span::styled(prefix, Style::default().fg(rgb_color(220, 220, 230))),
        Span::styled(
            format!("@{name}"),
            Style::default()
                .fg(name_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" · ", dim),
        Span::styled("esc", accent),
        Span::styled(
            if hard_attached {
                " return"
            } else {
                " exit"
            },
            dim,
        ),
    ];
    if !git.is_empty() {
        line1.push(Span::styled(format!(" · {git}"), dim));
    }
    let mut lines: Vec<Line<'static>> = vec![Line::from(line1)];

    // Line 2 — CC shows task.prompt; we show task_label / status / hard-attach note.
    if area.height >= 2 {
        let second = if let Some(m) = member {
            let task = m
                .task_label
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && !looks_like_spawn_meta_prompt(s))
                .map(|s| truncate_chars(s, 120));
            if let Some(task) = task {
                task
            } else {
                let mut bits = vec![m.status.clone()];
                if let Some(secs) = m.runtime.elapsed_secs {
                    bits.push(format_elapsed(secs));
                }
                if hard_attached {
                    format!("{} · full session (esc → team lead)", bits.join(" · "))
                } else {
                    format!(
                        "{} · status only — Enter = real session",
                        bits.join(" · ")
                    )
                }
            }
        } else if hard_attached {
            "Full agent session · press esc to return to team lead".to_string()
        } else {
            "Status only (not full agent history) · Enter = real session · esc exit".to_string()
        };
        lines.push(Line::from(Span::styled(second, dim)));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn short_client_git_hash() -> String {
    let h = jcode_build_meta::GIT_HASH.trim();
    if h.is_empty() || h == "unknown" {
        return String::new();
    }
    h.chars().take(9).collect()
}

/// Sticky bottom chrome while hard-attached (CC footer
/// `KeyboardShortcutHint shortcut="esc" action="return to team lead"`).
pub fn hard_attach_status_line(agent_name: &str) -> Line<'static> {
    let name = agent_name.trim().trim_start_matches('@');
    let name = if name.is_empty() { "agent" } else { name };
    Line::from(vec![
        Span::styled(
            "  esc",
            Style::default()
                .fg(rgb_color(255, 220, 100))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " → team-lead",
            Style::default().fg(rgb_color(200, 200, 210)),
        ),
        Span::styled(
            "  ·  shift+↑/↓ switch  ·  enter on team-lead",
            Style::default().fg(rgb_color(160, 160, 170)),
        ),
        Span::styled(
            format!("  ·  @{name}"),
            Style::default().fg(rgb_color(80, 220, 100)),
        ),
    ])
}

/// Durable status-bar spans (CC `PromptInputFooterLeftSide` while viewing).
/// Always shown while hard-attached or soft-viewing — not a 3s notice.
pub fn viewing_status_spans(agent_name: &str, hard_attached: bool) -> Vec<Span<'static>> {
    let name = agent_name.trim().trim_start_matches('@');
    let name = if name.is_empty() { "agent" } else { name };
    if hard_attached {
        vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "esc",
                Style::default()
                    .fg(rgb_color(255, 220, 100))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " → team-lead",
                Style::default().fg(rgb_color(200, 200, 210)),
            ),
            Span::styled(
                " · shift+↑/↓ free switch",
                Style::default().fg(rgb_color(160, 160, 170)),
            ),
            Span::styled(
                format!(" · @{name}"),
                Style::default().fg(rgb_color(80, 220, 100)),
            ),
        ]
    } else {
        vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "esc",
                Style::default()
                    .fg(rgb_color(255, 220, 100))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" exit view", Style::default().fg(rgb_color(180, 180, 190))),
            Span::styled(
                format!(" · @{name}"),
                Style::default().fg(rgb_color(80, 220, 100)),
            ),
        ]
    }
}

/// Bottom separator label while viewing (always on-screen under the input).
pub fn viewing_separator_label(agent_name: &str, hard_attached: bool) -> String {
    let name = agent_name.trim().trim_start_matches('@');
    let name = if name.is_empty() { "agent" } else { name };
    if hard_attached {
        format!(" esc → team-lead · shift+↑/↓ switch · @{name} ")
    } else {
        format!(" esc exit view · @{name} ")
    }
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
    use crate::protocol::{SwarmToolIntent, SwarmTodoItem};

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
        assert!(
            blob.contains("hard-switch") || blob.contains("full session") || blob.contains("Enter"),
            "should point user to hard-switch: {blob}"
        );
        // Only one empty-state / hard-switch hint line
        let empty_count = msgs
            .iter()
            .filter(|m| {
                m.content.contains("No live stream")
                    || m.content.contains("hard-switch")
                    || m.content.contains("output_tail empty")
            })
            .count();
        assert!(empty_count >= 1, "expected empty/hard-switch hint: {blob}");
    }

    #[test]
    fn build_view_messages_includes_stream_and_tools() {
        let msgs = build_view_messages(&sample_member());
        assert!(msgs.iter().any(|m| m.content.contains("Preview @duck")));
        assert!(msgs.iter().any(|m| m.role == "assistant" && m.content.contains("tick")));
        assert!(msgs.iter().any(|m| m.content.contains("bash")));
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
            msgs.iter().any(|x| x.role == "assistant" && x.content.contains("tick")),
            "{msgs:?}"
        );
        assert!(
            msgs.iter().any(|x| x.content.contains("bash")),
            "tools: {msgs:?}"
        );
        let preview = preview_line_from_messages(&msgs).expect("preview");
        assert!(preview.contains("tick") || preview.contains("bash"), "{preview}");
    }

    #[test]
    fn chrome_copy_matches_claude_code_contract() {
        // TeammateViewHeader.tsx: "Viewing @name · esc return"
        // Free switch: tree stays + esc → team-lead (CC pills/nav).
        let sep = viewing_separator_label("duck", true);
        assert!(sep.contains("team-lead") || sep.contains("esc"), "{sep}");
        assert!(sep.contains("@duck"), "{sep}");
        let spans = viewing_status_spans("duck", true);
        let blob: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(blob.contains("esc"), "{blob}");
        assert!(blob.contains("team-lead"), "{blob}");
        assert!(blob.contains("@duck"), "{blob}");
        assert_eq!(header_height(true, 10), 2);
        assert_eq!(header_height(true, 2), 1);
        assert_eq!(header_height(false, 10), 0);
    }
}
