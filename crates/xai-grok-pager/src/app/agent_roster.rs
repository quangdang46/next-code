//! Unified agent-team roster for Face (Claude Code–style panel).
//!
//! Merges Grok `subagent_sessions` with optional swarm members into one
//! selectable row list. Pure data — no I/O.

use std::collections::HashMap;
use std::time::Instant;

use crate::app::subagent::SubagentInfo;

/// Where a roster row came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterSource {
    Lead,
    GrokSubagent,
    SwarmMember,
}

/// Lifecycle chip for a roster row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RosterStatus {
    Lead,
    Running,
    Idle,
    NeedsInput,
    Completed,
    Failed,
    Cancelled,
    PendingKill,
}

impl RosterStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lead => "lead",
            Self::Running => "running",
            Self::Idle => "idle",
            Self::NeedsInput => "needs input",
            Self::Completed => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::PendingKill => "stopping",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Running | Self::NeedsInput | Self::PendingKill | Self::Idle
        )
    }
}

/// One row in the under-prompt agent panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRosterRow {
    /// Child session id, swarm session id, or `"__lead__"`.
    pub id: String,
    pub display_name: String,
    pub source: RosterSource,
    pub status: RosterStatus,
    pub activity: Option<String>,
    /// Stable key for color-by-name.
    pub color_key: String,
    /// Grok subagent id for kill, when applicable.
    pub kill_subagent_id: Option<String>,
    pub can_message: bool,
    pub can_open_transcript: bool,
    pub todo_progress: Option<(u32, u32)>,
    pub is_lead: bool,
}

/// Swarm member mirrored into Face (protocol / ACP bridge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmMemberMirror {
    pub session_id: String,
    pub friendly_name: Option<String>,
    pub status: String,
    pub detail: Option<String>,
    pub task_label: Option<String>,
    pub role: Option<String>,
    pub output_tail: Option<String>,
    pub todo_progress: Option<(u32, u32)>,
    pub last_update: Instant,
}

impl SwarmMemberMirror {
    pub fn display_name(&self) -> String {
        self.friendly_name
            .clone()
            .or_else(|| self.task_label.clone())
            .unwrap_or_else(|| short_id(&self.session_id))
    }

    pub fn roster_status(&self) -> RosterStatus {
        let s = self.status.to_ascii_lowercase();
        if s.contains("fail") || s.contains("error") {
            RosterStatus::Failed
        } else if s.contains("cancel") || s.contains("stopp") || s.contains("abort") {
            RosterStatus::Cancelled
        } else if s.contains("complete") || s.contains("done") || s == "succeeded" {
            RosterStatus::Completed
        } else if s.contains("input") || s.contains("permission") || s.contains("ask") {
            RosterStatus::NeedsInput
        } else if s.contains("idle") || s.contains("ready") || s.contains("waiting") {
            RosterStatus::Idle
        } else {
            RosterStatus::Running
        }
    }
}

/// Soft-buffer line for a swarm member transcript (Claude `task.messages` analogue).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftTranscriptLine {
    pub message_id: String,
    pub role: String,
    pub content: String,
    pub tool_name: Option<String>,
}

/// Shared team task strip item (pending / in progress / completed + claim).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamTaskItem {
    pub id: String,
    pub content: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub blocked_by: Vec<String>,
    pub priority: String,
}

impl TeamTaskItem {
    pub fn is_pending(&self) -> bool {
        let s = self.status.to_ascii_lowercase();
        s.is_empty() || s == "pending" || s == "todo" || s == "ready"
    }

    pub fn is_in_progress(&self) -> bool {
        let s = self.status.to_ascii_lowercase();
        s == "in_progress" || s == "in-progress" || s == "active" || s == "running"
    }

    pub fn is_completed(&self) -> bool {
        let s = self.status.to_ascii_lowercase();
        s == "completed" || s == "done" || s == "complete"
    }

    pub fn is_claimable(&self) -> bool {
        self.is_pending() && self.assigned_to.is_none() && self.blocked_by.is_empty()
    }
}

/// Build unified roster rows.
///
/// Lead is always first. Grok subagents win over swarm members with the same
/// session id (prefer child AgentView for transcript).
pub fn build_roster(
    lead_label: &str,
    subagents: &HashMap<String, SubagentInfo>,
    swarm_members: &HashMap<String, SwarmMemberMirror>,
    soft_buffers: &HashMap<String, Vec<SoftTranscriptLine>>,
    now: Instant,
    idle_hide_secs: u64,
) -> Vec<AgentRosterRow> {
    let mut rows = Vec::new();
    rows.push(AgentRosterRow {
        id: "__lead__".to_string(),
        display_name: lead_label.to_string(),
        source: RosterSource::Lead,
        status: RosterStatus::Lead,
        activity: None,
        color_key: "lead".to_string(),
        kill_subagent_id: None,
        can_message: true,
        can_open_transcript: false,
        todo_progress: None,
        is_lead: true,
    });

    let mut grok_ids: Vec<&String> = subagents.keys().collect();
    grok_ids.sort();
    for sid in grok_ids {
        let info = &subagents[sid];
        let status = grok_status(info);
        if should_collapse_idle(status, info.last_progress_at, now, idle_hide_secs, subagents) {
            continue;
        }
        rows.push(AgentRosterRow {
            id: sid.clone(),
            display_name: info.description.to_string(),
            source: RosterSource::GrokSubagent,
            status,
            activity: info.activity_label.clone(),
            color_key: sid.clone(),
            kill_subagent_id: Some(info.subagent_id.to_string()),
            can_message: !info.finished,
            can_open_transcript: true,
            todo_progress: None,
            is_lead: false,
        });
    }

    let mut swarm_ids: Vec<&String> = swarm_members.keys().collect();
    swarm_ids.sort();
    for sid in swarm_ids {
        if subagents.contains_key(sid) {
            continue;
        }
        let member = &swarm_members[sid];
        let status = member.roster_status();
        if should_collapse_idle_swarm(status, member.last_update, now, idle_hide_secs, swarm_members)
        {
            continue;
        }
        let activity = member
            .output_tail
            .as_ref()
            .map(|t| truncate_activity(t))
            .or_else(|| member.detail.clone());
        let has_soft = soft_buffers.get(sid).is_some_and(|b| !b.is_empty());
        rows.push(AgentRosterRow {
            id: sid.clone(),
            display_name: member.display_name(),
            source: RosterSource::SwarmMember,
            status,
            activity,
            color_key: sid.clone(),
            kill_subagent_id: None,
            can_message: !status.is_terminal(),
            can_open_transcript: has_soft || !status.is_terminal(),
            todo_progress: member.todo_progress,
            is_lead: false,
        });
    }

    rows
}

fn grok_status(info: &SubagentInfo) -> RosterStatus {
    if info.pending_kill {
        return RosterStatus::PendingKill;
    }
    if !info.finished {
        return RosterStatus::Running;
    }
    match info.status.as_deref() {
        Some("failed") => RosterStatus::Failed,
        Some("cancelled") => RosterStatus::Cancelled,
        _ => RosterStatus::Completed,
    }
}

/// Claude 2.1.199+ spirit: hide idle rows after `idle_hide_secs` when all workers idle.
fn should_collapse_idle(
    status: RosterStatus,
    last_progress: Instant,
    now: Instant,
    idle_hide_secs: u64,
    subagents: &HashMap<String, SubagentInfo>,
) -> bool {
    if idle_hide_secs == 0 || !status.is_terminal() {
        return false;
    }
    let any_active = subagents.values().any(|i| !i.finished || i.pending_kill);
    if any_active {
        return false;
    }
    now.duration_since(last_progress).as_secs() >= idle_hide_secs
}

fn should_collapse_idle_swarm(
    status: RosterStatus,
    last_update: Instant,
    now: Instant,
    idle_hide_secs: u64,
    members: &HashMap<String, SwarmMemberMirror>,
) -> bool {
    if idle_hide_secs == 0 || !status.is_terminal() {
        return false;
    }
    let any_active = members
        .values()
        .any(|m| !m.roster_status().is_terminal());
    if any_active {
        return false;
    }
    now.duration_since(last_update).as_secs() >= idle_hide_secs
}

fn short_id(id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.len() <= 8 {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..8])
    }
}

fn truncate_activity(text: &str) -> String {
    let line = text.lines().last().unwrap_or(text).trim();
    const MAX: usize = 48;
    if line.chars().count() <= MAX {
        line.to_string()
    } else {
        let truncated: String = line.chars().take(MAX.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

/// Stable color index from a name (0..n).
pub fn color_index_for(key: &str, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let mut hash: u32 = 2166136261;
    for b in key.as_bytes() {
        hash ^= u32::from(*b);
        hash = hash.wrapping_mul(16777619);
    }
    (hash as usize) % n
}

/// Panel selection state machine (pure).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentPanelState {
    /// Whether Shift+↑/↓ selection mode is active (or auto when roster visible).
    pub selecting: bool,
    pub selected_index: usize,
    /// Soft-view session id when viewing a swarm member without Grok child view.
    pub soft_view_session: Option<String>,
    /// Show shared team task strip (Ctrl+Shift+T).
    pub show_team_tasks: bool,
    /// Selected task index when task strip focused.
    pub task_selected: usize,
}

impl AgentPanelState {
    pub fn clamp_selection(&mut self, row_count: usize) {
        if row_count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index >= row_count {
            self.selected_index = row_count - 1;
        }
    }

    pub fn select_prev(&mut self, row_count: usize) {
        self.selecting = true;
        if row_count == 0 {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            row_count - 1
        } else {
            self.selected_index - 1
        };
    }

    pub fn select_next(&mut self, row_count: usize) {
        self.selecting = true;
        if row_count == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % row_count;
    }

    pub fn selected_row<'a>(&self, rows: &'a [AgentRosterRow]) -> Option<&'a AgentRosterRow> {
        rows.get(self.selected_index)
    }

    pub fn exit_selection(&mut self) {
        self.selecting = false;
        self.selected_index = 0;
        self.soft_view_session = None;
    }

    pub fn toggle_team_tasks(&mut self) {
        self.show_team_tasks = !self.show_team_tasks;
        if !self.show_team_tasks {
            self.task_selected = 0;
        }
    }

    pub fn select_task_prev(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.task_selected = if self.task_selected == 0 {
            count - 1
        } else {
            self.task_selected - 1
        };
    }

    pub fn select_task_next(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.task_selected = (self.task_selected + 1) % count;
    }
}

/// Resolve prompt routing target: lead session vs worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRoute {
    Lead,
    /// ACP prompt to Grok child session.
    GrokChild { child_session_id: String },
    /// Soft DM / notify for swarm member.
    SwarmMember { session_id: String },
}

pub fn resolve_message_route(
    active_subagent: Option<&str>,
    soft_view: Option<&str>,
    selecting: bool,
    selected: Option<&AgentRosterRow>,
) -> MessageRoute {
    if let Some(sid) = active_subagent {
        return MessageRoute::GrokChild {
            child_session_id: sid.to_string(),
        };
    }
    if let Some(sid) = soft_view {
        return MessageRoute::SwarmMember {
            session_id: sid.to_string(),
        };
    }
    if selecting {
        if let Some(row) = selected {
            if row.is_lead {
                return MessageRoute::Lead;
            }
            return match row.source {
                RosterSource::GrokSubagent => MessageRoute::GrokChild {
                    child_session_id: row.id.clone(),
                },
                RosterSource::SwarmMember => MessageRoute::SwarmMember {
                    session_id: row.id.clone(),
                },
                RosterSource::Lead => MessageRoute::Lead,
            };
        }
    }
    MessageRoute::Lead
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    fn sample_subagent(id: &str, finished: bool) -> SubagentInfo {
        let now = Instant::now();
        SubagentInfo {
            subagent_id: Arc::from(format!("sub-{id}")),
            child_session_id: Arc::from(id),
            description: Arc::from(format!("Worker {id}")),
            subagent_type: Arc::from("general-purpose"),
            persona: None,
            role: None,
            model: None,
            context_source: None,
            resumed_from: None,
            capability_mode: None,
            context_normalized: false,
            parent_prompt_id: None,
            started_at: now,
            last_progress_at: now,
            finished,
            status: if finished {
                Some(Arc::from("completed"))
            } else {
                None
            },
            error: None,
            duration_ms: None,
            tool_calls: None,
            turns: None,
            turn_count: None,
            tool_call_count: None,
            tokens_used: None,
            context_window_tokens: None,
            context_usage_pct: None,
            tools_used: vec![],
            error_count: None,
            activity_label: Some("Thinking".into()),
            is_background: true,
            pending_kill: false,
            kill_requested_at: None,
            scrollback_entry_id: None,
            prompt: None,
            child_cwd: None,
            worktree_path: None,
            child_updates_replayed: false,
        }
    }

    #[test]
    fn roster_merges_grok_and_swarm_dedupes_shared_id() {
        let mut sub = HashMap::new();
        sub.insert("child-a".into(), sample_subagent("child-a", false));
        let mut swarm = HashMap::new();
        swarm.insert(
            "child-a".into(),
            SwarmMemberMirror {
                session_id: "child-a".into(),
                friendly_name: Some("SwarmA".into()),
                status: "running".into(),
                detail: None,
                task_label: None,
                role: None,
                output_tail: None,
                todo_progress: None,
                last_update: Instant::now(),
            },
        );
        swarm.insert(
            "swarm-b".into(),
            SwarmMemberMirror {
                session_id: "swarm-b".into(),
                friendly_name: Some("SwarmB".into()),
                status: "running".into(),
                detail: Some("editing".into()),
                task_label: None,
                role: None,
                output_tail: None,
                todo_progress: Some((1, 3)),
                last_update: Instant::now(),
            },
        );
        let rows = build_roster("Lead", &sub, &swarm, &HashMap::new(), Instant::now(), 0);
        assert_eq!(rows.len(), 3); // lead + grok child-a + swarm-b
        assert!(rows[0].is_lead);
        assert_eq!(rows[1].id, "child-a");
        assert_eq!(rows[1].source, RosterSource::GrokSubagent);
        assert_eq!(rows[1].display_name, "Worker child-a");
        assert_eq!(rows[2].id, "swarm-b");
        assert_eq!(rows[2].source, RosterSource::SwarmMember);
    }

    #[test]
    fn selection_wraps_and_clamps() {
        let mut st = AgentPanelState::default();
        st.select_next(3);
        assert_eq!(st.selected_index, 1);
        st.select_next(3);
        st.select_next(3);
        assert_eq!(st.selected_index, 0);
        st.select_prev(3);
        assert_eq!(st.selected_index, 2);
        st.selected_index = 99;
        st.clamp_selection(3);
        assert_eq!(st.selected_index, 2);
    }

    #[test]
    fn message_route_prefers_active_then_selected_worker() {
        let worker = AgentRosterRow {
            id: "w1".into(),
            display_name: "W".into(),
            source: RosterSource::GrokSubagent,
            status: RosterStatus::Running,
            activity: None,
            color_key: "w1".into(),
            kill_subagent_id: Some("sub-w1".into()),
            can_message: true,
            can_open_transcript: true,
            todo_progress: None,
            is_lead: false,
        };
        assert_eq!(
            resolve_message_route(Some("active"), None, true, Some(&worker)),
            MessageRoute::GrokChild {
                child_session_id: "active".into()
            }
        );
        assert_eq!(
            resolve_message_route(None, None, true, Some(&worker)),
            MessageRoute::GrokChild {
                child_session_id: "w1".into()
            }
        );
        assert_eq!(
            resolve_message_route(None, None, false, Some(&worker)),
            MessageRoute::Lead
        );
        assert_eq!(
            resolve_message_route(None, Some("swarm-x"), false, None),
            MessageRoute::SwarmMember {
                session_id: "swarm-x".into()
            }
        );
    }

    #[test]
    fn idle_collapse_hides_finished_when_all_idle() {
        let mut sub = HashMap::new();
        let mut done = sample_subagent("done", true);
        done.last_progress_at = Instant::now() - Duration::from_secs(60);
        sub.insert("done".into(), done);
        let rows = build_roster("Lead", &sub, &HashMap::new(), &HashMap::new(), Instant::now(), 30);
        assert_eq!(rows.len(), 1); // lead only
        assert!(rows[0].is_lead);
    }

    #[test]
    fn team_task_claimable() {
        let item = TeamTaskItem {
            id: "t1".into(),
            content: "Do thing".into(),
            status: "pending".into(),
            assigned_to: None,
            blocked_by: vec![],
            priority: "medium".into(),
        };
        assert!(item.is_claimable());
        let blocked = TeamTaskItem {
            blocked_by: vec!["t0".into()],
            ..item.clone()
        };
        assert!(!blocked.is_claimable());
    }
}
