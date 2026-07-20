//! Stub of upstream `xai-grok-shell::cli_models`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliModelEntry {
    pub id: String,
    pub name: String,
}
