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

/// Bootstrap fields Face needs for Overview floats (model + provider).
struct SessionBootstrap {
    session: Rc<DaemonSession>,
    models: Option<acp::SessionModelState>,
}

struct DaemonSession {
    session_id: String,
    reader: Mutex<BufReader<ReadHalf>>,
    writer: Mutex<WriteHalf>,
    next_request_id: AtomicU64,
    prompt_running: AtomicBool,
    /// Session cwd — required so MemoryManager project graph saves/loads
    /// (same pattern as turn_memory / memory_agent::manager_for_working_dir).
    working_dir: Option<PathBuf>,
}

impl DaemonSession {
    fn new(
        session_id: String,
        reader: ReadHalf,
        writer: WriteHalf,
        next_request_id: u64,
        working_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            session_id,
            reader: Mutex::new(BufReader::new(reader)),
            writer: Mutex::new(writer),
            next_request_id: AtomicU64::new(next_request_id),
            prompt_running: AtomicBool::new(false),
            working_dir,
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

/// Build ACP `SessionModelState` from daemon History fields so Face Overview
/// gets model + context window (not a Context-only chip).
fn session_model_state_from_history(
    provider_model: Option<&str>,
    available_models: &[String],
    provider_name: Option<&str>,
) -> Option<acp::SessionModelState> {
    let current = provider_model
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            available_models
                .iter()
                .map(|s| s.trim())
                .find(|s| !s.is_empty())
                .map(str::to_string)
        })?;

    let mut ids: Vec<String> = Vec::new();
    for id in available_models
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
    {
        if !ids.iter().any(|existing| existing == &id) {
            ids.push(id);
        }
    }
    if !ids.iter().any(|id| id == &current) {
        ids.insert(0, current.clone());
    }

    let available: Vec<acp::ModelInfo> = ids
        .into_iter()
        .map(|id| {
            let mut info = acp::ModelInfo::new(acp::ModelId::new(std::sync::Arc::from(id.as_str())), id.clone());
            if let Some(limit) =
                next_code_provider_core::context_limit_for_model_with_provider(&id, provider_name)
            {
                info = info.meta(
                    serde_json::json!({ "totalContextTokens": limit })
                        .as_object()
                        .cloned(),
                );
            }
            info
        })
        .collect();

    Some(acp::SessionModelState::new(
        acp::ModelId::new(std::sync::Arc::from(current.as_str())),
        available,
    ))
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

    async fn create_session(&self, cwd: PathBuf) -> Result<SessionBootstrap> {
        let (reader, writer) = Self::connect_halves().await?;
        let session = DaemonSession::new(String::new(), reader, writer, 2, Some(cwd.clone()));
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
        let (session_id, provider_name, provider_model, available_models) = match history {
            ServerEvent::History {
                session_id,
                provider_name,
                provider_model,
                available_models,
                ..
            } => (session_id, provider_name, provider_model, available_models),
            other => anyhow::bail!("expected history after session creation, got {other:?}"),
        };

        let models = session_model_state_from_history(
            provider_model.as_deref(),
            &available_models,
            provider_name.as_deref(),
        );

        let live = Rc::new(DaemonSession::new(
            session_id.clone(),
            session.reader.into_inner().into_inner(),
            session.writer.into_inner(),
            session.next_request_id.load(Ordering::Relaxed),
            Some(cwd),
        ));
        self.sessions
            .borrow_mut()
            .insert(session_id.clone(), live.clone());
        if let Some(provider) = provider_name.as_deref().filter(|s| !s.is_empty()) {
            self.emit_provider_name(&session_id, provider).await;
        }
        self.emit_memory_info(&session_id).await;
        self.emit_git_status(&session_id).await;
        // Face TodoPane / Todos float only paint from ACP Plan — bridge disk todos.
        self.emit_todos_plan(&session_id, /*allow_empty=*/ false).await;
        Ok(SessionBootstrap {
            session: live,
            models,
        })
    }

    async fn attach_session(&self, target: String) -> Result<SessionBootstrap> {
        let (reader, writer) = Self::connect_halves().await?;
        let session = DaemonSession::new(String::new(), reader, writer, 2, None);
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
        let mut provider_name: Option<String> = None;
        let mut provider_model: Option<String> = None;
        let mut available_models: Vec<String> = Vec::new();
        loop {
            match session.read_event().await? {
                ServerEvent::Ack { .. } => {}
                ServerEvent::History {
                    session_id,
                    provider_name: hist_provider,
                    provider_model: hist_model,
                    available_models: hist_models,
                    ..
                } => {
                    attached = session_id;
                    if hist_provider.is_some() {
                        provider_name = hist_provider;
                    }
                    if hist_model.is_some() {
                        provider_model = hist_model;
                    }
                    if !hist_models.is_empty() {
                        available_models = hist_models;
                    }
                }
                ServerEvent::Done { id } if id == resume_id => break,
                ServerEvent::Error { id, message, .. } if id == resume_id => {
                    anyhow::bail!(message);
                }
                _ => {}
            }
        }

        let models = session_model_state_from_history(
            provider_model.as_deref(),
            &available_models,
            provider_name.as_deref(),
        );

        let working_dir = crate::session::Session::load(&attached)
            .ok()
            .and_then(|s| s.working_dir)
            .filter(|d| !d.trim().is_empty())
            .map(PathBuf::from);

        let live = Rc::new(DaemonSession::new(
            attached.clone(),
            session.reader.into_inner().into_inner(),
            session.writer.into_inner(),
            session.next_request_id.load(Ordering::Relaxed),
            working_dir,
        ));
        self.sessions.borrow_mut().insert(attached.clone(), live.clone());
        if let Some(provider) = provider_name.as_deref().filter(|s| !s.is_empty()) {
            self.emit_provider_name(&attached, provider).await;
        }
        self.emit_memory_info(&attached).await;
        self.emit_git_status(&attached).await;
        self.emit_todos_plan(&attached, /*allow_empty=*/ false).await;
        Ok(SessionBootstrap {
            session: live,
            models,
        })
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
                    acp::ToolCall::new(ToolCallId::new(tool_id), Self::tool_title(name))
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
            .title(Self::tool_title(name))
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
            .title(Self::tool_title(name))
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

    /// Bridge daemon TokenUsage into Face via ext notification (context + KV floats).
    async fn emit_token_usage(
        &self,
        session_id: &str,
        input: u64,
        output: u64,
        cache_read_input: Option<u64>,
        cache_creation_input: Option<u64>,
    ) {
        let payload = serde_json::json!({
            "sessionId": session_id,
            "input": input,
            "output": output,
            "cacheReadInput": cache_read_input,
            "cacheCreationInput": cache_creation_input,
        });
        let Ok(raw) = serde_json::value::to_raw_value(&payload) else {
            return;
        };
        let _ = self
            .gateway
            .ext_notification(acp::ExtNotification::new(
                "next-code/token_usage",
                std::sync::Arc::from(raw),
            ))
            .await;
    }

    /// Bridge daemon History `provider_name` into Face Overview float.
    async fn emit_provider_name(&self, session_id: &str, provider_name: &str) {
        let payload = serde_json::json!({
            "sessionId": session_id,
            "providerName": provider_name,
        });
        let Ok(raw) = serde_json::value::to_raw_value(&payload) else {
            return;
        };
        let _ = self
            .gateway
            .ext_notification(acp::ExtNotification::new(
                "next-code/provider_name",
                std::sync::Arc::from(raw),
            ))
            .await;
    }

    /// Bridge local MemoryManager counts + activity into Face MemoryActivity float.
    async fn emit_memory_info(&self, session_id: &str) {
        let working_dir = self
            .sessions
            .borrow()
            .get(session_id)
            .and_then(|s| s.working_dir.clone());
        // Copy of memory_agent::manager_for_working_dir — project graph is empty
        // without with_project_dir, so Face float stayed at 🧠 0 after remember.
        let manager = match working_dir.as_ref() {
            Some(dir) => crate::memory::MemoryManager::new().with_project_dir(dir),
            None => crate::memory::MemoryManager::new(),
        };
        let project_count = manager
            .load_project_graph()
            .ok()
            .map(|g| g.memory_count())
            .unwrap_or(0);
        let global_count = manager
            .load_global_graph()
            .ok()
            .map(|g| g.memory_count())
            .unwrap_or(0);
        let total_count = project_count + global_count;
        let activity = crate::memory::get_activity();
        let (activity_summary, show_activity) = match activity.as_ref() {
            Some(a) if a.is_processing() => {
                let summary = match &a.state {
                    crate::memory_types::MemoryState::Embedding => "searching",
                    crate::memory_types::MemoryState::SidecarChecking { .. } => "verifying",
                    crate::memory_types::MemoryState::FoundRelevant { .. } => "ready",
                    crate::memory_types::MemoryState::Extracting { .. } => "saving",
                    crate::memory_types::MemoryState::Maintaining { .. } => "updating",
                    crate::memory_types::MemoryState::ToolAction { .. } => "tool",
                    crate::memory_types::MemoryState::Idle => "working",
                };
                (Some(summary.to_string()), true)
            }
            Some(_) => (Some("idle".to_string()), false),
            None => (None, false),
        };
        // Always emit so Face can clear/update after remember/forget (do not
        // early-return forever on total_count == 0).
        let payload = serde_json::json!({
            "sessionId": session_id,
            "totalCount": total_count,
            "disabled": false,
            "activitySummary": activity_summary,
            "showActivity": show_activity,
        });
        let Ok(raw) = serde_json::value::to_raw_value(&payload) else {
            return;
        };
        let _ = self
            .gateway
            .ext_notification(acp::ExtNotification::new(
                "next-code/memory_info",
                std::sync::Arc::from(raw),
            ))
            .await;
    }

    /// Map next-code session todos → ACP `SessionUpdate::Plan` so Face
    /// `TodoPane` / Todos float paint (classic TUI uses `BusEvent::TodoUpdated`
    /// instead; pager never saw that bus).
    async fn emit_todos_plan(&self, session_id: &str, allow_empty: bool) {
        let todos = crate::todo::load_todos(session_id).unwrap_or_default();
        if todos.is_empty() && !allow_empty {
            return;
        }
        let entries: Vec<acp::PlanEntry> = todos
            .iter()
            .map(plan_entry_from_next_code_todo)
            .collect();
        let _ = self
            .gateway
            .session_notification(acp::SessionNotification::new(
                acp::SessionId::new(session_id),
                acp::SessionUpdate::Plan(acp::Plan::new(entries)),
            ))
            .await;
    }

    /// Bridge git porcelain into Face GitStatus float (same gather as TUI widget).
    async fn emit_git_status(&self, session_id: &str) {
        let Some(info) = gather_git_status_snapshot() else {
            return;
        };
        if !info.is_interesting {
            return;
        }
        let payload = serde_json::json!({
            "sessionId": session_id,
            "branch": info.branch,
            "modified": info.modified,
            "staged": info.staged,
            "untracked": info.untracked,
            "ahead": info.ahead,
            "behind": info.behind,
            "dirtyFiles": info.dirty_files,
        });
        let Ok(raw) = serde_json::value::to_raw_value(&payload) else {
            return;
        };
        let _ = self
            .gateway
            .ext_notification(acp::ExtNotification::new(
                "next-code/git_status",
                std::sync::Arc::from(raw),
            ))
            .await;
    }

    /// Mid-session model switch → Face catalog (Overview + prompt chrome).
    async fn emit_models_update(
        &self,
        model: &str,
        provider_name: Option<&str>,
        available: &[String],
    ) {
        let Some(state) =
            session_model_state_from_history(Some(model), available, provider_name)
        else {
            return;
        };
        let Ok(raw) = serde_json::value::to_raw_value(&state) else {
            return;
        };
        let _ = self
            .gateway
            .ext_notification(acp::ExtNotification::new(
                "x.ai/models/update",
                std::sync::Arc::from(raw),
            ))
            .await;
    }

    /// Title string for a tool, matching stock EventMapper.
    fn tool_title(name: &str) -> String {
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

    /// Kind enum for a tool, matching stock EventMapper.
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
            Ok(boot) => {
                let mut resp =
                    acp::NewSessionResponse::new(acp::SessionId::new(boot.session.session_id.clone()));
                if let Some(models) = boot.models {
                    resp = resp.models(models);
                }
                Ok(resp)
            }
            Err(err) => Err(acp::Error::internal_error().data(err.to_string())),
        }
    }

    async fn load_session(
        &self,
        args: acp::LoadSessionRequest,
    ) -> acp::Result<acp::LoadSessionResponse> {
        let id = args.session_id.to_string();
        match self.attach_session(id).await {
            Ok(boot) => {
                let mut resp = acp::LoadSessionResponse::new();
                if let Some(models) = boot.models {
                    resp = resp.models(models);
                }
                Ok(resp)
            }
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
                    // `todo` tool persists via next-code store + BusEvent for TUI;
                    // Face needs an ACP Plan refresh after each write (or clear).
                    if name.eq_ignore_ascii_case("todo") {
                        self.emit_todos_plan(&session_id, /*allow_empty=*/ true)
                            .await;
                    }
                    // Same for memory: refresh float after remember/list/forget.
                    if name.eq_ignore_ascii_case("memory") {
                        self.emit_memory_info(&session_id).await;
                    }
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
                ServerEvent::TokenUsage {
                    input,
                    output,
                    cache_read_input,
                    cache_creation_input,
                } => {
                    self.emit_token_usage(
                        &session_id,
                        input,
                        output,
                        cache_read_input,
                        cache_creation_input,
                    )
                    .await;
                }
                ServerEvent::ModelChanged {
                    model,
                    provider_name,
                    error: None,
                    ..
                } => {
                    if let Some(provider) = provider_name.as_deref().filter(|s| !s.is_empty()) {
                        self.emit_provider_name(&session_id, provider).await;
                    }
                    self.emit_models_update(&model, provider_name.as_deref(), &[]).await;
                }
                ServerEvent::MemoryActivity { activity } => {
                    crate::memory::apply_remote_activity_snapshot(&activity);
                    self.emit_memory_info(&session_id).await;
                }
                ServerEvent::Done { id } if id == prompt_id => {
                    // Refresh Plan in case compaction / other paths mutated todos
                    // without a `todo` ToolDone this turn.
                    self.emit_todos_plan(&session_id, /*allow_empty=*/ false)
                        .await;
                    break acp::StopReason::EndTurn;
                }
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
        assert_eq!(NextCodeFaceAgent::tool_title("Bash"), "Bash");
        assert_eq!(NextCodeFaceAgent::tool_title("BashDescription"), "Bash");
    }

    #[test]
    fn test_tool_title_read() {
        assert_eq!(NextCodeFaceAgent::tool_title("Read"), "Read");
        assert_eq!(NextCodeFaceAgent::tool_title("Glob"), "Read");
        assert_eq!(NextCodeFaceAgent::tool_title("Grep"), "Read");
    }

    #[test]
    fn test_tool_title_edit() {
        assert_eq!(NextCodeFaceAgent::tool_title("Edit"), "Edit");
        assert_eq!(NextCodeFaceAgent::tool_title("Write"), "Edit");
    }

    #[test]
    fn test_tool_title_web() {
        assert_eq!(NextCodeFaceAgent::tool_title("WebSearch"), "Web");
        assert_eq!(NextCodeFaceAgent::tool_title("WebFetch"), "Web");
    }

    #[test]
    fn test_tool_title_unknown() {
        assert_eq!(NextCodeFaceAgent::tool_title("Unknown"), "Unknown");
        assert_eq!(NextCodeFaceAgent::tool_title("SomeNewTool"), "SomeNewTool");
    }

    #[test]
    fn test_tool_kind_bash() {
        assert_eq!(NextCodeFaceAgent::tool_kind("Bash"), acp::ToolKind::Execute);
        assert_eq!(NextCodeFaceAgent::tool_kind("BashDescription"), acp::ToolKind::Execute);
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
        assert_eq!(NextCodeFaceAgent::tool_kind("WebSearch"), acp::ToolKind::Fetch);
    }

    #[test]
    fn test_tool_kind_fallback() {
        assert_eq!(NextCodeFaceAgent::tool_kind("Unknown"), acp::ToolKind::Other);
        assert_eq!(NextCodeFaceAgent::tool_kind("Search"), acp::ToolKind::Other);
    }

    #[test]
    fn plan_entry_maps_next_code_todo_status_and_priority() {
        let pending = crate::todo::TodoItem {
            content: "wire Plan".into(),
            status: "pending".into(),
            active_form: None,
            priority: "high".into(),
            id: "1".into(),
            group: None,
            confidence: None,
            completion_confidence: None,
            confidence_history: Vec::new(),
            blocked_by: Vec::new(),
            assigned_to: None,
        };
        let entry = plan_entry_from_next_code_todo(&pending);
        assert_eq!(entry.content, "wire Plan");
        assert_eq!(entry.status, acp::PlanEntryStatus::Pending);
        assert_eq!(entry.priority, acp::PlanEntryPriority::High);

        let active = crate::todo::TodoItem {
            status: "in_progress".into(),
            priority: "low".into(),
            ..pending.clone()
        };
        let entry = plan_entry_from_next_code_todo(&active);
        assert_eq!(entry.status, acp::PlanEntryStatus::InProgress);
        assert_eq!(entry.priority, acp::PlanEntryPriority::Low);

        let done = crate::todo::TodoItem {
            status: "cancelled".into(),
            priority: "medium".into(),
            ..pending
        };
        let entry = plan_entry_from_next_code_todo(&done);
        assert_eq!(entry.status, acp::PlanEntryStatus::Completed);
        assert_eq!(entry.priority, acp::PlanEntryPriority::Medium);
    }
}

/// Convert next-code disk/bus `TodoItem` (string status/priority) into the ACP
/// Plan entry shape Face already maps via `todo_item_from_plan_entry`.
fn plan_entry_from_next_code_todo(item: &crate::todo::TodoItem) -> acp::PlanEntry {
    let status = match item.status.as_str() {
        "in_progress" => acp::PlanEntryStatus::InProgress,
        "completed" | "cancelled" => acp::PlanEntryStatus::Completed,
        _ => acp::PlanEntryStatus::Pending,
    };
    let priority = match item.priority.as_str() {
        "high" => acp::PlanEntryPriority::High,
        "low" => acp::PlanEntryPriority::Low,
        _ => acp::PlanEntryPriority::Medium,
    };
    acp::PlanEntry::new(item.content.clone(), priority, status)
}

/// Paste of TUI `gather_git_info_inner` for Face float bridge (no tui dep from Face).
struct GitStatusSnapshot {
    branch: String,
    modified: usize,
    staged: usize,
    untracked: usize,
    ahead: usize,
    behind: usize,
    dirty_files: Vec<String>,
    is_interesting: bool,
}

fn gather_git_status_snapshot() -> Option<GitStatusSnapshot> {
    use std::process::Command;

    let in_repo = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !in_repo {
        return None;
    }

    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let b = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if b.is_empty() {
                    None
                } else {
                    Some(b)
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| "HEAD".to_string());

    let mut modified = 0;
    let mut staged = 0;
    let mut untracked = 0;
    let mut dirty_files = Vec::new();

    if let Ok(output) = Command::new("git").args(["status", "--porcelain"]).output()
        && output.status.success()
    {
        let status = String::from_utf8_lossy(&output.stdout);
        for line in status.lines() {
            if line.len() < 3 {
                continue;
            }
            let index_status = line.as_bytes()[0];
            let worktree_status = line.as_bytes()[1];
            let file_path = line[3..].to_string();
            if index_status == b'?' {
                untracked += 1;
            } else {
                if index_status != b' ' && index_status != b'?' {
                    staged += 1;
                }
                if worktree_status != b' ' && worktree_status != b'?' {
                    modified += 1;
                }
            }
            if dirty_files.len() < 10 {
                dirty_files.push(file_path);
            }
        }
    }

    let (ahead, behind) = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let parts: Vec<&str> = text.split('\t').collect();
                if parts.len() == 2 {
                    Some((
                        parts[0].parse::<usize>().unwrap_or(0),
                        parts[1].parse::<usize>().unwrap_or(0),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or((0, 0));

    let is_interesting =
        modified > 0 || staged > 0 || untracked > 0 || ahead > 0 || behind > 0;
    Some(GitStatusSnapshot {
        branch,
        modified,
        staged,
        untracked,
        ahead,
        behind,
        dirty_files,
        is_interesting,
    })
}
