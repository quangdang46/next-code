//! Vendored near-verbatim from upstream `xai-grok-shell::session::result`
//! (tiny — the ACP ext-method result envelope).

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Empty {}
