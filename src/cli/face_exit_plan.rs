//! Face ExitPlanMode bridge: daemon event → ACP reverse `ext_method` → daemon reply.

use agent_client_protocol as acp;
use agent_client_protocol::Client as _;
use anyhow::{Context, Result};
use xai_acp_lib::AcpGatewaySender;
use xai_grok_tools::implementations::grok_build::exit_plan_mode::ExitPlanModeExtRequest;

use crate::protocol::Request;

use super::pager_agent::DaemonSession;

pub(crate) async fn bridge_exit_plan_mode(
    gateway: &AcpGatewaySender<acp::AgentSide>,
    session: &DaemonSession,
    request_id: String,
    session_id: String,
    tool_call_id: String,
    plan_content: Option<String>,
) -> Result<()> {
    let ext_req = ExitPlanModeExtRequest {
        session_id,
        tool_call_id,
        plan_content,
    };
    let raw =
        serde_json::value::to_raw_value(&ext_req).context("serialize ExitPlanModeExtRequest")?;

    let reply_id = session.next_id();
    match gateway
        .ext_method(acp::ExtRequest::new(
            "x.ai/exit_plan_mode",
            raw.into(),
        ))
        .await
    {
        Ok(ext_resp) => {
            let response: serde_json::Value = serde_json::from_str(ext_resp.0.get())
                .unwrap_or_else(|_| serde_json::json!({ "outcome": "cancelled" }));
            session
                .send(&Request::ExitPlanModeResponse {
                    id: reply_id,
                    request_id,
                    response,
                    error: None,
                })
                .await?;
        }
        Err(err) => {
            session
                .send(&Request::ExitPlanModeResponse {
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
