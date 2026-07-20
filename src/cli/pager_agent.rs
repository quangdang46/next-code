//! In-process ACP Agent that bridges Face (`xai-grok-pager`) to the next-code
//! daemon socket protocol — same brain path as `next-code acp`, without stdio.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use agent_client_protocol as acp;
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
}

impl NextCodeFaceAgent {
    pub(crate) fn new(gateway: AcpGatewaySender<acp::AgentSide>) -> Self {
        Self {
            gateway,
            sessions: RefCell::new(HashMap::new()),
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
        use acp::Client as _;
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
}
