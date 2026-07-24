//! Face ACP `x.ai/goal/*` — session-goal store + GoalUpdated wire.

use serde_json::json;

use crate::session_goal::{self, SessionGoal, SessionGoalStatus};

fn session_id_from_params(params: &serde_json::Value) -> Option<&str> {
    params
        .get("sessionId")
        .or_else(|| params.get("session_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// Build shell `SessionUpdate::GoalUpdated` from a store goal.
pub fn goal_updated_update(
    goal: &SessionGoal,
    last_event: Option<&str>,
    last_event_detail: Option<&str>,
    pause_message: Option<&str>,
) -> xai_grok_shell::extensions::notification::SessionUpdate {
    let (status, phase) = session_goal::wire_status_and_phase(goal.status);
    let elapsed_ms = goal.time_used_seconds.saturating_mul(1000);
    xai_grok_shell::extensions::notification::SessionUpdate::GoalUpdated {
        goal_id: goal.id.clone(),
        objective: goal.objective.clone(),
        status: status.to_string(),
        phase: phase.to_string(),
        token_budget: None,
        tokens_used: goal.tokens_used as i64,
        elapsed_ms,
        total_deliverables: 0,
        completed_deliverables: 0,
        current_deliverable_id: None,
        current_deliverable_title: None,
        current_subagent_role: None,
        total_worker_rounds: 0,
        total_verify_rounds: 0,
        token_baseline: 0,
        finished_subagent_tokens: 0,
        live_subagent_tokens: None,
        live_tokens_by_model: Vec::new(),
        live_context_pct: None,
        live_turn_count: None,
        live_tool_call_count: None,
        last_event: last_event.map(str::to_string),
        last_event_detail: last_event_detail.map(str::to_string),
        last_event_timestamp: None,
        pause_message: pause_message.map(str::to_string),
        classifier_runs_attempted: None,
        classifier_max_runs: None,
        last_classifier_verdict: None,
        last_classifier_details_path: None,
        verifying_completion: None,
        planning: None,
    }
}

fn ok_result(goal: Option<&SessionGoal>) -> serde_json::Value {
    json!({ "result": { "ok": true, "goal": goal } })
}

fn err_result(message: impl Into<String>) -> serde_json::Value {
    json!({ "error": { "message": message.into() } })
}

/// Result of a mutating goal ACP call (store + wire update to emit).
pub struct GoalMutation {
    pub response: serde_json::Value,
    pub session_id: String,
    /// When `Some`, emit GoalUpdated. When clear succeeds, use a synthetic
    /// complete/idle update from `cleared_goal` before Face clears chrome.
    pub emit: Option<GoalEmit>,
}

pub enum GoalEmit {
    Updated {
        goal: SessionGoal,
        last_event: &'static str,
        last_event_detail: Option<&'static str>,
        pause_message: Option<&'static str>,
    },
    /// Cleared — Face chrome clears when `status == "cleared"`.
    Cleared { goal_id: String, objective: String },
}

/// Build the wire update for a [`GoalEmit`].
pub fn emit_to_update(
    emit: &GoalEmit,
) -> xai_grok_shell::extensions::notification::SessionUpdate {
    match emit {
        GoalEmit::Updated {
            goal,
            last_event,
            last_event_detail,
            pause_message,
        } => goal_updated_update(goal, Some(last_event), *last_event_detail, *pause_message),
        GoalEmit::Cleared { goal_id, objective } => {
            xai_grok_shell::extensions::notification::SessionUpdate::GoalUpdated {
                goal_id: goal_id.clone(),
                objective: objective.clone(),
                status: "cleared".to_string(),
                phase: "idle".to_string(),
                token_budget: None,
                tokens_used: 0,
                elapsed_ms: 0,
                total_deliverables: 0,
                completed_deliverables: 0,
                current_deliverable_id: None,
                current_deliverable_title: None,
                current_subagent_role: None,
                total_worker_rounds: 0,
                total_verify_rounds: 0,
                token_baseline: 0,
                finished_subagent_tokens: 0,
                live_subagent_tokens: None,
                live_tokens_by_model: Vec::new(),
                live_context_pct: None,
                live_turn_count: None,
                live_tool_call_count: None,
                last_event: Some("goal_cleared".into()),
                last_event_detail: None,
                last_event_timestamp: None,
                pause_message: None,
                classifier_runs_attempted: None,
                classifier_max_runs: None,
                last_classifier_verdict: None,
                last_classifier_details_path: None,
                verifying_completion: None,
                planning: None,
            }
        }
    }
}

pub fn handle_goal_set(params: &serde_json::Value) -> GoalMutation {
    let Some(session_id) = session_id_from_params(params) else {
        return GoalMutation {
            response: err_result("sessionId is required"),
            session_id: String::new(),
            emit: None,
        };
    };
    let Some(objective) = params
        .get("objective")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return GoalMutation {
            response: err_result("objective is required"),
            session_id: session_id.to_string(),
            emit: None,
        };
    };
    match session_goal::set(session_id, objective) {
        Ok(goal) => GoalMutation {
            response: ok_result(Some(&goal)),
            session_id: session_id.to_string(),
            emit: Some(GoalEmit::Updated {
                goal: goal.clone(),
                last_event: "goal_set",
                last_event_detail: None,
                pause_message: None,
            }),
        },
        Err(err) => GoalMutation {
            response: err_result(err.to_string()),
            session_id: session_id.to_string(),
            emit: None,
        },
    }
}

pub fn handle_goal_pause(params: &serde_json::Value) -> GoalMutation {
    let Some(session_id) = session_id_from_params(params) else {
        return GoalMutation {
            response: err_result("sessionId is required"),
            session_id: String::new(),
            emit: None,
        };
    };
    match session_goal::pause(session_id) {
        Ok(Some(goal)) => GoalMutation {
            response: ok_result(Some(&goal)),
            session_id: session_id.to_string(),
            emit: Some(GoalEmit::Updated {
                goal: goal.clone(),
                last_event: "goal_paused",
                last_event_detail: Some("user"),
                pause_message: Some("Paused by user"),
            }),
        },
        Ok(None) => GoalMutation {
            response: err_result("No active goal to pause"),
            session_id: session_id.to_string(),
            emit: None,
        },
        Err(err) => GoalMutation {
            response: err_result(err.to_string()),
            session_id: session_id.to_string(),
            emit: None,
        },
    }
}

pub fn handle_goal_resume(params: &serde_json::Value) -> GoalMutation {
    let Some(session_id) = session_id_from_params(params) else {
        return GoalMutation {
            response: err_result("sessionId is required"),
            session_id: String::new(),
            emit: None,
        };
    };
    match session_goal::resume(session_id) {
        Ok(Some(goal)) => GoalMutation {
            response: ok_result(Some(&goal)),
            session_id: session_id.to_string(),
            emit: Some(GoalEmit::Updated {
                goal: goal.clone(),
                last_event: "goal_resumed",
                last_event_detail: None,
                pause_message: None,
            }),
        },
        Ok(None) => GoalMutation {
            response: err_result("No goal to resume"),
            session_id: session_id.to_string(),
            emit: None,
        },
        Err(err) => GoalMutation {
            response: err_result(err.to_string()),
            session_id: session_id.to_string(),
            emit: None,
        },
    }
}

pub fn handle_goal_clear(params: &serde_json::Value) -> GoalMutation {
    let Some(session_id) = session_id_from_params(params) else {
        return GoalMutation {
            response: err_result("sessionId is required"),
            session_id: String::new(),
            emit: None,
        };
    };
    let prior = session_goal::get(session_id).ok().flatten();
    match session_goal::clear(session_id) {
        Ok(true) => {
            let Some(goal) = prior else {
                return GoalMutation {
                    response: ok_result(None),
                    session_id: session_id.to_string(),
                    emit: None,
                };
            };
            GoalMutation {
                response: ok_result(None),
                session_id: session_id.to_string(),
                emit: Some(GoalEmit::Cleared {
                    goal_id: goal.id,
                    objective: goal.objective,
                }),
            }
        }
        Ok(false) => GoalMutation {
            response: err_result("No active goal to clear"),
            session_id: session_id.to_string(),
            emit: None,
        },
        Err(err) => GoalMutation {
            response: err_result(err.to_string()),
            session_id: session_id.to_string(),
            emit: None,
        },
    }
}

pub fn handle_goal_status(params: &serde_json::Value) -> serde_json::Value {
    let Some(session_id) = session_id_from_params(params) else {
        return err_result("sessionId is required");
    };
    match session_goal::get(session_id) {
        Ok(goal) => json!({ "result": { "goal": goal } }),
        Err(err) => err_result(err.to_string()),
    }
}

/// Idle-continuation decision after an EndTurn.
pub enum ContinuationDecision {
    None,
    Continue { prompt: String, goal: SessionGoal },
    /// Hit max continuations — goal paused with toast-worthy event.
    MaxReached { goal: SessionGoal },
}

pub fn decide_continuation(
    session_id: &str,
    token_delta: u64,
    elapsed_seconds: u64,
) -> ContinuationDecision {
    if !crate::config::config().goal.enabled {
        return ContinuationDecision::None;
    }
    let Ok(Some(goal)) = session_goal::get(session_id) else {
        return ContinuationDecision::None;
    };
    if goal.status != SessionGoalStatus::Active {
        return ContinuationDecision::None;
    }
    let _ = session_goal::account_usage(session_id, token_delta, elapsed_seconds);
    let max = crate::config::config().goal.max_continuations;
    let Ok(Some(goal)) = session_goal::get(session_id) else {
        return ContinuationDecision::None;
    };
    if goal.continuation_count >= max {
        let _ = session_goal::pause(session_id);
        let goal = session_goal::get(session_id)
            .ok()
            .flatten()
            .unwrap_or(goal);
        return ContinuationDecision::MaxReached { goal };
    }
    let Ok(Some(goal)) = session_goal::bump_continuation(session_id) else {
        return ContinuationDecision::None;
    };
    if goal.status != SessionGoalStatus::Active {
        return ContinuationDecision::None;
    }
    let prompt = session_goal::build_continuation_prompt(&goal);
    ContinuationDecision::Continue { prompt, goal }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_updated_maps_active() {
        let g = SessionGoal {
            id: "sg-1".into(),
            session_id: "ses".into(),
            objective: "Ship".into(),
            status: SessionGoalStatus::Active,
            tokens_used: 10,
            time_used_seconds: 2,
            created_at: 1,
            updated_at: 1,
            last_started_at: Some(1),
            completed_at: None,
            continuation_count: 0,
        };
        let update = goal_updated_update(&g, Some("goal_set"), None, None);
        match update {
            xai_grok_shell::extensions::notification::SessionUpdate::GoalUpdated {
                status,
                phase,
                tokens_used,
                elapsed_ms,
                last_event,
                ..
            } => {
                assert_eq!(status, "active");
                assert_eq!(phase, "executing");
                assert_eq!(tokens_used, 10);
                assert_eq!(elapsed_ms, 2000);
                assert_eq!(last_event.as_deref(), Some("goal_set"));
            }
            other => panic!("expected GoalUpdated, got {other:?}"),
        }
    }
}
