//! Agent tree data + rendering — Claude Code `TeammateSpinnerTree` parity.
//!
//! Reference: `claude-code` `TeammateSpinnerTree` / `TeammateSpinnerLine` /
//! `AgentProgressLine` (see `/tmp/feature-research/claude-code/src/components/Spinner/`).
//!
//! Rules (match Claude Code):
//! 1. **Only render when there is at least one running teammate / subagent.**
//!    `if (teammateTasks.length === 0) return null`.
//! 2. **Flat list under a fixed leader** (`team-lead`) — not nested spawn trees,
//!    and never glued under a user prompt in the transcript.
//! 3. **Terminal members are evicted** (completed / failed / cancelled) — never
//!    leave `@name: cancelled` under an Interrupted banner.
//! 4. **Activity is a real verb/description** — never a bare number like `2`
//!    (todo counters belong in dim stats: ` · 2/5`).
//! 5. Live chrome only (conversation activity area), not a sticky chat section.

use ratatui::prelude::*;
use crate::tui::color_support::rgb as rgb_color;

/// Status of an agent in the agent tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Running,
    Completed,
    Failed,
    Stopped,
    Idle,
}

impl AgentStatus {
    /// Terminal = finished work; Claude Code evicts these from the live tree.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AgentStatus::Completed | AgentStatus::Failed | AgentStatus::Stopped
        )
    }

    pub fn from_swarm_status(status: &str) -> Self {
        match status.trim().to_ascii_lowercase().as_str() {
            "running" | "processing" | "active" | "starting" => AgentStatus::Running,
            // "ready" is waiting, not actively working — keep visible as idle.
            "ready" | "idle" => AgentStatus::Idle,
            "completed" | "done" | "ok" | "success" => AgentStatus::Completed,
            "failed" | "error" | "crashed" => AgentStatus::Failed,
            "stopped" | "cancelled" | "canceled" | "interrupted" | "killed" => {
                AgentStatus::Stopped
            }
            // Unknown statuses must NOT default to Running (would never prune).
            _ => AgentStatus::Idle,
        }
    }

    pub fn activity_fallback(&self) -> Option<&'static str> {
        match self {
            // Prefer a verb that matches swarm status language when we have no
            // richer task_label/detail/output_tail.
            AgentStatus::Running => Some("working…"),
            AgentStatus::Completed => Some("done"),
            AgentStatus::Failed => Some("failed"),
            AgentStatus::Stopped => Some("cancelled"),
            AgentStatus::Idle => Some("idle"),
        }
    }

    /// Prefer the raw swarm status string when it is a human verb
    /// (e.g. "processing" → "processing…"), else fall back.
    pub fn activity_from_raw_status(raw: &str) -> Option<String> {
        let t = raw.trim().to_ascii_lowercase();
        match t.as_str() {
            "processing" | "running" | "active" | "starting" | "connecting" | "thinking"
            | "searching" | "editing" | "reading" | "writing" => {
                Some(format!("{t}…"))
            }
            _ => None,
        }
    }
}

/// A single node in the agent tree (can have children).
#[derive(Debug, Clone)]
pub struct AgentTreeNode {
    pub agent_name: String,
    pub status: AgentStatus,
    pub tool_use_count: u32,
    pub token_count: u64,
    pub is_leaf: bool,
    pub is_leader: bool,
    pub children: Vec<AgentTreeNode>,
    pub session_id: Option<String>,
    pub activity: Option<String>,
    /// Optional todo progress `(done, total)` shown as dim ` · d/t`.
    pub todo_progress: Option<(u32, u32)>,
}

impl AgentTreeNode {
    /// True when this node or any descendant is actively running.
    pub fn has_active_work(&self) -> bool {
        matches!(self.status, AgentStatus::Running)
            || self.children.iter().any(AgentTreeNode::has_active_work)
    }

    /// Drop terminal children. Claude Code only lists running teammates.
    /// Also drops pure-idle leaves when the tree is used as a *live spinner*
    /// (idle waiting peers can stay if you want a roster — we keep Idle so a
    /// ready teammate remains visible while others work).
    pub fn prune_terminal_leaves(&mut self) {
        self.children.retain(|child| {
            if child.has_active_work() {
                return true;
            }
            !child.status.is_terminal()
        });
        for child in &mut self.children {
            child.prune_terminal_leaves();
        }
        self.is_leaf = self.children.is_empty();
    }

    /// Live spinner tree: keep only running (and nested active) children.
    /// Matches `getRunningTeammatesSorted` — idle/terminal peers are hidden.
    pub fn keep_running_children_only(&mut self) {
        self.children.retain(|child| child.has_active_work());
        for child in &mut self.children {
            child.keep_running_children_only();
        }
        self.is_leaf = self.children.is_empty();
    }
}

/// Colors for agent tree rendering.
const AGENT_TREE_COLOR: (u8, u8, u8) = (120, 180, 255);
const AGENT_CHILD_COLORS: &[(u8, u8, u8)] = &[
    (80, 220, 100),
    (255, 180, 80),
    (200, 140, 255),
    (80, 200, 230),
    (255, 150, 150),
];
const DIM_COLOR: (u8, u8, u8) = (100, 100, 110);
const ERROR_COLOR: (u8, u8, u8) = (255, 100, 100);
const SUCCESS_COLOR: (u8, u8, u8) = (100, 180, 100);

/// True when a status string is usable as on-screen activity (not a counter).
///
/// Rejects bare numbers like `"2"` / `"2/5"` that were showing up as
/// `@butterfly: 2` in the live tree (todo progress misused as activity).
pub fn is_meaningful_activity(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() || t.len() < 2 {
        return false;
    }
    // Pure numeric / fraction / percent counters belong in stats, not `: activity`.
    if t
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '/' | '.' | '%' | ' '))
    {
        return false;
    }
    true
}

/// Pick the best human activity string from swarm member fields.
pub fn pick_member_activity(
    task_label: Option<&str>,
    detail: Option<&str>,
    output_tail: Option<&str>,
    status: &AgentStatus,
    raw_status: Option<&str>,
) -> Option<String> {
    for candidate in [task_label, detail] {
        if let Some(s) = candidate.map(str::trim).filter(|s| is_meaningful_activity(s)) {
            return Some(s.to_string());
        }
    }
    if let Some(tail) = output_tail {
        if let Some(line) = tail
            .lines()
            .rev()
            .map(str::trim)
            .find(|l| is_meaningful_activity(l))
        {
            // Keep activity short for the spinner row.
            let truncated = if line.chars().count() > 48 {
                let mut out: String = line.chars().take(47).collect();
                out.push('…');
                out
            } else {
                line.to_string()
            };
            return Some(truncated);
        }
    }
    if let Some(raw) = raw_status {
        if let Some(from_raw) = AgentStatus::activity_from_raw_status(raw) {
            return Some(from_raw);
        }
    }
    status.activity_fallback().map(ToString::to_string)
}

/// Render the agent tree into a Vec of styled lines.
///
/// Empty input / no running teammates → no lines (Claude Code returns null).
pub fn render(trees: &[AgentTreeNode]) -> Vec<Line<'static>> {
    if trees.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();

    for tree in trees {
        // Claude Code: if (teammateTasks.length === 0) return null.
        // A lone leader with no running children is not a spinner tree.
        let has_running_child = tree.children.iter().any(AgentTreeNode::has_active_work);
        if !has_running_child {
            continue;
        }
        render_node(tree, 0, true, tree.is_leader, &mut lines);
    }

    lines
}

fn render_node(
    node: &AgentTreeNode,
    depth: usize,
    is_last_sibling: bool,
    is_leader: bool,
    out: &mut Vec<Line<'static>>,
) {
    let status_c = status_color(&node.status);
    let name_color = if is_leader {
        rgb_color(AGENT_TREE_COLOR.0, AGENT_TREE_COLOR.1, AGENT_TREE_COLOR.2)
    } else {
        let idx = depth.saturating_sub(1).min(AGENT_CHILD_COLORS.len() - 1);
        let c = AGENT_CHILD_COLORS[idx];
        rgb_color(c.0, c.1, c.2)
    };

    // Claude Code: leader uses ┌─ / ╒═ when highlighted (running); children ├─ / └─.
    // Keep single-cell-friendly glyphs (box-drawing width 1 each).
    let tree_char = if depth == 0 {
        if is_leader && matches!(node.status, AgentStatus::Running) {
            "╒═ "
        } else {
            "┌─ "
        }
    } else if is_last_sibling {
        "└─ "
    } else {
        "├─ "
    };

    let display_name = if is_leader && depth == 0 {
        // Fixed identity like Claude Code "team-lead" — never session titles.
        node.agent_name.clone()
    } else if node.agent_name.starts_with('@') {
        node.agent_name.clone()
    } else {
        format!("@{}", node.agent_name)
    };

    let mut spans: Vec<Span<'static>> = vec![
        // Claude Code pads with ~3 spaces before the tree char.
        Span::raw("  ".repeat(depth + 1)),
        Span::styled(
            tree_char,
            Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
        ),
        Span::styled(
            display_name,
            Style::default()
                .fg(name_color)
                .add_modifier(if is_leader {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
    ];

    // Claude Code TeammateSpinnerTree: leader activity/verb is ONLY shown when
    // the leader is *backgrounded* (viewing a teammate). While the main
    // session is foregrounded, leader is just `╒═ team-lead` — the spinner
    // line above already carries connecting/thinking/streaming. Showing
    // `team-lead: processing…` next to `connecting… 7s` is redundant noise.
    let show_activity = !(is_leader && depth == 0);
    let activity = if show_activity {
        node.activity
            .as_deref()
            .map(str::trim)
            .filter(|s| is_meaningful_activity(s))
            .map(ToString::to_string)
            .or_else(|| {
                node.status
                    .activity_fallback()
                    .map(ToString::to_string)
            })
    } else {
        None
    };

    if let Some(activity) = activity {
        spans.push(Span::styled(
            format!(": {activity}"),
            Style::default().fg(status_c),
        ));
    }

    if node.tool_use_count > 0 {
        spans.push(Span::styled(
            format!(
                " · {} tool {}",
                node.tool_use_count,
                if node.tool_use_count == 1 {
                    "use"
                } else {
                    "uses"
                }
            ),
            Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
        ));
    }
    if node.token_count > 0 {
        spans.push(Span::styled(
            format!(" · {} tokens", node.token_count),
            Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
        ));
    }
    if let Some((done, total)) = node.todo_progress {
        if total > 0 {
            spans.push(Span::styled(
                format!(" · {done}/{total}"),
                Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
            ));
        }
    }

    out.push(Line::from(spans));

    // Flat teammates preferred; recursion kept for intentional nested nodes.
    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i + 1 == child_count;
        render_node(child, depth + 1, child_is_last, false, out);
    }
}

fn status_color(status: &AgentStatus) -> Color {
    match status {
        AgentStatus::Running => Color::Rgb(200, 200, 210),
        AgentStatus::Completed => Color::Rgb(SUCCESS_COLOR.0, SUCCESS_COLOR.1, SUCCESS_COLOR.2),
        AgentStatus::Failed => Color::Rgb(ERROR_COLOR.0, ERROR_COLOR.1, ERROR_COLOR.2),
        AgentStatus::Stopped => Color::Rgb(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2),
        AgentStatus::Idle => Color::Rgb(150, 150, 160),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn child(name: &str, status: AgentStatus, activity: Option<&str>) -> AgentTreeNode {
        AgentTreeNode {
            agent_name: name.to_string(),
            status,
            tool_use_count: 0,
            token_count: 0,
            is_leaf: true,
            is_leader: false,
            children: Vec::new(),
            session_id: None,
            activity: activity.map(ToString::to_string),
            todo_progress: None,
        }
    }

    fn leader(children: Vec<AgentTreeNode>, running: bool) -> AgentTreeNode {
        AgentTreeNode {
            agent_name: "team-lead".to_string(),
            status: if running {
                AgentStatus::Running
            } else {
                AgentStatus::Idle
            },
            tool_use_count: 0,
            token_count: 0,
            is_leaf: children.is_empty(),
            is_leader: true,
            children,
            session_id: None,
            // Foreground leader: name only (spinner line owns the verb).
            activity: None,
            todo_progress: None,
        }
    }

    fn line_text(line: &Line<'_>) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn render_hides_tree_with_only_cancelled_children() {
        let mut tree = leader(
            vec![child("badger", AgentStatus::Stopped, Some("cancelled"))],
            false,
        );
        tree.prune_terminal_leaves();
        tree.keep_running_children_only();
        let lines = render(&[tree]);
        assert!(
            lines.is_empty(),
            "Claude Code hides the tree when no teammates are running; got: {:?}",
            lines.iter().map(line_text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn render_hides_lone_leader_with_no_children() {
        let tree = leader(vec![], true);
        let lines = render(&[tree]);
        assert!(
            lines.is_empty(),
            "no teammates → null (CC TeammateSpinnerTree); got: {:?}",
            lines.iter().map(line_text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn render_shows_running_child_with_at_prefix() {
        let tree = leader(
            vec![child("badger", AgentStatus::Running, Some("searching…"))],
            true,
        );
        let lines = render(&[tree]);
        let texts: Vec<String> = lines.iter().map(line_text).collect();
        assert!(
            texts.iter().any(|t| t.contains("team-lead")),
            "leader missing: {texts:?}"
        );
        assert!(
            texts
                .iter()
                .any(|t| t.contains("@badger") && t.contains("searching")),
            "child missing: {texts:?}"
        );
        // Leader must NOT carry ": processing…" while foregrounded (CC parity).
        let leader_line = texts.iter().find(|t| t.contains("team-lead")).unwrap();
        assert!(
            !leader_line.contains(": processing") && !leader_line.contains(": working"),
            "leader should be name-only when foregrounded: {leader_line}"
        );
    }

    #[test]
    fn pick_member_activity_uses_raw_processing_status() {
        let activity = pick_member_activity(None, None, None, &AgentStatus::Running, Some("processing"));
        assert_eq!(activity.as_deref(), Some("processing…"));
    }

    #[test]
    fn pick_member_activity_skips_numeric_junk() {
        let activity = pick_member_activity(
            Some("2"),
            Some("2/5"),
            Some("3\n"),
            &AgentStatus::Running,
            None,
        );
        assert_eq!(activity.as_deref(), Some("working…"));
    }

    #[test]
    fn render_flat_siblings_not_nested() {
        let tree = leader(
            vec![
                child("chick", AgentStatus::Running, Some("processing…")),
                child("butterfly", AgentStatus::Running, Some("editing…")),
            ],
            true,
        );
        let texts: Vec<String> = render(&[tree]).iter().map(line_text).collect();
        assert_eq!(texts.len(), 3, "leader + 2 flat children: {texts:?}");
        // Both children at same indent depth (one leading indent block beyond leader).
        let chick = texts.iter().find(|t| t.contains("@chick")).unwrap();
        let butterfly = texts.iter().find(|t| t.contains("@butterfly")).unwrap();
        let chick_indent = chick.chars().take_while(|c| *c == ' ').count();
        let butterfly_indent = butterfly.chars().take_while(|c| *c == ' ').count();
        assert_eq!(
            chick_indent, butterfly_indent,
            "siblings must be flat (same indent), got {texts:?}"
        );
    }

    #[test]
    fn meaningful_activity_rejects_bare_numbers() {
        assert!(!is_meaningful_activity("2"));
        assert!(!is_meaningful_activity("2/5"));
        assert!(!is_meaningful_activity("  "));
        assert!(is_meaningful_activity("processing…"));
        assert!(is_meaningful_activity("searching files"));
    }

    #[test]
    fn render_shows_todo_as_stats_not_activity() {
        let mut node = child("butterfly", AgentStatus::Running, None);
        node.todo_progress = Some((2, 5));
        let tree = leader(vec![node], true);
        let texts: Vec<String> = render(&[tree]).iter().map(line_text).collect();
        let line = texts.iter().find(|t| t.contains("@butterfly")).unwrap();
        assert!(
            line.contains("· 2/5"),
            "todo should be dim stats: {line}"
        );
        assert!(
            !line.contains(": 2") && !line.contains(": 2/5"),
            "todo must not be the activity: {line}"
        );
    }

    #[test]
    fn from_swarm_status_maps_cancelled_to_stopped() {
        assert_eq!(
            AgentStatus::from_swarm_status("cancelled"),
            AgentStatus::Stopped
        );
        assert_eq!(
            AgentStatus::from_swarm_status("running"),
            AgentStatus::Running
        );
        assert_eq!(
            AgentStatus::from_swarm_status("mysterious"),
            AgentStatus::Idle
        );
    }

    #[test]
    fn prune_drops_terminal_keeps_running() {
        let mut tree = leader(
            vec![
                child("done", AgentStatus::Completed, Some("done")),
                child("live", AgentStatus::Running, Some("work")),
                child("dead", AgentStatus::Stopped, Some("cancelled")),
            ],
            true,
        );
        tree.prune_terminal_leaves();
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].agent_name, "live");
    }

    #[test]
    fn render_is_recursive_for_nested_children() {
        let mut nested = child("worker", AgentStatus::Running, Some("editing"));
        nested.is_leaf = false;
        nested.children = vec![child("leaf", AgentStatus::Running, Some("read"))];
        let tree = leader(vec![nested], true);
        let texts: Vec<String> = render(&[tree]).iter().map(line_text).collect();
        assert!(
            texts.iter().any(|t| t.contains("@leaf")),
            "nested grandchild not rendered: {texts:?}"
        );
    }
}
