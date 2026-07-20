//! Façade stub of upstream `xai-grok-shell::sampling::types` (grown for PR7).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    #[default]
    Medium,
    High,
    Xhigh,
}

impl ReasoningEffort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }
}

impl fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ReasoningEffort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "minimal" => Ok(Self::Minimal),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "xhigh" | "max" => Ok(Self::Xhigh),
            _ => Err(format!("invalid reasoning effort: {s:?}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReasoningEffortOption {
    pub id: String,
    pub value: ReasoningEffort,
    pub label: String,
    pub description: Option<String>,
    pub default: bool,
}

pub const REASONING_EFFORT_META_KEY: &str = "reasoningEffort";
pub const SUPPORTS_REASONING_EFFORT_META_KEY: &str = "supportsReasoningEffort";
pub const REASONING_EFFORTS_META_KEY: &str = "reasoningEfforts";

pub fn parse_canonical_effort_token(token: &str) -> Option<ReasoningEffort> {
    token.parse().ok()
}

pub fn supports_reasoning_effort_meta(
    meta: Option<&serde_json::Map<String, serde_json::Value>>,
) -> bool {
    meta.and_then(|m| m.get(SUPPORTS_REASONING_EFFORT_META_KEY))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub fn parse_reasoning_effort_meta(
    meta: Option<&serde_json::Map<String, serde_json::Value>>,
) -> Option<ReasoningEffort> {
    let raw = meta?.get(REASONING_EFFORT_META_KEY)?;
    let s = raw.as_str()?;
    s.parse().ok()
}

pub fn parse_reasoning_efforts_meta(
    meta: Option<&serde_json::Map<String, serde_json::Value>>,
) -> Option<Vec<ReasoningEffortOption>> {
    let _ = meta;
    Some(vec![])
}

pub fn reasoning_effort_meta_value(effort: ReasoningEffort) -> serde_json::Value {
    serde_json::Value::String(effort.as_str().to_