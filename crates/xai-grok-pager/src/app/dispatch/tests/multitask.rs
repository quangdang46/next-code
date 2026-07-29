//! Tests for Face `/multitask` dispatch.

use super::*;
use crate::slash::commands::multitask::MultitaskArgs;
use agent_client_protocol as acp;

fn with_session(app: &mut AppView) {
    let agent = app.agents.get_mut(&AgentId(0)).unwrap();
    agent.session.session_id = Some(acp::SessionId::from("lead-sess".to_string()));
}

#[test]
fn multitask_empty_errors_helpfully() {
    let mut app = test_app_with_agent();
    with_session(&mut app);
    let effects = dispatch(Action::Multitask(MultitaskArgs::default()), &mut app);
    assert!(effects.is_empty());
    let text = last_system_text(&app, AgentId(0));
    assert!(text.contains("Nothing to multitask"), "got: {text}");
}

#[test]
fn multitask_drains_local_queue_into_spawns() {
    let mut app = test_app_with_agent();
    with_session(&mut app);
    {
        let agent = app.agents.get_mut(&AgentId(0)).unwrap();
        agent.session.enqueue_prompt("task a".into());
        agent.session.enqueue_prompt("task b".into());
    }
    let effects = dispatch(Action::Multitask(MultitaskArgs::default()), &mut app);
    assert_eq!(effects.len(), 2);
    assert!(
        matches!(
            &effects[0],
            Effect::MultitaskSpawn { prompt, .. } if prompt == "task a"
        ),
        "got: {effects:?}"
    );
    assert!(
        matches!(
            &effects[1],
            Effect::MultitaskSpawn { prompt, .. } if prompt == "task b"
        ),
        "got: {effects:?}"
    );
    assert!(app.agents[&AgentId(0)].session.pending_prompts.is_empty());
    let text = last_system_text(&app, AgentId(0));
    assert!(text.contains("starting 2 background worker"), "got: {text}");
}

#[test]
fn multitask_inline_plus_queue() {
    let mut app = test_app_with_agent();
    with_session(&mut app);
    {
        let agent = app.agents.get_mut(&AgentId(0)).unwrap();
        agent.session.enqueue_prompt("from queue".into());
    }
    let effects = dispatch(
        Action::Multitask(MultitaskArgs {
            to_session: None,
            prompts: vec!["inline one".into()],
        }),
        &mut app,
    );
    assert_eq!(effects.len(), 2);
    assert!(matches!(
        &effects[0],
        Effect::MultitaskSpawn { prompt, .. } if prompt == "inline one"
    ));
    assert!(matches!(
        &effects[1],
        Effect::MultitaskSpawn { prompt, .. } if prompt == "from queue"
    ));
}

#[test]
fn multitask_shared_queue_emits_removes_and_spawns() {
    let mut app = test_app_with_agent();
    with_session(&mut app);
    {
        let agent = app.agents.get_mut(&AgentId(0)).unwrap();
        agent.shared_queue = vec![crate::app::prompt_queue::QueueEntryWire {
            id: "q1".into(),
            version: 2,
            owner: None,
            last_editor: None,
            kind: "prompt".into(),
            text: "shared work".into(),
            position: 0,
        }];
    }
    let effects = dispatch(Action::Multitask(MultitaskArgs::default()), &mut app);
    assert_eq!(effects.len(), 2);
    assert!(matches!(
        &effects[0],
        Effect::QueueRemove {
            id,
            expected_version: 2,
            ..
        } if id == "q1"
    ));
    assert!(matches!(
        &effects[1],
        Effect::MultitaskSpawn { prompt, .. } if prompt == "shared work"
    ));
    assert!(app.agents[&AgentId(0)].shared_queue.is_empty());
}

#[test]
fn multitask_to_steers_via_swarm_dm() {
    let mut app = test_app_with_agent();
    with_session(&mut app);
    let effects = dispatch(
        Action::Multitask(MultitaskArgs {
            to_session: Some("worker-9".into()),
            prompts: vec!["please also cover errors".into()],
        }),
        &mut app,
    );
    assert!(
        matches!(
            &effects[..],
            [Effect::MessageSwarmMember {
                target_session_id,
                message,
                ..
            }] if target_session_id == "worker-9"
                && message == "please also cover errors"
        ),
        "got: {effects:?}"
    );
    let text = last_system_text(&app, AgentId(0));
    assert!(text.contains("follow-up"), "got: {text}");
}

#[test]
fn multitask_no_session_reports_error() {
    let mut app = test_app_with_agent();
    {
        let agent = app.agents.get_mut(&AgentId(0)).unwrap();
        agent.session.session_id = None;
    }
    let effects = dispatch(
        Action::Multitask(MultitaskArgs {
            to_session: None,
            prompts: vec!["x".into()],
        }),
        &mut app,
    );
    assert!(effects.is_empty());
    let text = last_system_text(&app, AgentId(0));
    assert!(text.contains("No active session"), "got: {text}");
}

#[test]
fn multitask_two_parallel_spawns_keep_main_queue_clean() {
    // Product smoke: two independent prompts → two MultitaskSpawn effects,
    // local queue empty (main stays interactive / not FIFO-draining).
    let mut app = test_app_with_agent();
    with_session(&mut app);
    let effects = dispatch(
        Action::Multitask(MultitaskArgs {
            to_session: None,
            prompts: vec!["explore auth".into(), "draft tests".into()],
        }),
        &mut app,
    );
    assert_eq!(effects.len(), 2);
    assert!(
        effects
            .iter()
            .all(|e| matches!(e, Effect::MultitaskSpawn { .. }))
    );
    assert!(app.agents[&AgentId(0)].session.pending_prompts.is_empty());
}
