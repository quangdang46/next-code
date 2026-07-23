//! Local session helpers for Face resume / `--continue` / project picker.
//!
//! Maps onto next-code's flat store under `<grok_home>/sessions/<id>.json`.
//! Pure FS — no `next-code-base` / app-core dependency.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agent_client_protocol::SessionId;
use serde::{Deserialize, Serialize};

use crate::auth::AuthManager;
use crate::session::info::Info as SessionPathInfo;
use crate::util::grok_home::grok_home;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalFeedbackEntry {
    pub session_id: String,
    pub rating: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserFeedbackEntry {
    pub session_id: String,
    pub comment: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SessionSnapshot {
    id: String,
    #[serde(default)]
    working_dir: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    custom_title: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    parent_id: Option<String>,
    #[serde(default)]
    messages: Vec<serde_json::Value>,
}

pub fn session_exists_by_id(session_id: &str) -> bool {
    session_snapshot_path(session_id).is_file()
}

pub fn session_exists_for_cwd(session_id: &str, cwd: &str) -> bool {
    load_snapshot(session_id)
        .map(|s| cwd_matches(s.working_dir.as_deref().unwrap_or(""), cwd))
        .unwrap_or(false)
}

pub fn find_local_child_for_remote(session_id: &str, cwd: &str) -> Option<String> {
    let mut best: Option<(chrono::DateTime<chrono::Utc>, String)> = None;
    for snap in iter_snapshots() {
        if snap.parent_id.as_deref() != Some(session_id) {
            continue;
        }
        if !cwd_matches(snap.working_dir.as_deref().unwrap_or(""), cwd) {
            continue;
        }
        match &best {
            Some((ts, _)) if snap.updated_at <= *ts => {}
            _ => best = Some((snap.updated_at, snap.id.clone())),
        }
    }
    best.map(|(_, id)| id)
}

pub fn resolve_local_session(session_id: &str, cwd: &str) -> Option<String> {
    if session_exists_for_cwd(session_id, cwd) {
        return Some(session_id.to_string());
    }
    find_local_child_for_remote(session_id, cwd)
}

pub fn resolve_local_session_any_cwd(session_id: &str) -> Option<String> {
    load_snapshot(session_id).and_then(|s| s.working_dir)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub id: SessionId,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub title: Option<String>,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            id: SessionId::new(String::new()),
            cwd: String::new(),
            title: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub info: Info,
    #[serde(default)]
    pub session_summary: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub num_messages: usize,
    #[serde(default)]
    pub num_chat_messages: usize,
    #[serde(default)]
    pub current_model_id: String,
    #[serde(default)]
    pub parent_session_id: Option<String>,
    #[serde(default)]
    pub manual_title: Option<String>,
}

impl Default for Summary {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            info: Info::default(),
            session_summary: String::new(),
            created_at: now,
            updated_at: now,
            num_messages: 0,
            num_chat_messages: 0,
            current_model_id: String::new(),
            parent_session_id: None,
            manual_title: None,
        }
    }
}

impl Summary {
    pub fn is_hidden(&self) -> bool {
        false
    }

    pub fn manual_title_opt(&self) -> Option<String> {
        self.manual_title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .or_else(|| {
                self.info
                    .title
                    .clone()
                    .filter(|t| !t.trim().is_empty())
            })
    }

    pub fn display_title_opt(&self) -> Option<String> {
        self.manual_title_opt().or_else(|| {
            let s = self.session_summary.trim();
            if s.is_empty() {
                None
            } else {
                Some(s.lines().next().unwrap_or(s).trim().to_string())
            }
        })
    }
}

pub async fn list_summaries(cwd: Option<&str>) -> anyhow::Result<Vec<Summary>> {
    let cwd_owned = cwd.map(str::to_owned);
    Ok(tokio::task::spawn_blocking(move || {
        list_summaries_sync(cwd_owned.as_deref())
    })
    .await?)
}

pub async fn list_recent_summaries(limit: usize) -> std::io::Result<Vec<Summary>> {
    let mut all = list_summaries_sync(None);
    all.truncate(limit);
    Ok(all)
}

pub fn session_dir(info: &SessionPathInfo) -> PathBuf {
    sessions_root().join(info.id.0.as_ref())
}

pub fn find_session_dir_by_id(session_id: &str) -> anyhow::Result<PathBuf> {
    if session_snapshot_path(session_id).is_file() {
        return Ok(sessions_root().join(session_id));
    }
    Ok(sessions_root().join("_").join(session_id))
}

pub fn resumed_session_sandbox_profile(
    _session_id: Option<&str>,
    _cwd: Option<&str>,
) -> Option<String> {
    None
}

#[derive(Debug, Clone, Default)]
pub struct SessionHistoryDeletion {
    pub local_removed: bool,
    pub remote_removed: bool,
}

impl SessionHistoryDeletion {
    pub fn any_removed(&self) -> bool {
        self.local_removed || self.remote_removed
    }
}

pub async fn delete_session_history(
    session_id: &str,
    _cwd: Option<&str>,
    _needs_remote: bool,
    _auth: Arc<AuthManager>,
) -> anyhow::Result<SessionHistoryDeletion> {
    let id = session_id.to_owned();
    Ok(tokio::task::spawn_blocking(move || {
        let mut out = SessionHistoryDeletion::default();
        let snap = session_snapshot_path(&id);
        let journal = sessions_root().join(format!("{id}.journal.jsonl"));
        if snap.is_file() {
            std::fs::remove_file(&snap)?;
            out.local_removed = true;
        }
        if journal.is_file() {
            let _ = std::fs::remove_file(&journal);
            out.local_removed = true;
        }
        Ok::<_, std::io::Error>(out)
    })
    .await??)
}

pub fn path_info(session_id: impl Into<String>, cwd: impl Into<String>) -> SessionPathInfo {
    SessionPathInfo {
        id: SessionId::new(session_id.into()),
        cwd: cwd.into(),
    }
}

pub fn sessions_root() -> PathBuf {
    resolve_home().join("sessions")
}

fn resolve_home() -> PathBuf {
    if let Ok(v) = std::env::var("GROK_HOME") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(v) = std::env::var("NEXT_CODE_HOME") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    grok_home()
}

pub async fn list_summaries_path(cwd: Option<&Path>) -> anyhow::Result<Vec<Summary>> {
    let owned = cwd.map(|p| p.to_string_lossy().into_owned());
    list_summaries(owned.as_deref()).await
}

fn session_snapshot_path(session_id: &str) -> PathBuf {
    sessions_root().join(format!("{session_id}.json"))
}

fn session_journal_path(session_id: &str) -> PathBuf {
    sessions_root().join(format!("{session_id}.journal.jsonl"))
}

fn load_snapshot(session_id: &str) -> Option<SessionSnapshot> {
    let raw = std::fs::read_to_string(session_snapshot_path(session_id)).ok()?;
    serde_json::from_str(&raw).ok()
}

/// One line in the Face resume-browser transcript preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptPreviewLine {
    pub role: String,
    pub text: String,
}

/// Load last `max_messages` visible turns from flat `sessions/<id>.json`
/// plus journal `append_messages` (if present).
///
/// Mirrors origin TUI preview richness: user/assistant **text** plus tool
/// fold lines (`✓ read path · 1.1k tok`) from paired `tool_use` /
/// `tool_result` blocks. Skips system-reminder / display_role=system noise.
/// Pure FS — no ACP / no `next-code-base`.
pub fn load_transcript_preview(
    session_id: &str,
    max_messages: usize,
) -> Vec<TranscriptPreviewLine> {
    let mut messages = load_snapshot(session_id)
        .map(|s| s.messages)
        .unwrap_or_default();
    append_journal_messages(session_id, &mut messages);
    let visible = messages_to_preview_lines(&messages);
    let start = visible.len().saturating_sub(max_messages);
    visible[start..].to_vec()
}

fn append_journal_messages(session_id: &str, messages: &mut Vec<serde_json::Value>) {
    let Ok(raw) = std::fs::read_to_string(session_journal_path(session_id)) else {
        return;
    };
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            continue;
        };
        let Some(appended) = value.get("append_messages").and_then(|v| v.as_array()) else {
            continue;
        };
        messages.extend(appended.iter().cloned());
    }
}

#[derive(Clone)]
struct PendingTool {
    name: String,
    input: serde_json::Value,
}

/// Walk stored messages like origin `session::render_messages`: register
/// `tool_use`, emit a fold line on each `tool_result`, flush text turns.
fn messages_to_preview_lines(messages: &[serde_json::Value]) -> Vec<TranscriptPreviewLine> {
    let mut out = Vec::new();
    let mut tool_map: std::collections::HashMap<String, PendingTool> =
        std::collections::HashMap::new();

    for msg in messages {
        let display_role = msg
            .get("display_role")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if display_role.eq_ignore_ascii_case("system") {
            continue;
        }

        let role = msg
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let Some(content) = msg.get("content") else {
            continue;
        };

        if let Some(s) = content.as_str() {
            push_text_preview(&mut out, role, s);
            continue;
        }

        let Some(arr) = content.as_array() else {
            continue;
        };

        let mut text = String::new();
        for part in arr {
            let part_type = part.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match part_type {
                "text" => {
                    if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                        text.push_str(t);
                    }
                }
                "tool_use" => {
                    let id = part
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if id.is_empty() {
                        continue;
                    }
                    let name = part
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("tool")
                        .to_string();
                    let input = part
                        .get("input")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    tool_map.insert(id, PendingTool { name, input });
                }
                "tool_result" => {
                    if !text.is_empty() {
                        push_text_preview(&mut out, role, &text);
                        text.clear();
                    }
                    let tool_use_id = part
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let result_text = extract_tool_result_text(part);
                    let pending = tool_map.remove(tool_use_id).unwrap_or(PendingTool {
                        name: "tool".into(),
                        input: serde_json::Value::Null,
                    });
                    out.push(TranscriptPreviewLine {
                        role: "tool".into(),
                        text: format_tool_fold_line(&pending.name, &pending.input, &result_text),
                    });
                }
                // reasoning / images / etc. — skip for preview density
                _ => {}
            }
        }
        if !text.is_empty() {
            push_text_preview(&mut out, role, &text);
        }
    }
    out
}

fn push_text_preview(out: &mut Vec<TranscriptPreviewLine>, role: &str, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.contains("<system-reminder>") {
        return;
    }
    out.push(TranscriptPreviewLine {
        role: role.to_string(),
        text: trimmed.to_string(),
    });
}

fn extract_tool_result_text(part: &serde_json::Value) -> String {
    let Some(content) = part.get("content") else {
        return String::new();
    };
    if let Some(s) = content.as_str() {
        return s.to_string();
    }
    let Some(arr) = content.as_array() else {
        return content.to_string();
    };
    let mut parts = Vec::new();
    for item in arr {
        if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
            parts.push(t);
        } else if let Some(s) = item.as_str() {
            parts.push(s);
        }
    }
    parts.join("\n")
}

fn format_tool_fold_line(name: &str, input: &serde_json::Value, result: &str) -> String {
    let failed = tool_result_looks_failed(result);
    let icon = if failed { "✗" } else { "✓" };
    let display = display_tool_name(name);
    let summary = tool_summary(name, input);
    let tok = format_approx_token_count(estimate_tokens(result));
    if summary.is_empty() {
        format!("{icon} {display} · {tok}")
    } else {
        format!("{icon} {display} {summary} · {tok}")
    }
}

fn tool_result_looks_failed(result: &str) -> bool {
    let t = result.trim_start();
    t.starts_with("Error:")
        || t.starts_with("error:")
        || t.starts_with("Failed ")
        || t.starts_with("FAILED")
}

fn display_tool_name(name: &str) -> String {
    match name.trim().to_ascii_lowercase().as_str() {
        "read_file" | "readfile" => "read".into(),
        "write_file" | "writefile" => "write".into(),
        "run_terminal_cmd" | "shell" => "bash".into(),
        other => other.to_string(),
    }
}

fn tool_summary(name: &str, input: &serde_json::Value) -> String {
    let key = name.trim().to_ascii_lowercase();
    match key.as_str() {
        "read" | "read_file" | "readfile" => input_path(input).unwrap_or_default(),
        "write" | "write_file" | "writefile" | "edit" | "apply_patch" | "multiedit" => {
            input_path(input).unwrap_or_default()
        }
        "bash" | "shell" | "run_terminal_cmd" => input
            .get("command")
            .and_then(|v| v.as_str())
            .map(|cmd| {
                let truncated = truncate_chars(cmd.trim(), 48);
                format!("$ {truncated}")
            })
            .unwrap_or_default(),
        "glob" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        "grep" => input
            .get("pattern")
            .or_else(|| input.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        _ => input_path(input)
            .or_else(|| {
                input
                    .get("command")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default(),
    }
}

fn input_path(input: &serde_json::Value) -> Option<String> {
    input
        .get("file_path")
        .or_else(|| input.get("path"))
        .and_then(|v| v.as_str())
        .map(|p| truncate_path_display(p, 56))
}

fn truncate_path_display(path: &str, max_chars: usize) -> String {
    let trimmed = path.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let keep = max_chars.saturating_sub(1);
    let start = chars.len().saturating_sub(keep);
    format!("…{}", chars[start..].iter().collect::<String>())
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn estimate_tokens(s: &str) -> usize {
    s.len() / 4
}

fn format_approx_token_count(tokens: usize) -> String {
    match tokens {
        0..=999 => format!("{tokens} tok"),
        1_000..=9_999 => {
            let whole = tokens / 1_000;
            let tenth = (tokens % 1_000) / 100;
            if tenth == 0 {
                format!("{whole}k tok")
            } else {
                format!("{whole}.{tenth}k tok")
            }
        }
        _ => format!("{}k tok", tokens / 1_000),
    }
}

fn iter_snapshots() -> Vec<SessionSnapshot> {
    let Ok(entries) = std::fs::read_dir(sessions_root()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".json") || name.contains(".journal.") {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(snap) = serde_json::from_str::<SessionSnapshot>(&raw) {
            out.push(snap);
        }
    }
    out
}

fn list_summaries_sync(cwd: Option<&str>) -> Vec<Summary> {
    let mut summaries: Vec<Summary> = iter_snapshots()
        .into_iter()
        .filter(|s| match cwd {
            Some(c) => cwd_matches(s.working_dir.as_deref().unwrap_or(""), c),
            None => true,
        })
        .map(snapshot_to_summary)
        .collect();
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    summaries
}

fn snapshot_to_summary(s: SessionSnapshot) -> Summary {
    let title = s
        .custom_title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .or(s.title.clone());
    let n = s.messages.len();
    Summary {
        info: Info {
            id: SessionId::new(s.id),
            cwd: s.working_dir.clone().unwrap_or_default(),
            title: title.clone(),
        },
        session_summary: title.clone().unwrap_or_default(),
        created_at: s.created_at,
        updated_at: s.updated_at,
        num_messages: n,
        num_chat_messages: n,
        current_model_id: s.model.unwrap_or_default(),
        parent_session_id: s.parent_id,
        manual_title: s.custom_title.filter(|t| !t.trim().is_empty()),
    }
}

fn cwd_matches(a: &str, b: &str) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    normalize_cwd(a) == normalize_cwd(b)
}

fn normalize_cwd(s: &str) -> String {
    let path = PathBuf::from(s);
    let canon = std::fs::canonicalize(&path).unwrap_or(path);
    let lossy = canon.to_string_lossy();
    #[cfg(windows)]
    {
        lossy.replace('\\', "/").to_ascii_lowercase()
    }
    #[cfg(not(windows))]
    {
        lossy.replace('\\', "/").into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static GROK_HOME_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_grok_home<R>(f: impl FnOnce(&Path) -> R) -> R {
        let _guard = GROK_HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = TempDir::new().unwrap();
        let prev = std::env::var_os("GROK_HOME");
        unsafe { std::env::set_var("GROK_HOME", home.path()) };
        let out = f(home.path());
        match prev {
            Some(v) => unsafe { std::env::set_var("GROK_HOME", v) },
            None => unsafe { std::env::remove_var("GROK_HOME") },
        }
        out
    }

    fn write_session(home: &Path, id: &str, cwd: &str) {
        let sessions = home.join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let body = serde_json::json!({
            "id": id,
            "parent_id": null,
            "title": "hello",
            "created_at": "2026-07-21T00:00:00Z",
            "updated_at": "2026-07-21T01:00:00Z",
            "messages": [],
            "working_dir": cwd,
            "model": "test-model",
            "status": "Closed",
            "is_canary": false,
            "is_debug": false,
            "saved": false
        });
        std::fs::write(
            sessions.join(format!("{id}.json")),
            serde_json::to_string_pretty(&body).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn resolve_and_list_flat_sessions() {
        with_temp_grok_home(|home| {
            let cwd = home.join("proj");
            std::fs::create_dir_all(&cwd).unwrap();
            let cwd_str = cwd.to_string_lossy().to_string();
            write_session(home, "session_a", &cwd_str);
            assert!(session_exists_for_cwd("session_a", &cwd_str));
            assert_eq!(
                resolve_local_session("session_a", &cwd_str).as_deref(),
                Some("session_a")
            );
            assert_eq!(
                resolve_local_session_any_cwd("session_a").as_deref(),
                Some(cwd_str.as_str())
            );
            let listed = list_summaries_sync(Some(&cwd_str));
            assert_eq!(listed.len(), 1);
            assert_eq!(&*listed[0].info.id.0, "session_a");
            assert_eq!(listed[0].current_model_id, "test-model");
        });
    }

    #[test]
    fn transcript_preview_merges_snapshot_and_journal() {
        with_temp_grok_home(|home| {
            let sessions = home.join("sessions");
            std::fs::create_dir_all(&sessions).unwrap();
            let body = serde_json::json!({
                "id": "session_prev",
                "parent_id": null,
                "title": "preview",
                "created_at": "2026-07-21T00:00:00Z",
                "updated_at": "2026-07-21T01:00:00Z",
                "messages": [
                    {
                        "role": "user",
                        "display_role": "system",
                        "content": [{"type": "text", "text": "<system-reminder>\nhidden\n</system-reminder>"}]
                    },
                    {
                        "role": "user",
                        "content": [{"type": "text", "text": "hello from snapshot"}]
                    }
                ],
                "working_dir": "/tmp/proj",
                "model": "test-model",
                "status": "Closed",
                "is_canary": false,
                "is_debug": false,
                "saved": false
            });
            std::fs::write(
                sessions.join("session_prev.json"),
                serde_json::to_string_pretty(&body).unwrap(),
            )
            .unwrap();
            let journal = serde_json::json!({
                "meta": { "updated_at": "2026-07-21T02:00:00Z" },
                "append_messages": [
                    {
                        "role": "assistant",
                        "content": [{"type": "text", "text": "hello from journal"}]
                    }
                ]
            });
            std::fs::write(
                sessions.join("session_prev.journal.jsonl"),
                format!("{}\n", serde_json::to_string(&journal).unwrap()),
            )
            .unwrap();

            let lines = load_transcript_preview("session_prev", 20);
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0].role, "user");
            assert_eq!(lines[0].text, "hello from snapshot");
            assert_eq!(lines[1].role, "assistant");
            assert_eq!(lines[1].text, "hello from journal");
        });
    }

    #[test]
    fn transcript_preview_emits_tool_fold_lines_like_origin() {
        with_temp_grok_home(|home| {
            let sessions = home.join("sessions");
            std::fs::create_dir_all(&sessions).unwrap();
            // Shape matches live bonehound: assistant tool_use + user tool_result,
            // often with no assistant text — origin preview still shows ✓ read · tok.
            let body = serde_json::json!({
                "id": "session_tools",
                "parent_id": null,
                "title": "tools",
                "created_at": "2026-07-21T00:00:00Z",
                "updated_at": "2026-07-21T01:00:00Z",
                "messages": [
                    {
                        "role": "user",
                        "display_role": "system",
                        "content": [{"type": "text", "text": "<system-reminder>\nhidden\n</system-reminder>"}]
                    },
                    {
                        "role": "user",
                        "content": [{"type": "text", "text": "2"}]
                    },
                    {
                        "role": "assistant",
                        "content": [
                            {"type": "reasoning", "text": "thinking"},
                            {
                                "type": "tool_use",
                                "id": "call_a",
                                "name": "read",
                                "input": {"file_path": "C:/proj/AGENTS.md"}
                            },
                            {
                                "type": "tool_use",
                                "id": "call_b",
                                "name": "read",
                                "input": {"file_path": "C:/proj/prompts/SPEC_DRIVEN.en.md"}
                            }
                        ]
                    },
                    {
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": "call_a",
                            "content": "x".repeat(4400)
                        }]
                    },
                    {
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": "call_b",
                            "content": "y".repeat(31600)
                        }]
                    }
                ],
                "working_dir": "/tmp/proj",
                "model": "test-model",
                "status": "Closed",
                "is_canary": false,
                "is_debug": false,
                "saved": false
            });
            std::fs::write(
                sessions.join("session_tools.json"),
                serde_json::to_string_pretty(&body).unwrap(),
            )
            .unwrap();

            let lines = load_transcript_preview("session_tools", 20);
            assert_eq!(lines[0].role, "user");
            assert_eq!(lines[0].text, "2");
            assert_eq!(lines[1].role, "tool");
            assert!(
                lines[1].text.starts_with("✓ read ") && lines[1].text.contains("AGENTS.md"),
                "expected read AGENTS fold, got {}",
                lines[1].text
            );
            assert!(
                lines[1].text.contains(" tok"),
                "expected tok badge, got {}",
                lines[1].text
            );
            assert_eq!(lines[2].role, "tool");
            assert!(
                lines[2].text.contains("SPEC_DRIVEN.en.md") && lines[2].text.contains("k tok"),
                "expected SPEC fold with k tok, got {}",
                lines[2].text
            );
            assert_eq!(lines.len(), 3);
        });
    }
}
