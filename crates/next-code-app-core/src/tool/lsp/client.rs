//! Minimal JSON-RPC 2.0 LSP client over stdio (`Content-Length` framing).

use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;

type PendingMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>;
type NotificationHandlers = Arc<Mutex<HashMap<String, Arc<dyn Fn(Value) + Send + Sync>>>>;

enum Outbound {
    Frame(Vec<u8>),
}

pub struct LspClient {
    name: String,
    child: Mutex<Option<Child>>,
    write_tx: Mutex<Option<mpsc::UnboundedSender<Outbound>>>,
    next_id: AtomicU64,
    pending: PendingMap,
    notification_handlers: NotificationHandlers,
    reader_task: Mutex<Option<JoinHandle<()>>>,
    writer_task: Mutex<Option<JoinHandle<()>>>,
    stopping: AtomicBool,
    initialized: AtomicBool,
}

impl LspClient {
    pub fn new(name: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            name: name.into(),
            child: Mutex::new(None),
            write_tx: Mutex::new(None),
            next_id: AtomicU64::new(1),
            pending: Arc::new(Mutex::new(HashMap::new())),
            notification_handlers: Arc::new(Mutex::new(HashMap::new())),
            reader_task: Mutex::new(None),
            writer_task: Mutex::new(None),
            stopping: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
        })
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    pub async fn on_notification<F>(&self, method: &str, handler: F)
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        self.notification_handlers
            .lock()
            .await
            .insert(method.to_string(), Arc::new(handler));
    }

    pub async fn start(
        self: &Arc<Self>,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
        cwd: &std::path::Path,
    ) -> Result<()> {
        self.stopping.store(false, Ordering::SeqCst);

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(cwd)
            .kill_on_drop(true);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        for (k, v) in env {
            cmd.env(k, v);
        }

        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to spawn LSP server '{}'", self.name))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("LSP server '{}' missing stdout", self.name))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("LSP server '{}' missing stdin", self.name))?;
        let stderr = child.stderr.take();

        if let Some(stderr) = stderr {
            let name = self.name.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        next_code_logging::debug(&format!("[lsp:{name}] {line}"));
                    }
                }
            });
        }

        let (tx, rx) = mpsc::unbounded_channel();
        *self.write_tx.lock().await = Some(tx.clone());
        *self.child.lock().await = Some(child);

        let writer = tokio::spawn(async move {
            writer_loop(stdin, rx).await;
        });
        *self.writer_task.lock().await = Some(writer);

        let pending = Arc::clone(&self.pending);
        let handlers = Arc::clone(&self.notification_handlers);
        let name = self.name.clone();
        let client = Arc::clone(self);
        let handle = tokio::spawn(async move {
            let stopping = Arc::clone(&client);
            if let Err(err) = read_loop(stdout, pending, handlers, Arc::clone(&client)).await
                && !stopping.stopping.load(Ordering::SeqCst)
            {
                next_code_logging::warn(&format!("[lsp:{name}] reader stopped: {err:#}"));
            }
        });
        *self.reader_task.lock().await = Some(handle);
        Ok(())
    }

    pub async fn initialize(&self, params: Value) -> Result<Value> {
        let result = self.request("initialize", params).await?;
        self.notify("initialized", json!({})).await?;
        self.initialized.store(true, Ordering::SeqCst);
        Ok(result)
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        if let Err(err) = self.write_message(&message).await {
            self.pending.lock().await.remove(&id);
            return Err(err);
        }

        match tokio::time::timeout(std::time::Duration::from_secs(60), rx).await {
            Ok(Ok(Ok(value))) => Ok(value),
            Ok(Ok(Err(err))) => Err(err),
            Ok(Err(_)) => Err(anyhow!("LSP request '{method}' channel closed")),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err(anyhow!("LSP request '{method}' timed out"))
            }
        }
    }

    pub async fn notify(&self, method: &str, params: Value) -> Result<()> {
        let message = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.write_message(&message).await
    }

    async fn write_message(&self, message: &Value) -> Result<()> {
        let body = serde_json::to_vec(message)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        let mut frame = header.into_bytes();
        frame.extend_from_slice(&body);
        let tx = self
            .write_tx
            .lock()
            .await
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("LSP server '{}' not started", self.name))?;
        tx.send(Outbound::Frame(frame))
            .map_err(|_| anyhow!("LSP server '{}' write channel closed", self.name))?;
        Ok(())
    }

    async fn reply(&self, id: Value, result: Value) -> Result<()> {
        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        });
        self.write_message(&message).await
    }

    pub async fn stop(&self) -> Result<()> {
        self.stopping.store(true, Ordering::SeqCst);
        self.initialized.store(false, Ordering::SeqCst);

        let _ = self.request("shutdown", Value::Null).await;
        let _ = self.notify("exit", Value::Null).await;

        *self.write_tx.lock().await = None;

        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        if let Some(handle) = self.reader_task.lock().await.take() {
            handle.abort();
        }
        if let Some(handle) = self.writer_task.lock().await.take() {
            handle.abort();
        }

        let mut pending = self.pending.lock().await;
        for (_, tx) in pending.drain() {
            let _ = tx.send(Err(anyhow!("LSP server '{}' stopped", self.name)));
        }
        Ok(())
    }
}

async fn writer_loop(mut stdin: ChildStdin, mut rx: mpsc::UnboundedReceiver<Outbound>) {
    while let Some(msg) = rx.recv().await {
        match msg {
            Outbound::Frame(bytes) => {
                if stdin.write_all(&bytes).await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn read_loop(
    stdout: impl tokio::io::AsyncRead + Unpin,
    pending: PendingMap,
    handlers: NotificationHandlers,
    client: Arc<LspClient>,
) -> Result<()> {
    let mut reader = BufReader::new(stdout);
    loop {
        let message = read_framed_message(&mut reader).await?;
        dispatch_message(message, &pending, &handlers, &client).await;
    }
}

async fn read_framed_message<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Value> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Err(anyhow!("LSP server closed stdout"));
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            content_length = Some(
                rest.trim()
                    .parse()
                    .context("invalid Content-Length header")?,
            );
        }
    }
    let len = content_length.ok_or_else(|| anyhow!("LSP message missing Content-Length"))?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    serde_json::from_slice(&buf).context("invalid LSP JSON body")
}

async fn dispatch_message(
    message: Value,
    pending: &PendingMap,
    handlers: &NotificationHandlers,
    client: &Arc<LspClient>,
) {
    let has_id = message.get("id").is_some();
    let has_method = message.get("method").is_some();

    if has_id && !has_method {
        let id = match message.get("id") {
            Some(Value::Number(n)) if n.as_u64().is_some() => n.as_u64().unwrap(),
            _ => return,
        };
        if let Some(tx) = pending.lock().await.remove(&id) {
            if let Some(error) = message.get("error") {
                let msg = error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("LSP error");
                let _ = tx.send(Err(anyhow!("{msg}")));
            } else {
                let result = message.get("result").cloned().unwrap_or(Value::Null);
                let _ = tx.send(Ok(result));
            }
        }
        return;
    }

    if let Some(method) = message.get("method").and_then(|m| m.as_str()) {
        let params = message.get("params").cloned().unwrap_or(Value::Null);

        // Answer server→client requests so servers don't hang.
        if let Some(id) = message.get("id").cloned() {
            let result = match method {
                "workspace/configuration" => {
                    let count = params
                        .get("items")
                        .and_then(|i| i.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    Value::Array(vec![Value::Null; count])
                }
                "window/workDoneProgress/create" => Value::Null,
                "client/registerCapability" | "client/unregisterCapability" => Value::Null,
                _ => Value::Null,
            };
            let _ = client.reply(id, result).await;
        }

        if let Some(handler) = handlers.lock().await.get(method).cloned() {
            handler(params);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn parses_content_length_frame() {
        let body = br#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#;
        let framed = format!("Content-Length: {}\r\n\r\n", body.len());
        let mut data = framed.into_bytes();
        data.extend_from_slice(body);
        let mut reader = BufReader::new(data.as_slice());
        let msg = read_framed_message(&mut reader).await.unwrap();
        assert_eq!(msg["result"]["ok"], true);
    }
}
