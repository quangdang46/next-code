//! MCP Client - handles communication with a single MCP server (stdio or HTTP).

use super::http::HttpBackend;
use super::protocol::*;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc, oneshot};

enum RequestTransport {
    Stdio {
        pending: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
        writer_tx: mpsc::Sender<String>,
    },
    Http(HttpBackend),
}

/// Shared communication handle for an MCP server.
/// Multiple sessions can hold clones of this and send concurrent requests.
#[derive(Clone)]
pub struct McpHandle {
    pub(crate) name: String,
    request_id: Arc<AtomicU64>,
    transport: Arc<RequestTransport>,
    server_info: Arc<std::sync::RwLock<Option<ServerInfo>>>,
    capabilities: Arc<std::sync::RwLock<ServerCapabilities>>,
    tools: Arc<std::sync::RwLock<Vec<McpToolDef>>>,
}

impl McpHandle {
    /// Send a request and wait for response
    pub async fn request(&self, method: &str, params: Option<Value>) -> Result<JsonRpcResponse> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest::new(id, method, params);

        match self.transport.as_ref() {
            RequestTransport::Stdio { pending, writer_tx } => {
                let (tx, rx) = oneshot::channel();
                {
                    let mut pending = pending.lock().await;
                    pending.insert(id, tx);
                }

                let msg = serde_json::to_string(&request)? + "\n";
                writer_tx
                    .send(msg)
                    .await
                    .context("Failed to send request")?;

                let response = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
                    .await
                    .context("Request timeout")?
                    .context("Channel closed")?;

                if let Some(err) = &response.error {
                    anyhow::bail!("MCP error {}: {}", err.code, err.message);
                }
                Ok(response)
            }
            RequestTransport::Http(http) => {
                let body = serde_json::to_value(&request)?;
                let response = http.request(&body).await?;
                if let Some(err) = &response.error {
                    anyhow::bail!("MCP error {}: {}", err.code, err.message);
                }
                Ok(response)
            }
        }
    }

    async fn notify_initialized(&self) -> Result<()> {
        match self.transport.as_ref() {
            RequestTransport::Stdio { writer_tx, .. } => {
                // Historical stdio path used id:0; keep for compatibility.
                let notif = JsonRpcRequest::new(0, "notifications/initialized", None);
                let msg = serde_json::to_string(&notif)? + "\n";
                writer_tx.send(msg).await?;
                Ok(())
            }
            RequestTransport::Http(http) => http.notify("notifications/initialized", None).await,
        }
    }

    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        let arguments = if arguments.is_null() {
            Value::Object(serde_json::Map::new())
        } else {
            arguments
        };
        let params = ToolCallParams {
            name: name.to_string(),
            arguments,
        };

        let response = self
            .request("tools/call", Some(serde_json::to_value(params)?))
            .await?;

        let result = response.result.context("No result from tool call")?;
        let tool_result: ToolCallResult = serde_json::from_value(result)?;

        Ok(tool_result)
    }

    /// Get the server name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get server info
    pub fn server_info(&self) -> Option<ServerInfo> {
        self.server_info
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Get available tools
    pub fn tools(&self) -> Vec<McpToolDef> {
        self.tools
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Refresh the list of available tools
    pub async fn refresh_tools(&self) -> Result<()> {
        let response = self.request("tools/list", None).await?;

        if let Some(result) = response.result {
            let tools_result: ToolsListResult = serde_json::from_value(result)?;
            *self
                .tools
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = tools_result.tools;
        }

        Ok(())
    }
}

/// MCP Client - owns the transport (stdio child or HTTP) and provides shared handles.
pub struct McpClient {
    handle: McpHandle,
    child: Option<Child>,
}

impl McpClient {
    /// Connect to an MCP server (stdio command or HTTP/SSE URL).
    pub async fn connect(name: String, config: &McpServerConfig) -> Result<Self> {
        if config.is_http() {
            Self::connect_http(name, config).await
        } else {
            Self::connect_stdio(name, config).await
        }
    }

    async fn connect_http(name: String, config: &McpServerConfig) -> Result<Self> {
        let url = config
            .url
            .as_deref()
            .filter(|u| !u.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("HTTP MCP server '{name}' is missing url"))?
            .to_string();

        crate::logging::info(&format!("MCP: Connecting to '{name}' via HTTP {url}"));

        let http = HttpBackend::new(url, &config.headers)?;
        let handle = McpHandle {
            name: name.clone(),
            request_id: Arc::new(AtomicU64::new(1)),
            transport: Arc::new(RequestTransport::Http(http)),
            server_info: Arc::new(std::sync::RwLock::new(None)),
            capabilities: Arc::new(std::sync::RwLock::new(ServerCapabilities::default())),
            tools: Arc::new(std::sync::RwLock::new(Vec::new())),
        };

        let mut client = Self {
            handle,
            child: None,
        };
        client
            .initialize()
            .await
            .with_context(|| format!("MCP HTTP server '{name}' failed to initialize"))?;
        client
            .handle
            .refresh_tools()
            .await
            .with_context(|| format!("MCP HTTP server '{name}' failed to list tools"))?;

        crate::logging::info(&format!(
            "MCP: Connected to '{name}' (HTTP) with {} tools",
            client.handle.tools().len()
        ));
        Ok(client)
    }

    async fn connect_stdio(name: String, config: &McpServerConfig) -> Result<Self> {
        crate::logging::info(&format!(
            "MCP: Connecting to '{}' ({} {:?})",
            name, config.command, config.args
        ));

        let mut env: HashMap<String, String> = std::env::vars().collect();
        env.extend(config.env.clone());

        let mut child = Command::new(&config.command)
            .args(&config.args)
            .envs(&env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn MCP server: {}", config.command))?;

        let stdin = child.stdin.take().context("No stdin")?;
        let stdout = child.stdout.take().context("No stdout")?;
        let stderr = child.stderr.take().context("No stderr")?;

        let server_name = name.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            crate::logging::warn(&format!(
                                "MCP [{}] stderr: {}",
                                server_name, trimmed
                            ));
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let (writer_tx, mut writer_rx) = mpsc::channel::<String>(32);

        let mut stdin = stdin;
        tokio::spawn(async move {
            while let Some(msg) = writer_rx.recv().await {
                if stdin.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        let pending_clone = Arc::clone(&pending);
        let reader_name = name.clone();
        let mut reader = BufReader::new(stdout);
        tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        crate::logging::debug(&format!("MCP [{}]: stdout EOF", reader_name));
                        break;
                    }
                    Ok(_) => {
                        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&line) {
                            if let Some(id) = response.id {
                                let mut pending = pending_clone.lock().await;
                                if let Some(tx) = pending.remove(&id) {
                                    let _ = tx.send(response);
                                }
                            }
                        } else {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                crate::logging::debug(&format!(
                                    "MCP [{}] non-JSON output: {}",
                                    reader_name, trimmed
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        crate::logging::warn(&format!("MCP [{}] read error: {}", reader_name, e));
                        break;
                    }
                }
            }
        });

        let handle = McpHandle {
            name: name.clone(),
            request_id: Arc::new(AtomicU64::new(1)),
            transport: Arc::new(RequestTransport::Stdio { pending, writer_tx }),
            server_info: Arc::new(std::sync::RwLock::new(None)),
            capabilities: Arc::new(std::sync::RwLock::new(ServerCapabilities::default())),
            tools: Arc::new(std::sync::RwLock::new(Vec::new())),
        };

        let mut client = Self {
            handle,
            child: Some(child),
        };

        client
            .initialize()
            .await
            .with_context(|| format!("MCP server '{}' failed to initialize", name))?;

        client
            .handle
            .refresh_tools()
            .await
            .with_context(|| format!("MCP server '{}' failed to list tools", name))?;

        crate::logging::info(&format!(
            "MCP: Connected to '{}' with {} tools",
            name,
            client.handle.tools().len()
        ));

        Ok(client)
    }

    /// Get a shareable handle to this client
    pub fn handle(&self) -> McpHandle {
        self.handle.clone()
    }

    /// Initialize the MCP connection
    async fn initialize(&mut self) -> Result<()> {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "next-code".to_string(),
                version: next_code_build_meta::PKG_VERSION.to_string(),
            },
        };

        let response = self
            .handle
            .request("initialize", Some(serde_json::to_value(params)?))
            .await?;

        if let Some(result) = response.result {
            let init_result: InitializeResult = serde_json::from_value(result)?;
            *self
                .handle
                .server_info
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = init_result.server_info;
            *self
                .handle
                .capabilities
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = init_result.capabilities;
        }

        self.handle.notify_initialized().await?;
        Ok(())
    }

    /// Check if server is still running (stdio child; HTTP always "running").
    pub fn is_running(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(None) => true,
                Ok(Some(_)) => false,
                Err(_) => false,
            },
            None => true,
        }
    }

    /// Shutdown the server
    pub async fn shutdown(&mut self) {
        match self.handle.transport.as_ref() {
            RequestTransport::Stdio { writer_tx, .. } => {
                let _ = writer_tx
                    .send("{\"jsonrpc\":\"2.0\",\"method\":\"shutdown\"}\n".to_string())
                    .await;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(child) = self.child.as_mut() {
                    let _ = child.kill().await;
                }
            }
            RequestTransport::Http(_) => {
                // Stateless HTTP — nothing to tear down beyond dropping the client.
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.handle.name
    }

    pub fn server_info(&self) -> Option<ServerInfo> {
        self.handle.server_info()
    }

    pub fn tools(&self) -> Vec<McpToolDef> {
        self.handle.tools()
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        self.handle.call_tool(name, arguments).await
    }

    pub async fn refresh_tools(&self) -> Result<()> {
        self.handle.refresh_tools().await
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.start_kill();
        }
    }
}
