//! Stub of upstream `xai-grok-shell::extensions::prompt_meta`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptBlockMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bash_command: Option<String>,
}

impl PromptBlockMeta {
    pub fn bash(command: impl Into<String>) -> Self {
        Self {
            bash_command: Some(command.into()),
        }
    }
}
