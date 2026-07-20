//! Tool output types used by the pager tracker / diff UI.
//! Shapes mirror upstream `xai-grok-tools` (serde `tag = "type"` on ToolOutput).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOutput {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consumed_completion_task_id: Option<String>,
}

impl From<String> for TextOutput {
    fn from(text: String) -> Self {
        Self {
            text,
            consumed_completion_task_id: None,
        }
    }
}

impl From<&str> for TextOutput {
    fn from(text: &str) -> Self {
        Self::from(text.to_owned())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicOutput {
    pub value: serde_json::Value,
}

impl From<serde_json::Value> for DynamicOutput {
    fn from(value: serde_json::Value) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaGenOutput {
    pub path: PathBuf,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub session_folder: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uploaded_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashOutput {
    pub output: Vec<u8>,
    #[serde(default)]
    pub output_for_prompt: String,
    pub exit_code: i32,
    pub command: String,
    pub truncated: bool,
    pub signal: Option<String>,
    pub timed_out: bool,
    pub description: Option<String>,
    pub current_dir: String,
    pub output_file: String,
    pub total_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub output_delta: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub was_bare_echo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTaskStarted {
    pub task_id: String,
    pub task_type: String,
    pub output_file: String,
    pub status: String,
    pub command: String,
    pub summary: String,
    #[serde(default)]
    pub retrieval_hint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_formatted: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepLineMatch {
    pub line_number: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepFileMatch {
    pub path: String,
    pub matches: Vec<GrepLineMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepSearchOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
    pub match_count: usize,
    #[serde(default)]
    pub file_matches: Vec<GrepFileMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_concise: Option<String>,
    pub absolute_path: PathBuf,
    pub offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    pub raw_output: String,
    #[serde(default)]
    pub total_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    pub data: String,
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPageImage {
    pub data: String,
    pub mime_type: String,
    pub page_number: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPageImages {
    pub pages: Vec<PdfPageImage>,
    pub total_pages: usize,
    pub file_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadFileOutput {
    FileContent(FileContent),
    FileNotFound(String),
    IsADirectory(String),
    PermissionDenied(String),
    FileTooLarge(String),
    FileReadError(String),
    ImageContent(ImageContent),
    ImageSizeError(String),
    PdfPageImages(PdfPageImages),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDirContent {
    pub absolute_path: PathBuf,
    pub entries: Vec<String>,
    #[serde(default)]
    pub tool_output_for_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListDirOutput {
    Content(ListDirContent),
    NotFound(String),
    IsAFile(String),
    NotADirectory(String),
    PermissionDenied(String),
    Error(String),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SearchReplaceEditContextInformation {
    pub details: Vec<SearchReplaceEditDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReplaceEditDetail {
    pub old_string: String,
    pub old_line: usize,
    pub new_string: String,
    pub new_line: usize,
    pub context_before: String,
    pub context_after: String,
    #[serde(default)]
    pub line_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReplaceEditsApplied {
    pub old_string: String,
    pub new_string: String,
    pub tool_output_for_prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_output_for_prompt_concise: Option<String>,
    pub absolute_path: PathBuf,
    pub edits: SearchReplaceEditContextInformation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unicode_normalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoMatchesFoundError {
    pub message: String,
    pub absolute_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchReplaceOutput {
    FileAlreadyExists(String),
    EditsApplied(SearchReplaceEditsApplied),
    MultipleMatchesFound(String),
    NoMatchesFound(NoMatchesFoundError),
    InvalidInput(String),
    FileNotFound(String),
    FilenameTooLong(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoWriteOutput {
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchOutput {
    pub query: String,
    pub content: String,
    pub citations: Vec<String>,
    pub allowed_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pre_formatted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebFetchOutputLocation {
    pub file_path: String,
    pub size_bytes: usize,
    pub line_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchContent {
    pub url: String,
    pub content: String,
    pub content_type: String,
    pub status_code: u16,
    pub bytes: usize,
    #[serde(
        rename = "outputLocation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub output_location: Option<WebFetchOutputLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebFetchOutput {
    Content(WebFetchContent),
    DomainNotAllowed(String),
    CrossHostRedirect {
        original_host: String,
        redirect_url: String,
    },
    Error {
        url: Option<String>,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MCPOutputDetails {
    OkayOutput(String),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPOutput {
    tool_name: String,
    server_name: String,
    output: MCPOutputDetails,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reconnect_attempted: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub auth_retry_attempted: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_timeout: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

impl MCPOutput {
    pub fn okay_output(tool_name: String, server_name: String, output: String) -> Self {
        Self {
            tool_name,
            server_name,
            output: MCPOutputDetails::OkayOutput(output),
            reconnect_attempted: false,
            auth_retry_attempted: false,
            is_timeout: false,
            is_error: false,
        }
    }

    pub fn errored(tool_name: String, server_name: String, error: String) -> Self {
        Self {
            tool_name,
            server_name,
            output: MCPOutputDetails::Error(error),
            reconnect_attempted: false,
            auth_retry_attempted: false,
            is_timeout: false,
            is_error: true,
        }
    }

    pub fn output(&self) -> &MCPOutputDetails {
        &self.output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchToolOutput {
    pub result_count: usize,
    pub content: String,
}

/// Opaque payloads for variants the pager rarely matches on.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OpaqueJson(pub serde_json::Value);

impl Default for OpaqueJson {
    fn default() -> Self {
        Self(serde_json::Value::Null)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolOutput {
    Bash(BashOutput),
    BackgroundTaskStarted(BackgroundTaskStarted),
    GrepSearch(GrepSearchOutput),
    ReadFile(ReadFileOutput),
    ListDir(ListDirOutput),
    SearchReplace(SearchReplaceOutput),
    Todo(OpaqueJson),
    WebSearch(WebSearchOutput),
    WebFetch(WebFetchOutput),
    MCP(MCPOutput),
    TaskOutput(OpaqueJson),
    KillTask(OpaqueJson),
    Skill(OpaqueJson),
    ApplyPatch(OpaqueJson),
    CodexGrepFiles(OpaqueJson),
    SearchTool(SearchToolOutput),
    SubagentCompleted(OpaqueJson),
    EnterPlanMode(OpaqueJson),
    ExitPlanMode(OpaqueJson),
    AskUserQuestion(OpaqueJson),
    Monitor(OpaqueJson),
    SchedulerCreate(OpaqueJson),
    SchedulerDelete(OpaqueJson),
    SchedulerList(OpaqueJson),
    UpdateGoal(OpaqueJson),
    Dynamic(DynamicOutput),
    Text(TextOutput),
    ImageGen(MediaGenOutput),
    ImageToVideo(MediaGenOutput),
    ReferenceToVideo(MediaGenOutput),
    ImageEdit(MediaGenOutput),
}
