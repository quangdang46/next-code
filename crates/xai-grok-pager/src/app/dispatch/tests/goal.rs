use super::*;
use crate::app::agent::GoalDisplayStatus;

#[test]
fn goal_set_pause_resume_clear_cycle() {
    let mut app = test_app_with_agent();
    let id = AgentId(0);

    let effects = dispatch(
        Action::GoalSet {
            objective: "ship /goal".into(),
        },
        &mut app,
    );
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::GoalSet { objective, .. } if objective == "ship /goal")),
        "set should ACP-persist: {effects:?}"
    );
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::SendPrompt { text, .. } if text.contains("ship /goal")
        ) || matches!(e, Effect::SendPromptNow { .. })),
        "set should enqueue a pursuit prompt: {effects:?}"
    );
    {
        let agent = app.agents.get_mut(&id).unwrap();
        let goal = agent.goal_state.as_ref().expect("goal set");
        assert_eq!(goal.objective, "ship /goal");
        assert_eq!(goal.status, GoalDisplayStatus::Active);
        assert!(agent.show_goal_detail);
    }

    let effects = dispatch(Action::GoalPause, &mut app);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::GoalPause { .. })),
        "pause should ACP-persist: {effects:?}"
    );
    {
        let agent = app.agents.get(&id).unwrap();
        let goal = agent.goal_state.as_ref().unwrap();
        assert_eq!(goal.status, GoalDisplayStatus::UserPaused);
    }

    let effects = dispatch(Action::GoalResume, &mut app);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::GoalResume { .. })),
        "resume should ACP-persist: {effects:?}"
    );
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::SendPrompt { text, .. } if text.contains("resumed")
        ) || matches!(e, Effect::SendPromptNow { .. })),
        "resume should enqueue: {effects:?}"
    );
    {
        let agent = app.agents.get(&id).unwrap();
        assert_eq!(
            agent.goal_state.as_ref().unwrap().status,
            GoalDisplayStatus::Active
        );
    }

    let effects = dispatch(Action::GoalClear, &mut app);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::GoalClear { .. })),
        "clear should ACP-persist: {effects:?}"
    );
    let agent = app.agents.get(&id).unwrap();
    assert!(agent.goal_state.is_none());
    assert!(agent.last_cleared_goal_id.is_some());
    assert!(!agent.show_goal_detail);
}

#[test]
fn goal_show_without_goal_requests_status() {
    let mut app = test_app_with_agent();
    let effects = dispatch(Action::GoalShow, &mut app);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::GoalStatus { .. })),
        "show should request ACP status: {effects:?}"
    );
}
