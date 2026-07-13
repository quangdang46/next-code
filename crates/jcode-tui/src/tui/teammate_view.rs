//! Soft teammate / swarm-agent view — Claude Code `enterTeammateView` parity.
//!
//! CC (claude-code-best):
//! - `enterTeammateView` sets `viewingAgentTaskId`
//! - `displayedMessages = task.messages` (full transcript takeover)
//! - `TeammateViewHeader` above messages
//! - typed input → `injectUserMessageToTeammate`
//! - Esc → `exitTeammateView`
//!
//! jcode swarm agents are separate remote sessions, so we:
//! 1. Soft-view: reconstruct a transcript from `SwarmMemberStatus` (output_tail,
//!    detail, todos) and swap it into the main messages area.
//! 2. Route typed input via `NotifySession` to that session_id.
//! 3. Optional hard-attach via `resume_session` (pop-out / Shift+Enter).

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
    } else {
        msgs.push(
            DisplayMessage::system(format!("status: {}", member.status))
                .with_title(format!("@{name}")),
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
    if let Some(tail) = member
        .output_tail
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        msgs.push(DisplayMessage::assistant(tail.to_string()));
    } else {
        msgs.push(DisplayMessage::system(
            "(no streamed output yet — waiting for agent activity…)",
        ));
    }

    msgs
}

/// Find a swarm member by session id.
pub fn find_member<'a>(
    members: &'a [SwarmMemberStatus],
    session_id: &str,
) -> Option<&'a SwarmMemberStatus> {
    members.iter().find(|m| m.session_id == session_id)
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

    let mut lines: Vec<Line<'static>> = vec![Line::from(vec![
        Span::styled("Viewing ", Style::default().fg(rgb_color(200, 200, 210))),
        Span::styled(
            format!("@{name}"),
            Style::default()
                .fg(name_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" · {} · esc return", member.status),
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
                "shift+enter hard-attach · type to message this agent",
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
            status_age_secs: None,
            output_tail: Some("line1\nline2".into()),
            report_back_to_session_id: None,
            todo_progress: Some((1, 3)),
            todo_items: vec![],
            runtime: Default::default(),
        }
    }

    #[test]
    fn build_view_messages_includes_stream_tail_and_header() {
        let msgs = build_view_messages(&sample_member());
        assert!(msgs.iter().any(|m| m.content.contains("Viewing @duck")));
        assert!(msgs.iter().any(|m| m.role == "assistant" && m.content.contains("line2")));
        assert!(msgs.iter().any(|m| m.content.contains("todos: 1/3") || m.content.contains("1/3")));
    }

    #[test]
    fn member_is_terminal_maps_cancelled() {
        let mut m = sample_member();
        m.status = "cancelled".into();
        assert!(member_is_terminal(&m));
        m.status = "running".into();
        assert!(!member_is_terminal(&m));
    }
}
