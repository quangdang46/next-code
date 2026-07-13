//! Agent tree data + rendering — Claude Code `TeammateSpinnerTree` parity.
//!
//! Reference (read these, don't invent UX):
//! - `/tmp/feature-research/claude-code/src/components/Spinner/TeammateSpinnerTree.tsx`
//! - `/tmp/feature-research/claude-code/src/components/Spinner/TeammateSpinnerLine.tsx`
//! - `/tmp/feature-research/claude-code/src/hooks/useBackgroundTaskNavigation.ts`
//! - `/tmp/feature-research/claude-code/src/state/teammateViewHelpers.ts`
//!
//! Claude Code rules:
//! 1. **Only render when `getRunningTeammatesSorted(tasks).length > 0`.**
//!    No subagents → tree is null. Normal single-agent turns never show a tree.
//! 2. **Flat list under fixed `team-lead`** (not nested spawn graphs).
//! 3. **Interactive selection** (`useBackgroundTaskNavigation`):
//!    - `Shift+↑/↓` steps selection: leader(-1) ↔ teammates(0..n-1) ↔ hide(n)
//!    - `Enter` on a teammate → `enterTeammateView` (switch transcript/view)
//!    - `Esc` exits selecting / viewing
//!    - Hint: `shift + ↑/↓ to select` / `enter to view`
//! 4. **Terminal members are evicted** — never sticky `@name: cancelled`.
//! 5. Live chrome with spinner (not a sticky transcript section).
//! 6. Alternative CC surface: footer **pills** when tree mode is off
//!    (`PromptInputFooterLeftSide` + `BackgroundTaskStatus`) — jcode maps this
//!    partially via swarm strip / running_items; tree is the spinner-tree mode.

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

/// Interactive selection state — mirrors CC `selectedIPAgentIndex` +
/// `viewSelectionMode === 'selecting-agent' | 'viewing-agent'`.
#[derive(Debug, Clone, Default)]
pub struct AgentTreeViewState {
    /// True while Shift+↑/↓ selection mode is active.
    pub selecting: bool,
    /// `-1` = leader, `0..n-1` = flat children (same order as `leader.children`).
    pub selected_index: i32,
    /// Session id of the teammate currently being viewed (if any).
    pub viewing_session_id: Option<String>,
}

/// Claude Code hint strings (TeammateSpinnerLine / teammateSelectHint.ts).
/// Use ASCII arrows — Unicode ↑↓ also tofu on some fonts.
pub const TEAMMATE_SELECT_HINT: &str = "shift+up/down to select";
/// CC Enter = enterTeammateView (soft transcript swap). Shift+Enter = hard session.
pub const TEAMMATE_VIEW_HINT: &str = "enter to view · shift+enter full session";
/// While already viewing: how to get back / free-switch.
pub const TEAMMATE_RETURN_HINT: &str = "esc → team-lead · shift+up/down switch";

/// Render the agent tree into a Vec of styled lines.
///
/// Empty input / no teammates → no lines (Claude Code returns null).
/// Exception: while *viewing* an agent the tree must stay painted so the user
/// can free-switch back to team-lead (CC keeps TeammateSpinnerTree / pills).
pub fn render(trees: &[AgentTreeNode], view: &AgentTreeViewState) -> Vec<Line<'static>> {
    if trees.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let viewing = view.viewing_session_id.is_some();

    for tree in trees {
        // Claude Code: if (teammateTasks.length === 0) return null — unless we
        // are mid-view and the builder left a switch roster in place.
        let has_child = !tree.children.is_empty();
        let has_running_child = tree.children.iter().any(AgentTreeNode::has_active_work);
        if !has_child {
            continue;
        }
        if !has_running_child && !viewing {
            continue;
        }
        // Flat CC tree: only paint depth-0 leader + depth-1 children for
        // selection indices. Nested grandchildren (if any) still recurse.
        render_node(tree, 0, true, true, -1, view, &mut lines);

        // CC hide row (index === teammateCount) — only in selection mode.
        // Hide is suppressed while hard/soft viewing so free-switch is simpler.
        if view.selecting && !viewing {
            let child_n = tree.children.len() as i32;
            let is_hide_selected = view.selected_index == child_n;
            let pointer = if is_hide_selected { ">" } else { " " };
            let mut spans = vec![
                Span::raw("  "),
                Span::styled(
                    format!("{pointer} "),
                    if is_hide_selected {
                        Style::default()
                            .fg(rgb_color(255, 220, 100))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
                Span::styled(
                    "└─ ",
                    Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
                ),
                Span::styled(
                    "hide",
                    if is_hide_selected {
                        Style::default()
                            .fg(rgb_color(255, 220, 100))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2))
                    },
                ),
            ];
            if is_hide_selected {
                spans.push(Span::styled(
                    " · enter to collapse",
                    Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    lines
}

/// Convenience for tests / non-interactive callers.
pub fn render_plain(trees: &[AgentTreeNode]) -> Vec<Line<'static>> {
    render(trees, &AgentTreeViewState::default())
}

/// Count selectable children under the first leader tree (for key navigation).
///
/// Uses **all** children the builder put in the tree. Do not re-filter by
/// `has_active_work` — hard-attach snapshots intentionally keep Idle roster
/// rows so free lead↔agent switch survives after `resume_session`.
pub fn selectable_child_count(trees: &[AgentTreeNode]) -> usize {
    trees.first().map(|t| t.children.len()).unwrap_or(0)
}

/// Session id of the child at flat index `idx` (0-based), if any.
pub fn child_session_id_at(trees: &[AgentTreeNode], idx: usize) -> Option<String> {
    let leader = trees.first()?;
    leader
        .children
        .get(idx)
        .and_then(|c| c.session_id.clone())
}

/// Display name of the child at flat index `idx`.
pub fn child_label_at(trees: &[AgentTreeNode], idx: usize) -> Option<String> {
    let leader = trees.first()?;
    leader.children.get(idx).map(|c| c.agent_name.clone())
}

fn render_node(
    node: &AgentTreeNode,
    depth: usize,
    is_last_sibling: bool,
    is_leader: bool,
    row_index: i32, // flat selection index: -1 leader, 0.. for children
    view: &AgentTreeViewState,
    out: &mut Vec<Line<'static>>,
) {
    let status_c = status_color(&node.status);
    let is_selected = view.selecting && view.selected_index == row_index;
    let is_viewing = view
        .viewing_session_id
        .as_ref()
        .zip(node.session_id.as_ref())
        .is_some_and(|(v, s)| v == s);
    let is_highlighted = is_selected || is_viewing || (is_leader && depth == 0 && view.viewing_session_id.is_none());

    let name_color = if is_selected {
        rgb_color(255, 220, 100) // selection highlight
    } else if is_leader {
        rgb_color(AGENT_TREE_COLOR.0, AGENT_TREE_COLOR.1, AGENT_TREE_COLOR.2)
    } else {
        let idx = depth.saturating_sub(1).min(AGENT_CHILD_COLORS.len() - 1);
        let c = AGENT_CHILD_COLORS[idx];
        rgb_color(c.0, c.1, c.2)
    };

    // Single-line box drawing only. Double-line forms (╒═ ╘═ ╞═) render as
    // empty tofu □ in many terminal fonts — the user's screenshot showed blank
    // boxes before "team-lead" / "@pig". CC uses double-line; we prefer
    // reliability over glyph fidelity until a font-probe exists.
    let tree_char = if depth == 0 {
        "┌─ "
    } else if is_last_sibling {
        "└─ "
    } else {
        "├─ "
    };

    let display_name = if is_leader && depth == 0 {
        node.agent_name.clone()
    } else if node.agent_name.starts_with('@') {
        node.agent_name.clone()
    } else {
        format!("@{}", node.agent_name)
    };

    // Selection pointer: plain ASCII `>` — always visible (Unicode › often tofu).
    let pointer = if is_selected { ">" } else { " " };
    let mut spans: Vec<Span<'static>> = vec![
        Span::raw("  ".repeat(depth.max(1))),
        Span::styled(
            format!("{pointer} "),
            if is_selected {
                Style::default()
                    .fg(rgb_color(255, 220, 100))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ),
        Span::styled(
            tree_char,
            Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
        ),
        Span::styled(
            display_name,
            Style::default()
                .fg(name_color)
                .add_modifier(if is_highlighted {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
    ];

    // Leader activity only when backgrounded (viewing a teammate).
    let leader_backgrounded = is_leader && depth == 0 && view.viewing_session_id.is_some();
    let show_activity = if is_leader && depth == 0 {
        leader_backgrounded
    } else {
        // When a child is selected/highlighted, CC hides its activity text
        // (main spinner already shows the verb). Keep activity when not selected.
        !is_selected
    };

    let activity = if show_activity {
        node.activity
            .as_deref()
            .map(str::trim)
            .filter(|s| is_meaningful_activity(s))
            .map(ToString::to_string)
            .or_else(|| {
                if is_leader && depth == 0 {
                    Some("processing…".to_string())
                } else {
                    node.status
                        .activity_fallback()
                        .map(ToString::to_string)
                }
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

    // Hints — TeammateSpinnerLine / free-switch chrome
    if is_viewing {
        spans.push(Span::styled(
            " · viewing",
            Style::default()
                .fg(rgb_color(255, 220, 100))
                .add_modifier(Modifier::BOLD),
        ));
    }
    if is_leader && depth == 0 && view.viewing_session_id.is_some() {
        // Always label the path home on the team-lead row while viewing.
        spans.push(Span::styled(
            format!(" · {TEAMMATE_RETURN_HINT}"),
            Style::default().fg(rgb_color(255, 220, 100)),
        ));
    } else if is_highlighted && !view.selecting && view.viewing_session_id.is_none() {
        spans.push(Span::styled(
            format!(" · {TEAMMATE_SELECT_HINT}"),
            Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
        ));
    }
    if is_selected && !is_leader && view.viewing_session_id.is_none() {
        spans.push(Span::styled(
            format!(" · {TEAMMATE_VIEW_HINT}"),
            Style::default().fg(rgb_color(DIM_COLOR.0, DIM_COLOR.1, DIM_COLOR.2)),
        ));
    }
    if is_selected && is_leader && depth == 0 && view.viewing_session_id.is_some() {
        spans.push(Span::styled(
            " · enter to return",
            Style::default()
                .fg(rgb_color(255, 220, 100))
                .add_modifier(Modifier::BOLD),
        ));
    }

    out.push(Line::from(spans));

    // Flat children: selection indices 0..n-1 in tree order (all children).
    // Builder already decided who is in the tree — do not re-filter by status
    // or Idle rows lose indices and free-switch breaks.
    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i + 1 == child_count;
        let child_row = if depth == 0 { i as i32 } else { -99 };
        render_node(
            child,
            depth + 1,
            child_is_last,
            false,
            child_row,
            view,
            out,
        );
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
        child_with_sid(name, status, activity, None)
    }

    fn child_with_sid(
        name: &str,
        status: AgentStatus,
        activity: Option<&str>,
        session_id: Option<&str>,
    ) -> AgentTreeNode {
        AgentTreeNode {
            agent_name: name.to_string(),
            status,
            tool_use_count: 0,
            token_count: 0,
            is_leaf: true,
            is_leader: false,
            children: Vec::new(),
            session_id: session_id.map(ToString::to_string),
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
        let lines = render_plain(&[tree]);
        assert!(
            lines.is_empty(),
            "Claude Code hides the tree when no teammates are running; got: {:?}",
            lines.iter().map(line_text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn render_hides_lone_leader_with_no_children() {
        let tree = leader(vec![], true);
        let lines = render_plain(&[tree]);
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
        let lines = render_plain(&[tree]);
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
        // Hint for shift selection when not actively selecting.
        assert!(
            leader_line.contains(TEAMMATE_SELECT_HINT),
            "expected select hint: {leader_line}"
        );
    }

    #[test]
    fn render_selection_shows_pointer_and_view_hint() {
        let tree = leader(
            vec![child("dragon", AgentStatus::Running, Some("processing…"))],
            true,
        );
        let view = AgentTreeViewState {
            selecting: true,
            selected_index: 0,
            viewing_session_id: None,
        };
        let texts: Vec<String> = render(&[tree], &view).iter().map(line_text).collect();
        let child = texts.iter().find(|t| t.contains("@dragon")).unwrap();
        assert!(
            child.contains('>') || child.starts_with(">") || child.contains("> "),
            "pointer missing: {child}"
        );
        assert!(
            child.contains(TEAMMATE_VIEW_HINT),
            "enter-to-view hint missing: {child}"
        );
        // Single-line tree chars must appear (not empty tofu double-line).
        let leader = texts.iter().find(|t| t.contains("team-lead")).unwrap();
        assert!(
            leader.contains("┌─") || leader.contains("┌"),
            "leader tree glyph missing: {leader}"
        );
    }

    #[test]
    fn free_switch_while_viewing_keeps_tree_and_return_path() {
        // Hard-attach snapshot: one Idle + one viewing Running — both must be
        // selectable so Shift+↑ can reach team-lead and Enter returns home.
        let tree = leader(
            vec![
                child_with_sid("duck", AgentStatus::Running, Some("viewing"), Some("ses_duck")),
                child_with_sid("pig", AgentStatus::Idle, Some("idle"), Some("ses_pig")),
            ],
            false,
        );
        let view = AgentTreeViewState {
            selecting: true,
            selected_index: -1, // team-lead selected → path home
            viewing_session_id: Some("ses_duck".into()),
        };
        let trees = vec![tree];
        assert_eq!(
            selectable_child_count(&trees),
            2,
            "Idle roster rows must stay selectable for free switch"
        );
        assert_eq!(
            child_session_id_at(&trees, 0).as_deref(),
            Some("ses_duck")
        );
        assert_eq!(child_session_id_at(&trees, 1).as_deref(), Some("ses_pig"));
        let texts: Vec<String> = render(&trees, &view).iter().map(line_text).collect();
        assert!(
            !texts.is_empty(),
            "tree must stay visible while viewing: {texts:?}"
        );
        let lead = texts.iter().find(|t| t.contains("team-lead")).expect("lead");
        assert!(
            lead.contains(TEAMMATE_RETURN_HINT) || lead.contains("esc"),
            "team-lead must show return path while viewing: {lead}"
        );
        assert!(
            lead.contains("enter to return") || lead.contains('>'),
            "selected team-lead must be obvious: {lead}"
        );
        let duck = texts.iter().find(|t| t.contains("@duck")).expect("duck");
        assert!(
            duck.contains("viewing"),
            "viewed agent must be marked: {duck}"
        );
        // No hide row while viewing.
        assert!(
            !texts.iter().any(|t| t.contains("hide")),
            "hide row suppressed while viewing: {texts:?}"
        );
    }

    #[test]
    fn selection_indices_are_dense_for_idle_and_running() {
        // Regression: previously Idle children shared flat indices with Running
        // (flat_idx only advanced on has_active_work) → broken free switch.
        let tree = leader(
            vec![
                child_with_sid("a", AgentStatus::Idle, None, Some("ses_a")),
                child_with_sid("b", AgentStatus::Running, Some("work"), Some("ses_b")),
            ],
            true,
        );
        let trees = vec![tree];
        assert_eq!(selectable_child_count(&trees), 2);
        assert_eq!(child_label_at(&trees, 0).as_deref(), Some("a"));
        assert_eq!(child_label_at(&trees, 1).as_deref(), Some("b"));
        assert_eq!(child_session_id_at(&trees, 0).as_deref(), Some("ses_a"));
        assert_eq!(child_session_id_at(&trees, 1).as_deref(), Some("ses_b"));
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
        let texts: Vec<String> = render_plain(&[tree]).iter().map(line_text).collect();
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
        let texts: Vec<String> = render_plain(&[tree]).iter().map(line_text).collect();
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
        let texts: Vec<String> = render_plain(&[tree]).iter().map(line_text).collect();
        assert!(
            texts.iter().any(|t| t.contains("@leaf")),
            "nested grandchild not rendered: {texts:?}"
        );
    }
}
