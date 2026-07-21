use std::collections::HashMap;

use crate::types::tool::ToolKind;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ToolConfig {
    pub id: String,
    pub params: Option<serde_json::Map<String, serde_json::Value>>,
    pub name_override: Option<String>,
    pub params_name_overrides: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_override: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behavior_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<ToolKind>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ToolServerConfig {
    pub tools: Vec<ToolConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behavior_preset: Option<String>,
}
