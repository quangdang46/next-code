//! Face permission confirm bridge: daemon `ServerEvent::PermissionRequest`
//! → ACP `Client::request_permission` → daemon `Request::PermissionResponse`.
//!
//! Distinct from AskUserQuestion (`question_view` / `x.ai/ask_user_question`).

use std::sync::Arc;

use agent_client_protocol as acp;
use agent_client_protocol::Client as _;
use anyhow::{Context, Result};
use xai_acp_lib::AcpGatewaySender;

use crate::protocol::{Request, ServerEvent};

use super::pager_agent::DaemonSession;

pub(crate) const OUTCOME_ALLOW_ONCE: &str = "allow-once";
pub(crate) const OUTCOME_ALLOW_ALWAYS: &str = "allow-always";
pub(crate) const OUTCOME_ALLOW_ALL: &str = "allow-all";
pub(crate) const OUTCOME_REJECT_ONCE: &str = "reject-once";
pub(crate) const OUTCOME_CANCELLED: &str = "cancelled";

/// Map an ACP permission response to a daemon outcome id.
pub(crate) fn outcome_from_acp_response(resp: &acp::RequestPermissionResponse) -> String {
    match &resp.outcome {
        acp::RequestPermissionOutcome::Cancelled => OUTCOME_CANCELLED.to_string(),
        acp::RequestPermissionOutcome::Selected(selected) => {
            selected.option_id.0.as_ref().to_string()
        }
        _ => OUTCOME_CANCELLED.to_string(),
    }
}

fn permission_options() -> Vec<acp::PermissionOption> {
    vec![
        acp::PermissionOption::new(
            acp::PermissionOptionId::new(Arc::from(OUTCOME_ALLOW_ONCE)),
            "Allow once".to_string(),
            acp::PermissionOptionKind::AllowOnce,
        ),
        acp::PermissionOption::new(
            acp::PermissionOptionId::new(Arc::from(OUTCOME_ALLOW_ALWAYS)),
            "Always allow this tool".to_string(),
            acp::PermissionOptionKind::AllowAlways,
        ),
        acp::PermissionOption::new(
            acp::PermissionOptionId::new(Arc::from(OUTCOME_ALLOW_ALL)),
            "Allow all tools this session".to_string(),
            acp::PermissionOptionKind::AllowAlways,
        ),
        acp::PermissionOption::new(
            acp::PermissionOptionId::new(Arc::from(OUTCOME_REJECT_ONCE)),
            "Reject".to_string(),
            acp::PermissionOptionKind::RejectOnce,
        ),
    ]
}

fn build_request(
    session_id: &str,
    tool_name: &str,
    reason: &str,
    tool_call_id: &str,
    tool_input: Option<serde_json::Value>,
) -> acp::RequestPermissionRequest {
    let call_id = if tool_call_id.is_empty() {
        format!("perm-{tool_name}")
    } else {
        tool_call_id.to_string()
    };
    let mut fields = acp::ToolCallUpdateFields::new()
        .title(format!("Allow {tool_name}?"))
        .kind(acp::ToolKind::Other);
    if let Some(raw) = tool_input {
        fields = fields.raw_input(raw);
    } else if !reason.is_empty() {
        fields = fields.raw_input(serde_json::json!({
            "reason": reason,
            "tool_name": tool_name,
        }));
    }
    acp::RequestPermissionRequest::new(
        acp::SessionId::new(Arc::from(session_id)),
        acp::ToolCallUpdate::new(acp::ToolCallId::new(Arc::from(call_id.as_str())), fields),
        permission_options(),
    )
}

pub(crate) async fn bridge_permission_request(
    gateway: &AcpGatewaySender<acp::AgentSide>,
    session: &DaemonSession,
    event: ServerEvent,
) -> Result<()> {
    let ServerEvent::PermissionRequest {
        request_id,
        session_id,
        tool_name,
        reason,
        allow_once_code,
        tool_input,
        tool_call_id,
        ..
    } = event
    else {
        anyhow::bail!("bridge_permission_request expected PermissionRequest event");
    };

    let args = build_request(
        &session_id,
        &tool_name,
        &reason,
        &tool_call_id,
        tool_input,
    );
    let reply_id = session.next_id();
    let outcome = match gateway.request_permission(args).await {
        Ok(resp) => outcome_from_acp_response(&resp),
        Err(err) => {
            crate::logging::warn(&format!(
                "Face request_permission failed; denying: {err}"
            ));
            OUTCOME_CANCELLED.to_string()
        }
    };

    session
        .send(&Request::PermissionResponse {
            id: reply_id,
            request_id,
            outcome,
            session_id,
            tool_name,
            allow_once_code,
        })
        .await
        .context("send PermissionResponse")?;
    wait_done(session, reply_id).await
}

async fn wait_done(session: &DaemonSession, request_id: u64) -> Result<()> {
    loop {
        match session.read_event().await? {
            ServerEvent::Done { id } if id == request_id => return Ok(()),
            ServerEvent::Error { id, message, .. } if id == request_id => {
                anyhow::bail!("permission_response failed: {message}");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_maps_cancelled() {
        let resp = acp::RequestPermissionResponse::new(acp::RequestPermissionOutcome::Cancelled);
        assert_eq!(outcome_from_acp_response(&resp), OUTCOME_CANCELLED);
    }

    #[test]
    fn outcome_maps_selected_option_id() {
        let resp = acp::RequestPermissionResponse::new(acp::RequestPermissionOutcome::Selected(
            acp::SelectedPermissionOutcome::new(acp::PermissionOptionId::new(Arc::from(
                OUTCOME_ALLOW_ONCE,
            ))),
        ));
        assert_eq!(outcome_from_acp_response(&resp), OUTCOME_ALLOW_ONCE);
    }

    #[test]
    fn build_request_includes_four_options() {
        let req = build_request("sess", "bash", "needs approval", "tc-1", None);
        assert_eq!(req.options.len(), 4);
        assert_eq!(req.session_id.0.as_ref(), "sess");
        let ids: Vec<_> = req.options.iter().map(|o| o.option_id.0.as_ref()).collect();
        assert_eq!(
            ids,
            [
                OUTCOME_ALLOW_ONCE,
                OUTCOME_ALLOW_ALWAYS,
                OUTCOME_ALLOW_ALL,
                OUTCOME_REJECT_ONCE
            ]
        );
    }
}
