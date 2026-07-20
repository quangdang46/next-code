use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHeadChanged {
    pub session_id: String,
    pub branch: Option<String>,
    #[serde(default)]
    pub is_worktree: bool,
    #[serde(default)]
    pub main_repo: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreDegree {
    Full,
    HeadOnly,
}
