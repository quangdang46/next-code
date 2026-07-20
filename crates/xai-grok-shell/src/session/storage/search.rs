//! Local session search stubs.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::SearchHit;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionSearchRequest {
    pub query: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    #[serde(default)]
    pub include_content: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionSearchResponse {
    pub results: Vec<SearchHit>,
}

pub async fn execute_search(
    _root: &Path,
    _req: &SessionSearchRequest,
) -> anyhow::Result<SessionSearchResponse> {
    Ok(SessionSearchResponse::default())
}
