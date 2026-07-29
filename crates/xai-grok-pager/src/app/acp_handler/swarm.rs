//! Face handlers for next-code swarm ACP extensions (`x.ai/swarm/*`).

use std::time::Instant;

use agent_client_protocol as acp;

use super::routing::find_session_match;
use crate::app::agent_roster::{SoftTranscriptLine, SwarmMemberMirror, TeamTaskItem};
use crate::app::app_view::AppView;

#[derive(Debug, serde::Deserialize)]
struct SwarmStatusPayload {
    #[serde(alias = "sessionId")]
    session_id: String,
    #[serde(default)]
    members: Vec<SwarmMemberWire>,
}

#[derive(Debug, serde::Deserialize)]
struct SwarmMemberWire {
    session_id: String,
    #[serde(default)]
    friendly_name: Option<String>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    task_label: Option<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    output_tail: Option<String>,
    #[serde(default)]
    todo_progress: Option<(u32, u32)>,
}

#[derive(Debug, serde::Deserialize)]
struct SwarmMemberMessagePayload {
    #[serde(alias = "sessionId")]
    _lead_session_id: String,
    message: SwarmMemberMessageWire,
}

#[derive(Debug, serde::Deserialize)]
struct SwarmMemberMessageWire {
    session_id: String,
    message_id: String,
    role: String,
    content: String,
    #[serde(default)]
    tool_name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SwarmPlanPayload {
    #[serde(alias = "sessionId")]
    session_id: String,
    #[serde(default)]
    items: Vec<SwarmPlanItemWire>,
}

#[derive(Debug, serde::Deserialize)]
struct SwarmPlanItemWire {
    id: String,
    content: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    priority: String,
    #[serde(default)]
    assigned_to: Option<String>,
    #[serde(default)]
    blocked_by: Vec<String>,
}

/// `x.ai/swarm/status` — replace Face swarm mirrors from daemon roster.
pub(super) fn handle_swarm_status(notif: &acp::ExtNotification, app: &mut AppView) -> bool {
    let Ok(parsed) = serde_json::from_str::<SwarmStatusPayload>(notif.params.get()) else {
        tracing::warn!("Failed to parse x.ai/swarm/status");
        return false;
    };
    let sid = acp::SessionId::new(parsed.session_id);
    let Some(matched) = find_session_match(app, &sid) else {
        return false;
    };
    let parent_id = matched.agent_id();
    let Some(agent) = app.agents.get_mut(&parent_id) else {
        return false;
    };
    let now = Instant::now();
    let members: Vec<SwarmMemberMirror> = parsed
        .members
        .into_iter()
        .map(|m| SwarmMemberMirror {
            session_id: m.session_id,
            friendly_name: m.friendly_name,
            status: m.status,
            detail: m.detail,
            task_label: m.task_label,
            role: m.role,
            output_tail: m.output_tail,
            todo_progress: m.todo_progress,
            last_update: now,
        })
        .collect();
    agent.apply_swarm_status_members(members);
    true
}

/// `x.ai/swarm/member_message` — append soft transcript line for a teammate.
pub(super) fn handle_swarm_member_message(
    notif: &acp::ExtNotification,
    app: &mut AppView,
) -> bool {
    let Ok(parsed) = serde_json::from_str::<SwarmMemberMessagePayload>(notif.params.get()) else {
        tracing::warn!("Failed to parse x.ai/swarm/member_message");
        return false;
    };
    // Match on lead session from envelope sessionId when present; else scan.
    let lead = acp::SessionId::new(parsed._lead_session_id.clone());
    let parent_id = find_session_match(app, &lead)
        .map(|m| m.agent_id())
        .or_else(|| {
            app.agents.iter().find_map(|(id, agent)| {
                agent
                    .swarm_members
                    .contains_key(&parsed.message.session_id)
                    .then_some(*id)
            })
        });
    let Some(parent_id) = parent_id else {
        return false;
    };
    let Some(agent) = app.agents.get_mut(&parent_id) else {
        return false;
    };
    if let Some(member) = agent.swarm_members.get_mut(&parsed.message.session_id) {
        member.last_update = Instant::now();
        if parsed.message.role.eq_ignore_ascii_case("assistant") {
            member.output_tail = Some(parsed.message.content.clone());
        }
    }
    agent.upsert_swarm_soft_line(
        SoftTranscriptLine {
            message_id: parsed.message.message_id,
            role: parsed.message.role,
            content: parsed.message.content,
            tool_name: parsed.message.tool_name,
        },
        &parsed.message.session_id,
    );
    true
}

/// `x.ai/swarm/plan` — shared team task strip from daemon plan snapshot.
pub(super) fn handle_swarm_plan(notif: &acp::ExtNotification, app: &mut AppView) -> bool {
    let Ok(parsed) = serde_json::from_str::<SwarmPlanPayload>(notif.params.get()) else {
        tracing::warn!("Failed to parse x.ai/swarm/plan");
        return false;
    };
    let sid = acp::SessionId::new(parsed.session_id);
    let Some(matched) = find_session_match(app, &sid) else {
        return false;
    };
    let parent_id = matched.agent_id();
    let Some(agent) = app.agents.get_mut(&parent_id) else {
        return false;
    };
    let tasks: Vec<TeamTaskItem> = parsed
        .items
        .into_iter()
        .map(|item| TeamTaskItem {
            id: item.id,
            content: item.content,
            status: if item.status.is_empty() {
                "pending".into()
            } else {
                item.status
            },
            assigned_to: item.assigned_to,
            blocked_by: item.blocked_by,
            priority: if item.priority.is_empty() {
                "normal".into()
            } else {
                item.priority
            },
        })
        .collect();
    agent.apply_team_tasks(tasks);
    true
}
