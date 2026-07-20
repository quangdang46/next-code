//! Vendored near-verbatim from upstream `xai-grok-shell::session::result`
//! (tiny — the ACP ext-method result envelope).

use std::sync::Arc;

use agent_client_protocol as acp;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtMethodResult<T> {
    pub result: Option<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}

impl<T> ExtMethodResult<T> {
    pub fn success(result: T) -> Self {
        Self {
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(error: serde_json::Value) -> Self {
        Self {
            result: None,
            error: Some(error),
        }
    }
}

impl<T: Serialize> ExtMethodResult<T> {
    /// Serialize into an ACP `ExtResponse` body (JSON envelope).
    pub fn to_ext_response(self) -> Result<acp::ExtResponse, serde_json::Error> {
        let raw: Box<RawValue> = serde_json::value::to_raw_value(&self)?;
        Ok(acp::ExtResponse::new(Arc::<RawValue>::from(raw)))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Empty {}
