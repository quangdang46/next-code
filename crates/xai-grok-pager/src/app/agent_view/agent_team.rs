//! Agent-team panel helpers on [`AgentView`].

use std::time::Instant;

use crate::app::agent_roster::{
    build_roster, resolve_message_route, AgentRosterRow, MessageRoute, SoftTranscriptLine,
    SwarmMemberMirror, TeamTaskItem,
};
use crate::app::agent_view::AgentView;
use crate::app::actions::Action;
use crate::views::agent_panel;

impl AgentView {
    /// Unified roster for the under-prompt agent panel.
    pub fn agent_team_roster(&self) -> Vec<AgentRosterRow> {
        let lead = self
            .session_agent_name
            .as_deref()
            .unwrap_or("lead");
        build_roster(
            lead,
            &self.subagent_sessions,
            &self.swarm_members,
            &self.swarm_soft_transcripts,
            Instant::now(),
            30,
        )
    }

    pub fn agent_panel_desired_height(&self, max_rows: u16) -> u16 {
        if self.is_subagent_view {
            return 0;
        }
        let rows = self.agent_team_roster();
        agent_panel::desired_height(
            &rows,
            self.agent_panel.show_team_tasks,
            &self.team_tasks,
            max_rows,
        )
    }

    pub fn viewing_worker_label(&self) -> Option<String> {
        if let Some(ref sid) = self.active_subagent {
            return self
                .subagent_sessions
                .get(sid)
                .map(|i| i.description.to_string())
                .or_else(|| Some(sid.clone()));
        }
        if let Some(ref sid) = self.agent_panel.soft_view_session {
            return self
                .swarm_members
                .get(sid)
                .map(|m| m.display_name())
                .or_else(|| Some(sid.clone()));
        }
        None
    }

    pub fn current_message_route(&self) -> MessageRoute {
        let rows = self.agent_team_roster();
        let selected = self.agent_panel.selected_row(&rows);
        resolve_message_route(
            self.active_subagent.as_deref(),
            self.agent_panel.soft_view_session.as_deref(),
            self.agent_panel.selecting,
            selected,
        )
    }

    /// Session id for the next `Effect::SendPrompt` (lead or Grok child).
    pub fn routed_acp_session_id(&self) -> Option<agent_client_protocol::SessionId> {
        match self.current_message_route() {
            MessageRoute::GrokChild { child_session_id } => {
                Some(agent_client_protocol::SessionId::new(child_session_id))
            }
            MessageRoute::Lead | MessageRoute::SwarmMember { .. } => {
                self.session.session_id.clone()
            }
        }
    }

    /// Soft-DM a swarm member: echo into soft buffer and ask the lead to forward.
    pub fn soft_message_swarm_member(&mut self, session_id: &str, text: &str) -> String {
        let name = self
            .swarm_members
            .get(session_id)
            .map(|m| m.display_name())
            .unwrap_or_else(|| session_id.to_string());
        let msg_id = format!("soft-{}", uuid::Uuid::new_v4());
        self.upsert_swarm_soft_line(
            SoftTranscriptLine {
                message_id: msg_id,
                role: "user".into(),
                content: text.to_string(),
                tool_name: None,
            },
            session_id,
        );
        if let Some(member) = self.swarm_members.get_mut(session_id) {
            member.last_update = Instant::now();
            member.detail = Some("user message pending lead forward".into());
        }
        format!(
            "Send this message to teammate @{name} (session `{session_id}`) via swarm/DM tools if available:\n\n{text}"
        )
    }

    /// Mirror Grok todo pane items into the team task strip when swarm plan
    /// updates are not yet bridged over ACP.
    pub fn sync_team_tasks_from_todos(&mut self) {
        use xai_grok_shell::tools::TodoStatus;
        let tasks: Vec<TeamTaskItem> = self
            .todo
            .todos()
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let status = match t.status {
                    TodoStatus::Pending => "pending",
                    TodoStatus::InProgress => "in_progress",
                    TodoStatus::Completed => "completed",
                    TodoStatus::Cancelled => "cancelled",
                };
                TeamTaskItem {
                    id: format!("todo-{i}"),
                    content: t.content.clone(),
                    status: status.into(),
                    assigned_to: None,
                    blocked_by: Vec::new(),
                    priority: "normal".into(),
                }
            })
            .collect();
        if !tasks.is_empty() {
            self.apply_team_tasks(tasks);
        }
    }

    /// Enter the selected worker transcript (Grok child or soft swarm buffer).
    pub fn agent_panel_enter_selected(&mut self) -> Option<Action> {
        let rows = self.agent_team_roster();
        let Some(row) = self.agent_panel.selected_row(&rows).cloned() else {
            return None;
        };
        if row.is_lead {
            self.active_subagent = None;
            self.agent_panel.soft_view_session = None;
            return None;
        }
        match row.source {
            crate::app::agent_roster::RosterSource::GrokSubagent => {
                self.agent_panel.soft_view_session = None;
                self.open_subagent_fullscreen(row.id);
                None
            }
            crate::app::agent_roster::RosterSource::SwarmMember => {
                self.active_subagent = None;
                self.agent_panel.soft_view_session = Some(row.id);
                None
            }
            crate::app::agent_roster::RosterSource::Lead => None,
        }
    }

    /// Kill the selected worker when possible.
    pub fn agent_panel_kill_selected(&mut self) -> Option<Action> {
        let rows = self.agent_team_roster();
        let row = self.agent_panel.selected_row(&rows)?;
        if row.is_lead {
            return None;
        }
        if let Some(ref sid) = row.kill_subagent_id {
            return Some(Action::KillSubagent(sid.clone()));
        }
        // Swarm-only: mark cancelled locally; daemon stop is a later bridge.
        if let Some(member) = self.swarm_members.get_mut(&row.id) {
            member.status = "cancelled".into();
            member.detail = Some("stopped from agent panel".into());
            member.last_update = Instant::now();
        }
        if self.agent_panel.soft_view_session.as_deref() == Some(row.id.as_str()) {
            self.agent_panel.soft_view_session = None;
        }
        None
    }

    pub fn apply_swarm_status_members(&mut self, members: Vec<SwarmMemberMirror>) {
        self.swarm_members.clear();
        for m in members {
            self.swarm_members.insert(m.session_id.clone(), m);
        }
    }

    pub fn upsert_swarm_soft_line(&mut self, line: SoftTranscriptLine, session_id: &str) {
        let buf = self
            .swarm_soft_transcripts
            .entry(session_id.to_string())
            .or_default();
        if let Some(existing) = buf.iter_mut().find(|l| l.message_id == line.message_id) {
            *existing = line;
        } else {
            buf.push(line);
            if buf.len() > 200 {
                let drain = buf.len() - 200;
                buf.drain(0..drain);
            }
        }
    }

    pub fn apply_team_tasks(&mut self, tasks: Vec<TeamTaskItem>) {
        self.team_tasks = tasks;
        if self.team_tasks.is_empty() {
            self.agent_panel.task_selected = 0;
        } else if self.agent_panel.task_selected >= self.team_tasks.len() {
            self.agent_panel.task_selected = self.team_tasks.len() - 1;
        }
    }

    /// Claim the selected team task locally and emit a lead prompt to claim.
    pub fn agent_panel_claim_selected_task(&mut self) -> Option<String> {
        let idx = self.agent_panel.task_selected;
        let task = self.team_tasks.get_mut(idx)?;
        if !task.is_claimable() {
            return None;
        }
        let id = task.id.clone();
        let content = task.content.clone();
        task.assigned_to = Some("lead".into());
        task.status = "in_progress".into();
        Some(format!(
            "Claim team task `{id}`: {content}"
        ))
    }

    /// Effects to message a swarm member (ACP notify extension when available).
    pub fn effects_message_swarm_member(
        &self,
        _session_id: String,
        _text: String,
    ) -> Vec<crate::app::actions::Effect> {
        // Face has no next-code daemon client; soft_message_swarm_member
        // rewrites into a lead-forward prompt instead.
        vec![]
    }
}
