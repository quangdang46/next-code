//! Façade stub of upstream `xai-grok-shell::extensions::notification` — the
//! ACP session-notification DTOs the future pager renders (hook runs,
//! memory files, image compression stats, prompt usage, goal-tracker
//! verdicts). Field shapes are copied from upstream where cheap; `IndexMap`
//! (ordering-sensitive `modelUsage` map) is simplified to `HashMap` since
//! this façade layer has no serialization-order consumer yet.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum HookRunStatusDto {
    Success {
        elapsed_ms: u64,
    },
    Skipped,
    Failed {
        error: String,
        elapsed_ms: u64,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        blocked: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookRunEntryDto {
    pub name: String,
    pub status: HookRunStatusDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MemoryFileInfo {
    pub path: String,
    pub source: String,
    pub size_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_epoch_secs: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ImageCompressedEntry {
    pub index: usize,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub original_width: u32,
    pub original_height: u32,
    pub compressed_width: u32,
    pub compressed_height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalClassifierVerdict {
    Achieved,
    NotAchieved,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PromptUsageModel {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub cached_read_tokens: u64,
    #[serde(default)]
    pub reasoning_tokens: u64,
    #[serde(default)]
    pub model_calls: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PromptUsage {
    #[serde(flatten)]
    pub totals: PromptUsageModel,
    #[serde(default, rename = "modelUsage")]
    pub model_usage: HashMap<String, PromptUsageModel>,
    #[serde(default, rename = "numTurns")]
    pub num_turns: u64,
    #[serde(default, rename = "usageIsIncomplete")]
    pub usage_is_incomplete: bool,
}
