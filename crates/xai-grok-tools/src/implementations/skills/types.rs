use std::collections::HashMap;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    strum::AsRefStr,
)]
#[repr(u8)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SkillScope {
    Local = 0,
    Repo = 1,
    User = 2,
    Server = 3,
    Bundled = 4,
    Plugin = 5,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillInfo {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub description: String,
    #[serde(default)]
    pub has_user_specified_description: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when_to_use: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    pub path: String,
    pub scope: SkillScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default = "default_true")]
    pub user_invocable: bool,
    #[serde(default)]
    pub disable_model_invocation: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for SkillInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            display_name: None,
            description: String::new(),
            has_user_specified_description: false,
            paths: None,
            when_to_use: None,
            short_description: None,
            author: None,
            argument_hint: None,
            license: None,
            compatibility: None,
            metadata: None,
            path: String::new(),
            scope: SkillScope::User,
            plugin_name: None,
            plugin_version: None,
            plugin_root: None,
            plugin_data: None,
            allowed_tools: None,
            model: None,
            effort: None,
            user_invocable: true,
            disable_model_invocation: false,
            enabled: true,
        }
    }
}

pub fn skill_name_from_path(path: &str) -> Option<&str> {
    let p = std::path::Path::new(path);
    if p.file_name()?.to_str()? == "SKILL.md" {
        p.parent()?.file_name()?.to_str()
    } else {
        None
    }
}
