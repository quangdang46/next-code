//! Stub bash tool input DTO (compile surface for pager permission UI).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashToolInput {
    pub command: String,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub background: Option<bool>,
}
