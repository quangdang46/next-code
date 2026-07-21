use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogSource {
    #[serde(rename = "shell")]
    Shell,
    #[serde(rename = "grok-pager")]
    GrokPager,
    #[serde(rename = "grok-desktop")]
    GrokDesktop,
}

pub const LOG_METHOD: &str = "x.ai/log";
pub const LOG_DIR: &str = "logs";
pub const MAX_SIZE: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogNotificationParams {
    pub src: LogSource,
    pub entries: Vec<ClientLogEntry>,
}

/// Entry as sent by a client (no `src` field — shell stamps it).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientLogEntry {
    pub ts: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ver: Option<String>,
    pub lvl: LogLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    pub msg: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctx: Option<serde_json::Value>,
}

pub fn set_version(_ver: &str) {}

pub fn write(_msg: &str) {}

pub fn emit(
    _lvl: LogLevel,
    _msg: &str,
    _sid: Option<&str>,
    _ctx: Option<serde_json::Value>,
) {
}

pub fn ingest_client_entries(_src: LogSource, _entries: &[ClientLogEntry]) {}

pub fn info(_msg: &str, _sid: Option<&str>, _ctx: Option<serde_json::Value>) {}
pub fn warn(_msg: &str, _sid: Option<&str>, _ctx: Option<serde_json::Value>) {}
pub fn error(_msg: &str, _sid: Option<&str>, _ctx: Option<serde_json::Value>) {}
pub fn debug(_msg: &str, _sid: Option<&str>, _ctx: Option<serde_json::Value>) {}

pub fn snapshot_log() -> Option<Vec<u8>> {
    None
}

pub fn snapshot_session_log(_session_id: &str) -> Option<Vec<u8>> {
    None
}
