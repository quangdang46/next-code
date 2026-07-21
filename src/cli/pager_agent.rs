//! In-process ACP Agent that bridges Face (`xai-grok-pager`) to the next-code
//! daemon socket protocol — same brain path as `next-code acp`, without stdio.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use agent_client_protocol as acp;
use agent_client_protocol::{
    Client as _, SessionId, ToolCallId, ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use xai_acp_lib::AcpGatewaySender;

use crate::protocol::{Request, ServerEvent};
use crate::transport::{ReadHalf, WriteHalf};

struct DaemonSession {
    session_id: String,
    reader: Mutex<BufReader<ReadHalf>>,
    writer: Mutex<WriteHalf>,
    next_request_id: AtomicU64,
    prompt_running: AtomicBool,
}

impl DaemonSession {
    fn new(session_id: String, reader: ReadHalf, writer: WriteHalf, next_request_id: u64) -> Self {
        Self {
            session_id,
            reader: Mutex::new(BufReader::new(reader)),
            writer: Mutex::new(writer),
            next_request_id: AtomicU64::new(next_request_id),
            prompt_running: AtomicBool::new(false),
        }
    }

    fn next_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn send(&self, request: &Request) -> Result<()> {
        let mut json = serde_json::to_string(request)?;
        json.push('\n');
        let mut writer = self.writer.lock().await;
        writer.write_all(json.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn read_event(&self) -> Result<ServerEvent> {
        let mut line = String::new();
        let mut reader = self.reader.lock().await;
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            anyhow::bail!("Next Code daemon disconnected");
        }
        serde_json::from_str(&line)
            .with_context(|| format!("failed to decode daemon event: {}", line.trim_end()))
    }
}

async fn wait_for_done(session: &DaemonSession, request_id: u64) -> Result<()> {
    loop {
        match session.read_event().await? {
            ServerEvent::Done { id } if id == request_id => return Ok(()),
            ServerEvent::Error { id, message, .. } if id == request_id => {
                anyhow::bail!(message);
            }
            _ => {}
        }
    }
}

async fn request_history(session: &DaemonSession) -> Result<ServerEvent> {
    let id = session.next_id();
    session.send(&Request::GetHistory { id }).await?;
    loop {
        match session.read_event().await? {
            ServerEvent::Ack { .. } => {}
            event @ ServerEvent::History { id: event_id, .. } if event_id == id => {
                return Ok(event);
            }
            ServerEvent::Error {
                id: event_id,
                message,
                ..
            } if event_id == id => anyhow::bail!(message),
            _ => {}
        }
    }
}

/// Face-facing ACP agent: Client (pager) ↔ this ↔ next-code `serve` socket.
pub(crate) struct NextCodeFaceAgent {
    gateway: AcpGatewaySender<acp::AgentSide>,
    sessions: RefCell<HashMap<String, Rc<DaemonSession>>>,
    /// Tool input accumulation buffer, keyed by tool call id.
    /// Mirrors EventMapper::tool_inputs in acp.rs.
    tool_inputs: RefCell<HashMap<String, String>>,
    /// Current tool ID for ToolInput accumulation (ToolInput has no id field).
    current_tool_id: RefCell<Option<String>>,
}

impl NextCodeFaceAgent {
    pub(crate) fn new(gateway: AcpGatewaySender<acp::AgentSide>) -> Self {
        Self {
            gateway,
            sessions: RefCell::new(HashMap::new()),
            tool_inputs: RefCell::new(HashMap::new()),
            current_tool_id: RefCell::new(None),
        }
    }

    async fn connect_halves() -> Result<(ReadHalf, WriteHalf)> {
        let stream = crate::server::connect_socket(&crate::server::socket_path()).await?;
        Ok(stream.into_split())
    }

    async fn create_session(&self, cwd: PathBuf) -> Result<Rc<DaemonSession>> {
        let (reader, writer) = Self::connect_halves().await?;
        let session = DaemonSession::new(String::new(), reader, writer, 2);
        let subscribe_id = 1;
        session
            .send(&Request::Subscribe {
                id: subscribe_id,
                working_dir: Some(cwd.display().to_string()),
                selfdev: None,
                target_session_id: None,
                client_instance_id: Some("face".to_string()),
                client_has_local_history: false,
                allow_session_takeover: false,
                terminal_env: crate::terminal_launch::snapshot_client_terminal_env(),
            })
            .await?;
        wait_for_done(&session, subscribe_id).await?;
        let history = request_history(&session).await?;
        let session_id = match history {
            ServerEvent::History { session_id, .. } => session_id,
            other => anyhow::bail!("expected history after session creation, got {other:?}"),
        };

        let live = Rc::new(DaemonSession::new(
            session_id.clone(),
            session.reader.into_inner().into_inner(),
            session.writer.into_inner(),
            session.next_request_id.load(Ordering::Relaxed),
        ));
        self.sessions
            .borrow_mut()
            .insert(session_id, live.clone());
        Ok(live)
    }

    async fn attach_session(&self, target: String) -> Result<Rc<DaemonSession>> {
        let (reader, writer) = Self::connect_halves().await?;
        let session = DaemonSession::new(String::new(), reader, writer, 2);
        let resume_id = 1;
        session
            .send(&Request::ResumeSession {
                id: resume_id,
                session_id: target.clone(),
                client_instance_id: Some("face".to_string()),
                client_has_local_history: false,
                allow_session_takeover: false,
            })
            .await?;

        let mut attached = target;
        loop {
            match session.read_event().await? {
                ServerEvent::Ack { .. } => {}
                ServerEvent::History { session_id, .. } => {
                    attached = session_id;
                }
                ServerEvent::Done { id } if id == resume_id => break,
                ServerEvent::Error { id, message, .. } if id == resume_id => {
                    anyhow::bail!(message);
                }
                _ => {}
            }
        }

        let live = Rc::new(DaemonSession::new(
            attached.clone(),
            session.reader.into_inner().into_inner(),
            session.writer.into_inner(),
            session.next_request_id.load(Ordering::Relaxed),
        ));
        self.sessions.borrow_mut().insert(attached, live.clone());
        Ok(live)
    }

    async fn emit_text(&self, session_id: &str, text: String) {
        let notif = acp::SessionNotification::new(
            acp::SessionId::new(session_id),
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(
                acp::ContentBlock::Text(acp::TextContent::new(text)),
            )),
        );
        let _ = self.gateway.session_notification(notif).await;
    }

    fn prompt_text(args: &acp::PromptRequest) -> String {
        let mut parts = Vec::new();
        for block in &args.prompt {
            if let acp::ContentBlock::Text(t) = block {
                parts.push(t.text.clone());
            }
        }
        parts.join("\n")
    }

    // ── Tool lifecycle helpers (Grok typed ACP) ────────────────
    /// Emit a `ToolCall` notification with Pending status. Face
    /// `AcpUpdateTracker::handle_update()` creates a scrollback entry.
    async fn emit_tool_call(&self, session_id: &str, tool_id: &str, name: &str) {
        let _ = self
            .gateway
            .session_notification(acp::SessionNotification::new(
                SessionId::new(session_id),
                acp::SessionUpdate::ToolCall(
                    acp::ToolCall::new(
                        ToolCallId::new(tool_id),
                        Self::tool_title(name, None),
                    )
                    .status(ToolCallStatus::Pending)
                    .kind(Self::tool_kind(name)),
                ),
            ))
            .await;
    }

    /// Emit a `ToolCallUpdate` with the given status and optional raw_input.
    async fn emit_tool_update(
        &self,
        session_id: &str,
        tool_id: &str,
        name: &str,
        status: ToolCallStatus,
        raw_input: Option<serde_json::Value>,
    ) {
        let fields = ToolCallUpdateFields::new()
            .status(status)
            .title(Self::tool_title(name, raw_input.as_ref()))
            .kind(Self::tool_kind(name));
        let fields = if let Some(input) = raw_input {
            fields.raw_input(input)
        } else {
            fields
        };
        let _ = self
            .gateway
            .session_notification(acp::SessionNotification::new(
                SessionId::new(session_id),
                acp::SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                    ToolCallId::new(tool_id),
                    fields,
                )),
            ))
            .await;
    }

    /// Emit a `ToolCallUpdate` with Completed/Failed status, content blocks,
    /// and raw output. Content blocks drive the Face tool result rendering.
    async fn emit_tool_done(
        &self,
        session_id: &str,
        tool_id: &str,
        name: &str,
        output: &str,
        error: &Option<String>,
        raw_input: Option<serde_json::Value>,
    ) {
        let status = if error.is_some() {
            ToolCallStatus::Failed
        } else {
            ToolCallStatus::Completed
        };
        // Build raw_output as ToolOutput::Bash so Face tracker
        // extract_bash_output_from_value finds the output bytes.
        let exit_code = if error.is_some() { 1 } else { 0 };
        let raw_output = Some(serde_json::json!({
            "type": "Bash",
            "output": output.as_bytes(),
            "exit_code": exit_code,
            "command": name,
            "description": null,
            "timed_out": false,
            "truncated": false,
            "signal": null,
            "current_dir": "",
            "output_file": "",
            "total_bytes": output.len(),
            "output_delta": null,
            "was_bare_echo": false,
        }));
        let fields = ToolCallUpdateFields::new()
            .status(status)
            .title(Self::tool_title(name, raw_input.as_ref()))
            .kind(Self::tool_kind(name))
            .content(Some(vec![
                acp::ContentBlock::Text(acp::TextContent::new(output)).into(),
            ]))
            .raw_output(raw_output);
        let fields = if let Some(input) = raw_input {
            fields.raw_input(input)
        } else {
            fields
        };
        let _ = self
            .gateway
            .session_notification(acp::SessionNotification::new(
                SessionId::new(session_id),
                acp::SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                    ToolCallId::new(tool_id),
                    fields,
                )),
            ))
            .await;
    }

    /// Emit a tool_call_update for GeneratedImage.
    async fn emit_generated_image(
        &self,
        session_id: &str,
        tool_id: &str,
        path: &str,
        output_format: &str,
        revised_prompt: Option<&str>,
    ) {
        let text = format!(
            "Generated image: {path} ({output_format}){}",
            revised_prompt
                .map(|rp| format!("\nRevised prompt: {rp}"))
                .unwrap_or_default()
        );
        let fields = ToolCallUpdateFields::new()
            .status(ToolCallStatus::Completed)
            .content(Some(vec![
                acp::ContentBlock::Text(acp::TextContent::new(text)).into(),
            ]));
        let _ = self
            .gateway
            .session_notification(acp::SessionNotification::new(
                SessionId::new(session_id),
                acp::SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(
                    ToolCallId::new(tool_id),
                    fields,
                )),
            ))
            .await;
    }

    /// Parse accumulated tool input as JSON and return the Value.
    fn accumulated_raw_input(&self, tool_id: &str) -> Option<serde_json::Value> {
        let buffer = self.tool_inputs.borrow();
        let input = buffer.get(tool_id)?;
        serde_json::from_str(input).ok()
    }

    /// Emit a session-info-update for renamed sessions.
    async fn emit_session_renamed(&self, session_id: &str, title: &str) {
        let _ = self
            .gateway
            .session_notification(acp::SessionNotification::new(
                SessionId::new(session_id),
                acp::SessionUpdate::SessionInfoUpdate(
                    acp::SessionInfoUpdate::new().title(title),
                ),
            ))
            .await;
    }

    /// Title string for a tool, matching stock EventMapper.
    ///
    /// Memory tools use Face's `Memory search:` title convention so
    /// `AcpUpdateTracker` materializes `MemorySearchToolCallBlock` (verb-group
    /// + MemorySearch chrome). ACP has no `ToolKind::MemorySearch`.
    fn tool_title(name: &str, raw_input: Option<&serde_json::Value>) -> String {
        if name.eq_ignore_ascii_case("memory") {
            return Self::memory_search_title(raw_input);
        }
        if name.starts_with("Bash") {
            "Bash".to_string()
        } else if name.starts_with("Read")
            || name.starts_with("Glob")
            || name.starts_with("Grep")
        {
            "Read".to_string()
        } else if name.starts_with("Edit") || name.starts_with("Write") {
            "Edit".to_string()
        } else if name.starts_with("Web") {
            "Web".to_string()
        } else if name.starts_with("Search") {
            "Search".to_string()
        } else {
            name.to_string()
        }
    }

    /// Face tracker matches `title.starts_with("Memory search:")`.
    fn memory_search_title(raw_input: Option<&serde_json::Value>) -> String {
        let detail = raw_input.and_then(|v| {
            v.get("query")
                .or_else(|| v.get("content"))
                .or_else(|| v.get("id"))
                .or_else(|| v.get("action"))
                .and_then(|x| x.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
        });
        match detail {
            Some(d) => format!("Memory search: \"{d}\""),
            None => "Memory search: ".to_string(),
        }
    }

    /// Kind enum for a tool, matching stock EventMapper.
    ///
    /// Memory stays `Other` so the title-based MemorySearch arm in Face
    /// tracker wins (a bare `ToolKind::Search` would become a grep Search
    /// block instead). ACP has no `ToolKind::MemorySearch`.
    fn tool_kind(name: &str) -> acp::ToolKind {
        if name.starts_with("Bash") {
            acp::ToolKind::Execute
        } else if name.starts_with("Read")
            || name.starts_with("Glob")
            || name.starts_with("Grep")
        {
            acp::ToolKind::Read
        } else if name.starts_with("Edit") || name.starts_with("Write") {
            acp::ToolKind::Edit
        } else if name.starts_with("Web") {
            acp::ToolKind::Fetch
        } else {
            acp::ToolKind::Other
        }
    }
}

#[async_trait(?Send)]
impl acp::Agent for NextCodeFaceAgent {
    async fn initialize(
        &self,
        _args: acp::InitializeRequest,
    ) -> acp::Result<acp::InitializeResponse> {
        // Face treats empty authMethods as fail-closed → Grok login screen.
        // Advertise a non-interactive method so the pager skips Grok OAuth;
        // real provider login stays on next-code (serve bootstrap / `next-code login`).
        let caps = acp::AgentCapabilities::default().load_session(true);
        let auth = acp::AuthMethod::Agent(
            acp::AuthMethodAgent::new(acp::AuthMethodId::new("xai.api_key"), "Next Code")
                .description("Provider credentials owned by the next-code daemon"),
        );
        Ok(acp::InitializeResponse::new(acp::ProtocolVersion::V1)
            .agent_capabilities(caps)
            .auth_methods(vec![auth]))
    }

    async fn authenticate(
        &self,
        _args: acp::AuthenticateRequest,
    ) -> acp::Result<acp::AuthenticateResponse> {
        Ok(acp::AuthenticateResponse::new())
    }

    async fn new_session(
        &self,
        args: acp::NewSessionRequest,
    ) -> acp::Result<acp::NewSessionResponse> {
        let cwd = args.cwd;
        match self.create_session(cwd).await {
            Ok(session) => Ok(acp::NewSessionResponse::new(acp::SessionId::new(
                session.session_id.clone(),
            ))),
            Err(err) => Err(acp::Error::internal_error().data(err.to_string())),
        }
    }

    async fn load_session(
        &self,
        args: acp::LoadSessionRequest,
    ) -> acp::Result<acp::LoadSessionResponse> {
        let id = args.session_id.to_string();
        match self.attach_session(id).await {
            Ok(_) => Ok(acp::LoadSessionResponse::new()),
            Err(err) => Err(acp::Error::internal_error().data(err.to_string())),
        }
    }

    async fn prompt(&self, args: acp::PromptRequest) -> acp::Result<acp::PromptResponse> {
        let session_id = args.session_id.to_string();
        let session = self.sessions.borrow().get(&session_id).cloned();
        let Some(session) = session else {
            return Err(
                acp::Error::invalid_params().data(format!("Unknown session id: {session_id}"))
            );
        };

        if session
            .prompt_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(acp::Error::internal_error()
                .data(format!("Session {session_id} already processing a prompt")));
        }

        let text = Self::prompt_text(&args);
        let prompt_id = session.next_id();
        if let Err(err) = session
            .send(&Request::Message {
                id: prompt_id,
                content: text,
                images: Vec::new(),
                system_reminder: None,
            })
            .await
        {
            session.prompt_running.store(false, Ordering::SeqCst);
            return Err(acp::Error::internal_error().data(err.to_string()));
        }

        let stop = loop {
            let event = match session.read_event().await {
                Ok(e) => e,
                Err(err) => {
                    session.prompt_running.store(false, Ordering::SeqCst);
                    return Err(acp::Error::internal_error().data(err.to_string()));
                }
            };
            match event {
                ServerEvent::TextDelta { text } | ServerEvent::TextReplace { text } => {
                    self.emit_text(&session_id, text).await;
                }
                // Tool lifecycle — typed ACP (Grok way)
                ServerEvent::ToolStart { id, name } => {
                    *self.current_tool_id.borrow_mut() = Some(id.clone());
                    self.tool_inputs.borrow_mut().entry(id.clone()).or_default();
                    self.emit_tool_call(&session_id, &id, &name).await;
                }
                ServerEvent::ToolInput { delta } => {
                    // Accumulate input delta into buffer for current tool
                    // ToolInput has no id — use current_tool_id (mirrors EventMapper)
                    let tid = self.current_tool_id.borrow().clone();
                    if let Some(tid) = tid {
                        self.tool_inputs.borrow_mut()
                            .entry(tid)
                            .or_default()
                            .push_str(&delta);
                    }
                }
                ServerEvent::ToolExec { id, name } => {
                    *self.current_tool_id.borrow_mut() = Some(id.clone());
                    let raw_input = self.accumulated_raw_input(&id);
                    self.emit_tool_update(&session_id, &id, &name, ToolCallStatus::InProgress, raw_input)
                        .await;
                }
                ServerEvent::ToolDone {
                    id,
                    name,
                    output,
                    error,
                } => {
                    let raw_input = self.accumulated_raw_input(&id);
                    self.emit_tool_done(&session_id, &id, &name, &output, &error, raw_input)
                        .await;
                    self.tool_inputs.borrow_mut().remove(&id);
                }
                ServerEvent::GeneratedImage {
                    id,
                    path,
                    output_format,
                    revised_prompt,
                    ..
                } => {
                    self.emit_generated_image(
                        &session_id,
                        &id,
                        &path,
                        &output_format,
                        revised_prompt.as_deref(),
                    )
                    .await;
                }
                ServerEvent::Compaction { trigger, .. } => {
                    self.emit_text(
                        &session_id,
                        format!("\n[Context compacted: {trigger}]\n"),
                    )
                    .await;
                }
                ServerEvent::SessionRenamed {
                    display_title, ..
                } => {
                    self.emit_session_renamed(&session_id, &display_title)
                        .await;
                }
                ServerEvent::Done { id } if id == prompt_id => break acp::StopReason::EndTurn,
                ServerEvent::Error { id, message, .. } if id == prompt_id => {
                    self.emit_text(&session_id, format!("Error: {message}"))
                        .await;
                    break acp::StopReason::EndTurn;
                }
                _ => {}
            }
        };

        session.prompt_running.store(false, Ordering::SeqCst);
        Ok(acp::PromptResponse::new(stop))
    }

    async fn cancel(&self, args: acp::CancelNotification) -> acp::Result<()> {
        let session_id = args.session_id.to_string();
        if let Some(session) = self.sessions.borrow().get(&session_id).cloned() {
            let cancel_id = session.next_id();
            let _ = session.send(&Request::Cancel { id: cancel_id }).await;
        }
        Ok(())
    }

    async fn set_session_model(
        &self,
        args: acp::SetSessionModelRequest,
    ) -> acp::Result<acp::SetSessionModelResponse> {
        let model_id = args.model_id.to_string();
        let session_id = args.session_id.to_string();
        let session = self.sessions.borrow().get(&session_id).cloned();
        let Some(session) = session else {
            return Err(
                acp::Error::invalid_params().data(format!("Unknown session: {session_id}"))
            );
        };
        let req_id = session.next_id();
        session
            .send(&Request::SetModel {
                id: req_id,
                model: model_id,
            })
            .await
            .map_err(|e| acp::Error::internal_error().data(e.to_string()))?;
        Ok(acp::SetSessionModelResponse::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_title_bash() {
        assert_eq!(NextCodeFaceAgent::tool_title("Bash", None), "Bash");
        assert_eq!(
            NextCodeFaceAgent::tool_title("BashDescription", None),
            "Bash"
        );
    }

    #[test]
    fn test_tool_title_read() {
        assert_eq!(NextCodeFaceAgent::tool_title("Read", None), "Read");
        assert_eq!(NextCodeFaceAgent::tool_title("Glob", None), "Read");
        assert_eq!(NextCodeFaceAgent::tool_title("Grep", None), "Read");
    }

    #[test]
    fn test_tool_title_edit() {
        assert_eq!(NextCodeFaceAgent::tool_title("Edit", None), "Edit");
        assert_eq!(NextCodeFaceAgent::tool_title("Write", None), "Edit");
    }

    #[test]
    fn test_tool_title_web() {
        assert_eq!(NextCodeFaceAgent::tool_title("WebSearch", None), "Web");
        assert_eq!(NextCodeFaceAgent::tool_title("WebFetch", None), "Web");
    }

    #[test]
    fn test_tool_title_unknown() {
        assert_eq!(NextCodeFaceAgent::tool_title("Unknown", None), "Unknown");
        assert_eq!(
            NextCodeFaceAgent::tool_title("SomeNewTool", None),
            "SomeNewTool"
        );
    }

    #[test]
    fn test_tool_title_memory_uses_face_memory_search_prefix() {
        assert!(
            NextCodeFaceAgent::tool_title("memory", None).starts_with("Memory search:")
        );
        let input = serde_json::json!({ "action": "search", "query": "auth prefs" });
        assert_eq!(
            NextCodeFaceAgent::tool_title("memory", Some(&input)),
            "Memory search: \"auth prefs\""
        );
        assert_eq!(
            NextCodeFaceAgent::tool_title("Memory", Some(&input)),
            "Memory search: \"auth prefs\""
        );
    }

    #[test]
    fn test_tool_kind_bash() {
        assert_eq!(NextCodeFaceAgent::tool_kind("Bash"), acp::ToolKind::Execute);
        assert_eq!(
            NextCodeFaceAgent::tool_kind("BashDescription"),
            acp::ToolKind::Execute
        );
    }

    #[test]
    fn test_tool_kind_read() {
        assert_eq!(NextCodeFaceAgent::tool_kind("Read"), acp::ToolKind::Read);
        assert_eq!(NextCodeFaceAgent::tool_kind("Grep"), acp::ToolKind::Read);
    }

    #[test]
    fn test_tool_kind_edit() {
        assert_eq!(NextCodeFaceAgent::tool_kind("Edit"), acp::ToolKind::Edit);
    }

    #[test]
    fn test_tool_kind_web() {
        assert_eq!(
            NextCodeFaceAgent::tool_kind("WebSearch"),
            acp::ToolKind::Fetch
        );
    }

    #[test]
    fn test_tool_kind_memory_stays_other_for_title_route() {
        // Face MemorySearch chrome is title-driven; Search kind would become
        // a grep Search block instead.
        assert_eq!(NextCodeFaceAgent::tool_kind("memory"), acp::ToolKind::Other);
    }

    #[test]
    fn test_tool_kind_fallback() {
        assert_eq!(NextCodeFaceAgent::tool_kind("Unknown"), acp::ToolKind::Other);
        assert_eq!(NextCodeFaceAgent::tool_kind("Search"), acp::ToolKind::Other);
    }
}
