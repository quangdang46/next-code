//! Face Best-of-N bridge: daemon progress/pick → ACP → Face UI → daemon reply.
//!
//! - Progress: non-blocking `ext_notification("x.ai/best_of_n/progress")`
//! - Pick (`mode=show`): blocking `ext_method("x.ai/ask_user_question")` with
//!   candidate options (reuses Face `question_view`), then maps the answer to
//!   `BestOfNPickExtResponse`.

use agent_client_protocol as acp;
use agent_client_protocol::Client as _;
use anyhow::{Context, Result};
use next_code_best_of_n::{
    BestOfNCandidateUi, BestOfNPickExtResponse, BestOfNProgressPayload, index_from_option_label,
};
use xai_acp_lib::AcpGatewaySender;
use xai_grok_tools::implementations::grok_build::ask_user_question::{
    AskUserQuestionExtRequest, AskUserQuestionExtResponse, AskUserQuestionMode, Question,
    QuestionOption,
};

use crate::protocol::Request;

use super::pager_agent::DaemonSession;

pub(crate) async fn emit_best_of_n_progress(
    gateway: &AcpGatewaySender<acp::AgentSide>,
    session_id: &str,
    payload: serde_json::Value,
) {
    let mut body = payload;
    if let Some(obj) = body.as_object_mut() {
        obj.entry("sessionId")
            .or_insert_with(|| serde_json::json!(session_id));
    }
    let Ok(raw) = serde_json::value::to_raw_value(&body) else {
        return;
    };
    let _ = gateway
        .ext_notification(acp::ExtNotification::new(
            "x.ai/best_of_n/progress",
            std::sync::Arc::from(raw),
        ))
        .await;
}

/// Build AskUserQuestion payload from BoN candidates (also used by Face tests).
pub fn candidates_to_question(
    selection_reason: &str,
    recommended_index: usize,
    candidates: &[BestOfNCandidateUi],
) -> Question {
    let options: Vec<QuestionOption> = candidates
        .iter()
        .map(|c| QuestionOption {
            label: c.option_label(),
            description: c.option_description(),
            preview: None,
            id: Some(c.index.to_string()),
        })
        .collect();
    Question {
        question: format!(
            "Best-of-N: pick a winner to apply (recommended #{recommended_index}).\n{selection_reason}"
        ),
        options,
        multi_select: Some(false),
        id: Some("best_of_n_pick".into()),
    }
}

/// Map Face AskUserQuestion response → BoN pick outcome.
pub fn map_ask_response_to_pick(
    response: AskUserQuestionExtResponse,
    candidates: &[BestOfNCandidateUi],
) -> BestOfNPickExtResponse {
    match response {
        AskUserQuestionExtResponse::Accepted { answers, .. } => {
            let label = answers
                .values()
                .flat_map(|v| v.iter())
                .next()
                .map(|s| s.as_str())
                .unwrap_or("");
            match index_from_option_label(label, candidates) {
                Some(index) => BestOfNPickExtResponse::Selected { index },
                None => BestOfNPickExtResponse::Cancelled,
            }
        }
        AskUserQuestionExtResponse::Cancelled
        | AskUserQuestionExtResponse::ChatAboutThis { .. }
        | AskUserQuestionExtResponse::SkipInterview { .. } => BestOfNPickExtResponse::Cancelled,
    }
}

pub(crate) async fn bridge_best_of_n_pick(
    gateway: &AcpGatewaySender<acp::AgentSide>,
    session: &DaemonSession,
    request_id: String,
    session_id: String,
    _run_id: String,
    tool_call_id: String,
    recommended_index: usize,
    selection_reason: String,
    candidates: serde_json::Value,
) -> Result<()> {
    let candidates: Vec<BestOfNCandidateUi> =
        serde_json::from_value(candidates).context("BestOfN candidates payload")?;
    let question = candidates_to_question(&selection_reason, recommended_index, &candidates);
    let ext_req = AskUserQuestionExtRequest {
        session_id,
        tool_call_id,
        questions: vec![question],
        mode: AskUserQuestionMode::Default,
    };
    let raw =
        serde_json::value::to_raw_value(&ext_req).context("serialize AskUserQuestionExtRequest")?;

    let reply_id = session.next_id();
    match gateway
        .ext_method(acp::ExtRequest::new(
            "x.ai/ask_user_question",
            raw.into(),
        ))
        .await
    {
        Ok(ext_resp) => {
            let ask: AskUserQuestionExtResponse = serde_json::from_str(ext_resp.0.get())
                .unwrap_or(AskUserQuestionExtResponse::Cancelled);
            let pick = map_ask_response_to_pick(ask, &candidates);
            let response = serde_json::to_value(pick)?;
            session
                .send(&Request::BestOfNPickResponse {
                    id: reply_id,
                    request_id,
                    response,
                    error: None,
                })
                .await?;
        }
        Err(err) => {
            session
                .send(&Request::BestOfNPickResponse {
                    id: reply_id,
                    request_id,
                    response: serde_json::Value::Null,
                    error: Some(err.to_string()),
                })
                .await?;
        }
    }
    Ok(())
}

/// Pure helper: progress JSON → card text (unit-tested).
pub fn progress_cards_text(payload: &BestOfNProgressPayload) -> String {
    next_code_best_of_n::format_progress_cards(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_code_best_of_n::{BestOfNPhase, BestOfNProgressPayload};

    fn sample_candidates() -> Vec<BestOfNCandidateUi> {
        vec![
            BestOfNCandidateUi {
                index: 0,
                candidate_id: "c0".into(),
                label: "temp-0".into(),
                status: "success".into(),
                file_count: 1,
                files: vec!["a.rs".into()],
                error: None,
                recommended: true,
            },
            BestOfNCandidateUi {
                index: 1,
                candidate_id: "c1".into(),
                label: "temp-1".into(),
                status: "success".into(),
                file_count: 0,
                files: vec![],
                error: None,
                recommended: false,
            },
        ]
    }

    #[test]
    fn progress_cards_include_candidates() {
        let payload = BestOfNProgressPayload {
            run_id: "r1".into(),
            phase: BestOfNPhase::AwaitingPick,
            message: "pick".into(),
            completed: 2,
            total: 2,
            candidates: sample_candidates(),
            recommended_index: Some(0),
            selection_reason: Some("focused".into()),
        };
        let text = progress_cards_text(&payload);
        assert!(text.contains("awaiting_pick"));
        assert!(text.contains("★"));
        assert!(text.contains("#0 temp-0"));
        assert!(text.contains("a.rs"));
    }

    #[test]
    fn map_accepted_selects_index() {
        let cands = sample_candidates();
        let label = cands[1].option_label();
        let ask: AskUserQuestionExtResponse = serde_json::from_value(serde_json::json!({
            "outcome": "accepted",
            "answers": { "best_of_n_pick": [label] }
        }))
        .unwrap();
        let pick = map_ask_response_to_pick(ask, &cands);
        assert_eq!(pick, BestOfNPickExtResponse::Selected { index: 1 });
    }

    #[test]
    fn map_cancelled() {
        let pick = map_ask_response_to_pick(
            AskUserQuestionExtResponse::Cancelled,
            &sample_candidates(),
        );
        assert_eq!(pick, BestOfNPickExtResponse::Cancelled);
    }

    #[test]
    fn candidates_to_question_has_options() {
        let q = candidates_to_question("reason", 0, &sample_candidates());
        assert_eq!(q.options.len(), 2);
        assert!(q.question.contains("recommended #0"));
    }
}
