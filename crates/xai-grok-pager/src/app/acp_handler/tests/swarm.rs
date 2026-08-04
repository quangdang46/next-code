#![cfg_attr(rustfmt, rustfmt::skip)]
use super::*;

/// Build a `x.ai/swarm/status` notification with the given `members` list.
fn swarm_status_notif(
    session_id: &str,
    members: &[serde_json::Value],
) -> acp::ExtNotification {
    let raw = serde_json::value::to_raw_value(
            &serde_json::json!(
                { "sessionId" : session_id, "members" : members, }
            ),
        )
        .unwrap();
    acp::ExtNotification::new("x.ai/swarm/status", std::sync::Arc::from(raw))
}

// The daemon serializes `SwarmMemberStatus` with default serde field names
// (snake_case): `session_id`, `friendly_name`, `status`. Only the envelope's
// top-level `sessionId` is camelCase.
fn member_value(session_id: &str, status: &str) -> serde_json::Value {
    serde_json::json!(
        { "session_id" : session_id, "friendly_name" : session_id, "status" : status, }
    )
}

/// Regression: the daemon's swarm roster includes the LEAD session itself in
/// `members[]`. Mirroring it verbatim rendered the lead twice in the agent
/// panel — a phantom worker row in addition to the dedicated `__lead__` row.
/// `handle_swarm_status` must exclude the member whose session_id equals the
/// agent's own root session, keeping genuinely-distinct workers.
#[test]
fn swarm_status_excludes_lead_from_members() {
    let mut app = make_app_with_agent("session_wolf_1785835328611_bd3c1b86b895e094");
    assert!(handle_ext_notification(
        &swarm_status_notif(
            "session_wolf_1785835328611_bd3c1b86b895e094",
            &[
                member_value("session_wolf_1785835328611_bd3c1b86b895e094", "running"),
                member_value("session_worker_9a2b", "running"),
            ],
        ),
        &mut app,
    ));

    let agent = app.agents.get(&AgentId(0)).unwrap();
    let ids: Vec<&str> = agent.swarm_members.keys().map(|k| k.as_str()).collect();
    assert_eq!(ids, vec!["session_worker_9a2b"], "lead must be excluded");
    assert!(
        !agent.swarm_members.contains_key("session_wolf_1785835328611_bd3c1b86b895e094"),
        "the lead session must not be mirrored as a swarm worker"
    );
}

/// Distinct workers must all be kept (no accidental over-filtering).
#[test]
fn swarm_status_keeps_all_distinct_members() {
    let mut app = make_app_with_agent("lead-1");
    assert!(handle_ext_notification(
        &swarm_status_notif(
            "lead-1",
            &[
                member_value("worker-a", "running"),
                member_value("worker-b", "completed"),
            ],
        ),
        &mut app,
    ));

    let agent = app.agents.get(&AgentId(0)).unwrap();
    let mut ids: Vec<&str> = agent.swarm_members.keys().map(|k| k.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["worker-a", "worker-b"]);
}
