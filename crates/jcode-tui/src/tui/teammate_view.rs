//! Soft teammate / swarm-agent view — Claude Code `enterTeammateView` parity.
//!
//! CC (claude-code-best):
//! - `enterTeammateView` sets `viewingAgentTaskId`
//! - `displayedMessages = task.messages` (full transcript takeover)
//! - `TeammateViewHeader` above messages
//! - typed input → `injectUserMessageToTeammate`
//! - Esc → `exitTeammateView` (or abort turn if still running)
//!
//! jcode swarm agents are separate remote sessions, so we:
//! 1. Soft-view: reconstruct a transcript from `SwarmMemberStatus` (output_tail,
//!    detail, todos, tool_intents, runtime) and swap main messages.
//! 2. Route typed input via `CommMessage` DM (fallback `NotifySession`).
//! 3. Kill via `CommStop`; optional hard-attach via `resume_session`.

use jcode_tui_messages::DisplayMessage;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use crate::protocol::SwarmMemberStatus;
use crate::tui::color_support::rgb as rgb_color;

/// Build Claude-Code-style transcript messages for a swarm member soft-view.
pub fn build_view_messages(member: &SwarmMemberStatus) -> Vec<DisplayMessage> {
    let name = member
        .friendly_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("agent");

    let mut msgs = Vec::new();

    // Header-equivalent as system banner (CC TeammateViewHeader content).
    let mut header = format!("Viewing @{name}");
    if let Some(task) = member
        .task_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        header.push_str(&format!("\n{task}"));
    }
    let mut meta_bits: Vec<String> = Vec::new();
    meta_bits.push(member.status.clone());
    if let Some(model) = member.runtime.model.as_deref().filter(|s| !s.is_empty()) {
        meta_bits.push(model.to_string());
    }
    if let Some(secs) = member.runtime.elapsed_secs {
        meta_bits.push(format_elapsed(secs));
    }
    if let Some(age) = member.status_age_secs {
        meta_bits.push(format!("updated {age}s ago"));
    }
    header.push_str(&format!("\n{}", meta_bits.join(" · ")));
    header.push_str("\nesc return · shift+enter hard-attach · k kill (select mode)");
    msgs.push(DisplayMessage::system(header));

    if let Some(detail) = member
        .detail
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        msgs.push(
            DisplayMessage::system(format!("status: {} — {detail}", member.status))
                .with_title(format!("@{name}")),
        );
    }

    // Tool activity from todo intents (closest thing to CC tool_use lines).
    let tool_lines = collect_tool_activity_lines(member);
    if !tool_lines.is_empty() {
        msgs.push(
            DisplayMessage::system(tool_lines.join("\n")).with_title("tools"),
        );
    }

    // Todo snapshot as a dedicated card when present.
    if !member.todo_items.is_empty() {
        let mut lines = Vec::new();
        for t in &member.todo_items {
            let mark = match t.status.to_ascii_lowercase().as_str() {
                "completed" | "done" => "✓",
                "in_progress" | "running" => "…",
                "cancelled" | "canceled" => "○",
                _ => "·",
            };
            lines.push(format!("{mark} {}", t.content));
            for tool in &t.tool_intents {
                let st = match tool.status.to_ascii_lowercase().as_str() {
                    "running" => "…",
                    "completed" | "done" => "✓",
                    "error" | "failed" => "✗",
                    _ => "·",
                };
                let mut line = format!("    {st} {} — {}", tool.tool_name, tool.intent);
                if let Some(p) = &tool.progress {
                    let unit = p.unit.as_deref().unwrap_or("");
                    line.push_str(&format!(" ({}/{}{})", p.current, p.total, unit));
                }
                lines.push(line);
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

    // Live stream tail → assistant message (CC task.messages mirror).
    // Split into chunks so long tails read like multi-turn assistant output.
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
        msgs.push(DisplayMessage::system(
            "(no streamed output yet — waiting for agent activity…)",
        ));
    }

    msgs
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
            let mut line = format!("├─ {} ({st}): {}", tool.tool_name, tool.intent);
            if let Some(p) = &tool.progress {
                let unit = p.unit.as_deref().unwrap_or("");
                line.push_str(&format!(" · {}/{}{}", p.current, p.total, unit));
            }
            lines.push(line);
        }
    }
    // Cap display noise.
    if lines.len() > 12 {
        let omitted = lines.len() - 12;
        lines.truncate(12);
        lines.push(format!("└─ … +{omitted} more tool events"));
    } else if let Some(last) = lines.last_mut() {
        *last = last.replacen("├─", "└─", 1);
    }
    lines
}

/// Split a long output_tail into readable assistant chunks (paragraphs / ~40 lines).
fn split_stream_chunks(tail: &str) -> Vec<String> {
    let paragraphs: Vec<&str> = tail
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if paragraphs.len() > 1 {
        return paragraphs
            .into_iter()
            .map(|p| p.to_string())
            .collect();
    }
    let lines: Vec<&str> = tail.lines().collect();
    if lines.len() <= 40 {
        return vec![tail.to_string()];
    }
    let mut out = Vec::new();
    for chunk in lines.chunks(40) {
        out.push(chunk.join("\n"));
    }
    out
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

/// Stats string for tree child line (CC: tool uses · tokens).
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
        // Keep short model id tail.
        let short = model.rsplit('/').next().unwrap_or(model);
        let short = if short.len() > 16 {
            format!("{}…", &short[..14])
        } else {
            short.to_string()
        };
        parts.push(short);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

/// Find a swarm member by session id.
pub fn find_member<'a>(
    members: &'a [SwarmMemberStatus],
    session_id: &str,
) -> Option<&'a SwarmMemberStatus> {
    members.iter().find(|m| m.session_id == session_id)
}

/// True when member is actively running (Esc should abort, not only exit).
pub fn member_is_running(member: &SwarmMemberStatus) -> bool {
    matches!(
        member.status.trim().to_ascii_lowercase().as_str(),
        "running" | "processing" | "active" | "starting" | "ready"
    )
}

/// True when member is in a terminal lifecycle state (CC auto-exit).
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

/// Draw full-width Viewing header above the messages area (CC TeammateViewHeader).
pub fn draw_header(frame: &mut Frame, area: Rect, member: &SwarmMemberStatus) {
    if area.height == 0 || area.width < 10 {
        return;
    }
    let name = member
        .friendly_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("agent");
    let name_color = rgb_color(80, 220, 100);
    let dim = Style::default().fg(rgb_color(100, 100, 110));

    let mut status_bits = vec![member.status.clone()];
    if let Some(secs) = member.runtime.elapsed_secs {
        status_bits.push(format_elapsed(secs));
    }

    let mut lines: Vec<Line<'static>> = vec![Line::from(vec![
        Span::styled("Viewing ", Style::default().fg(rgb_color(200, 200, 210))),
        Span::styled(
            format!("@{name}"),
            Style::default()
                .fg(name_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" · {} · esc return", status_bits.join(" · ")),
            dim,
        ),
    ])];

    if area.height >= 2 {
        if let Some(task) = member
            .task_label
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let max = area.width.saturating_sub(2) as usize;
            let t = if task.chars().count() > max {
                format!(
                    "{}…",
                    task.chars().take(max.saturating_sub(1)).collect::<String>()
                )
            } else {
                task.to_string()
            };
            lines.push(Line::from(Span::styled(t, dim)));
        } else {
            lines.push(Line::from(Span::styled(
                "type to message · shift+enter hard-attach · esc exits view",
                dim,
            )));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}

/// Height reserved for the viewing header (0 when not viewing).
pub fn header_height(viewing: bool) -> u16 {
    if viewing {
        2
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
            detail: Some("working".into()),
            task_label: Some("In ra 3 dong".into()),
            role: None,
            is_headless: Some(true),
            live_attachments: None,
            status_age_secs: Some(3),
            output_tail: Some("line1\n\nline2 para".into()),
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
    fn build_view_messages_includes_stream_tail_tools_and_runtime() {
        let msgs = build_view_messages(&sample_member());
        assert!(msgs.iter().any(|m| m.content.contains("Viewing @duck")));
        assert!(msgs.iter().any(|m| m.content.contains("gpt-4.1") || m.content.contains("42s")));
        assert!(msgs.iter().any(|m| m.content.contains("bash") && m.content.contains("hostname")));
        assert!(msgs.iter().any(|m| m.role == "assistant"));
        // paragraph split
        assert!(
            msgs.iter().filter(|m| m.role == "assistant").count() >= 1
        );
    }

    #[test]
    fn member_tree_stats_shows_tools_and_elapsed() {
        let s = member_tree_stats(&sample_member()).expect("stats");
        assert!(s.contains("tool"));
        assert!(s.contains("42s") || s.contains("gpt"));
    }

    #[test]
    fn member_is_terminal_maps_cancelled() {
        let mut m = sample_member();
        m.status = "cancelled".into();
        assert!(member_is_terminal(&m));
        m.status = "running".into();
        assert!(!member_is_terminal(&m));
        assert!(member_is_running(&m));
    }
}
