//! Stub of upstream `xai-grok-shell::leader` — ACP leader cluster types the pager imports.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch};

use crate::auth::GrokComConfig;

pub const LEADER_SOCKET_ENV: &str = "GROK_LEADER_SOCKET";
pub const LEADER_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct LeaderEnvUrls {
    pub grok_ws_url: String,
    pub grok_ws_origin: String,
}

impl From<&GrokComConfig> for LeaderEnvUrls {
    fn from(c: &GrokComConfig) -> Self {
        Self {
            grok_ws_url: c.grok_ws_url.clone(),
            grok_ws_origin: c.grok_ws_origin.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientMode {
    Headless,
    Stdio,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub yolo_mode: bool,
    #[serde(default)]
    pub auto_mode: bool,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub client_version: Option<String>,
    #[serde(default)]
    pub code_nav_enabled: bool,
    #[serde(default)]
    pub terminal: bool,
    #[serde(default)]
    pub fs_read: bool,
    #[serde(default)]
    pub fs_write: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownReason {
    Manual,
    AutoUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected { generation: u64 },
    Reconnecting { attempt: u32 },
    Failed { error: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectPolicy {
    Unbounded,
    Bounded { max_attempts: u32 },
}

impl ReconnectPolicy {
    pub fn bounded() -> Self {
        Self::Bounded { max_attempts: 5 }
    }

    pub fn unbounded() -> Self {
        Self::Unbounded
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("leader stub: {0}")]
    Stub(String),
    #[error("cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectReason {
    ConnectionLost,
    LeaderShutdown,
    ClientDrop,
}

#[derive(Debug, Clone, Default)]
pub struct LeaderRegistration {
    pub client_id: u64,
}

#[derive(Debug, Default)]
pub struct LeaderClient;

impl LeaderClient {
    pub fn send(&self, _payload: String) -> Result<(), ConnectionError> {
        Ok(())
    }

    pub fn registration(&self) -> &LeaderRegistration {
        static REG: LeaderRegistration = LeaderRegistration { client_id: 0 };
        &REG
    }

    pub async fn recv(&mut self) -> Option<String> {
        None
    }

    pub fn shutting_down_reason(&self) -> watch::Receiver<Option<ShutdownReason>> {
        watch::channel(None).1
    }

    pub fn into_channels(self) -> (mpsc::UnboundedSender<String>, mpsc::UnboundedReceiver<String>) {
        mpsc::unbounded_channel()
    }

    pub fn into_channels_with_disconnect(
        self,
    ) -> (
        mpsc::UnboundedSender<String>,
        mpsc::UnboundedReceiver<String>,
        watch::Receiver<DisconnectReason>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (_dtx, drx) = watch::channel(DisconnectReason::ConnectionLost);
        (tx, rx, drx)
    }
}

#[derive(Debug, Default)]
pub struct LeaderConnection {
    client: LeaderClient,
}

impl LeaderConnection {
    pub fn send(&self, payload: String) -> Result<(), ConnectionError> {
        self.client.send(payload)
    }

    pub fn registration(&self) -> &LeaderRegistration {
        self.client.registration()
    }

    pub async fn recv(&mut self) -> Option<String> {
        self.client.recv().await
    }

    pub fn shutting_down_reason(&self) -> watch::Receiver<Option<ShutdownReason>> {
        self.client.shutting_down_reason()
    }

    pub fn into_channels(self) -> (mpsc::UnboundedSender<String>, mpsc::UnboundedReceiver<String>) {
        self.client.into_channels()
    }

    pub fn into_channels_with_disconnect(
        self,
    ) -> (
        mpsc::UnboundedSender<String>,
        mpsc::UnboundedReceiver<String>,
        watch::Receiver<DisconnectReason>,
    ) {
        self.client.into_channels_with_disconnect()
    }
}

pub struct LeaderReconnector {
    status_tx: watch::Sender<ConnectionStatus>,
    next_generation: AtomicU64,
}

impl LeaderReconnector {
    pub fn new(
        _client_type: impl Into<String>,
        _mode: ClientMode,
        _env_urls: LeaderEnvUrls,
        _capabilities: ClientCapabilities,
        status_tx: watch::Sender<ConnectionStatus>,
    ) -> Self {
        Self {
            status_tx,
            next_generation: AtomicU64::new(1),
        }
    }

    pub fn status_channel() -> (watch::Sender<ConnectionStatus>, watch::Receiver<ConnectionStatus>)
    {
        watch::channel(ConnectionStatus::Connected { generation: 0 })
    }

    pub fn notify_connected(&self) {
        let generation = self.next_generation.fetch_add(1, Ordering::Relaxed);
        let _ = self
            .status_tx
            .send(ConnectionStatus::Connected { generation });
    }

    pub async fn reconnect(
        &self,
        policy: ReconnectPolicy,
        cancel: &tokio_util::sync::CancellationToken,
    ) -> Result<
        (
            mpsc::UnboundedSender<String>,
            mpsc::UnboundedReceiver<String>,
            watch::Receiver<DisconnectReason>,
        ),
        ConnectionError,
    > {
        let max_attempts = match policy {
            ReconnectPolicy::Unbounded => u32::MAX,
            ReconnectPolicy::Bounded { max_attempts } => max_attempts.max(1),
        };
        let mut attempt = 0u32;
        while attempt < max_attempts {
            if cancel.is_cancelled() {
                return Err(ConnectionError::Cancelled);
            }
            attempt += 1;
            let _ = self
                .status_tx
                .send(ConnectionStatus::Reconnecting { attempt });
            // Stub transport: surface reconnect UI, then fail closed. Real
            // leader IPC belongs in a non-stub shell build.
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        let error = "reconnect not implemented".to_string();
        let _ = self
            .status_tx
            .send(ConnectionStatus::Failed { error: error.clone() });
        Err(ConnectionError::Stub(error))
    }
}

#[derive(Debug, Default)]
pub struct LeaderLock {
    pub socket_path: PathBuf,
}

impl LeaderLock {
    pub fn new(_ws_url: &str) -> Self {
        Self::default()
    }

    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeaderServerMetadata {
    pub pid: u32,
    pub version: String,
}

#[derive(Debug, Clone, Default)]
pub struct LeaderServerControlState;

pub async fn connect_or_spawn(
    _client_type: &str,
    _mode: ClientMode,
    _env_urls: &LeaderEnvUrls,
    _capabilities: ClientCapabilities,
) -> Result<LeaderConnection, ConnectionError> {
    Ok(LeaderConnection::default())
}

pub async fn kill_stale_reachable_leaders(_reason: &'static str) {}

pub async fn run_leader_server(
    _socket_path: PathBuf,
    _shutdown: watch::Receiver<ShutdownReason>,
) -> Result<(), ConnectionError> {
    Ok(())
}

pub mod protocol {
    pub use super::{ClientCapabilities, ClientMode, ShutdownReason};
}

pub mod transport {
    pub fn listener_is_ready(_path: &std::path::Path) -> bool {
        false
    }
}
