//! Façade stub of upstream `xai-grok-shell::extensions::session_search`.
//! DTO copied verbatim (small); the actual `x.ai/session/search` handler
//! (upstream searches on-disk session history) is not implemented here.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SearchSessionHit {
    pub session_id: String,
    pub cwd: String,
    pub summary: String,
    pub updated_at: String,
    pub score: f32,
    #[serde(default)]
    pub matched_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}
