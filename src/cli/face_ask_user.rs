//! Face AskUserQuestion bridge: daemon event → ACP reverse `ext_method` → daemon reply.
//!
//! Distinct from permission_view / `request_permission` and from freeform StdinRequest.

use agent_client_protocol as acp;
use agent_client_protocol::Client as _;
use anyhow::{Context, Result};
use xai_acp_lib::AcpGatewaySender;
use xai_grok_tools::implementations::grok_build::ask_user_question::{
    AskUserQuestionExtRequest, AskUserQuestionMode, Question,
};

use crate::protocol::Request;

use super::pager_agent::DaemonSession;

pub(crate) async fn bridge_ask_user_question(
    gateway: &AcpGatewaySender<acp::AgentSide>,
    session: &DaemonSession,
    request_id: String,
    session_id: String,
    tool_call_id: String,
    questions: serde_json::Value,
    mode: String,
) -> Result<()> {
    let questions: Vec<Question> =
        serde_json::from_value(questions).context("AskUserQuestion questions payload")?;
    let mode = match mode.as_str() {
        "plan" => AskUserQuestionMode::Plan,
        _ => AskUserQuestionMode::Default,
    };
    let ext_req = AskUserQuestionExtRequest {
        session_id,
        tool_call_id,
        questions,
        mode,
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
            // ExtResponse wraps Arc<RawValue> (`resp.0.get()`).
            let response: serde_json::Value = serde_json::from_str(ext_resp.0.get())
                .unwrap_or_else(|_| serde_json::json!({ "outcome": "cancelled" }));
            session
                .send(&Request::AskUserQuestionResponse {
                    id: reply_id,
                    request_id,
                    response,
                    error: None,
                })
                .await?;
        }
        Err(err) => {
            session
                .send(&Request::AskUserQuestionResponse {
                    id: reply_id,
                    request_id,
                    response: serde_json::Value::Null,
                    error: Some(err.to_string()),
                })
                .await?;
        }
    }
    // Matching `Done` for `reply_id` is ignored by the prompt event loop (`_ => {}`).
    Ok(())
}
