//! Per-language LSP server manager: start/stop, didOpen, requests, diagnostics.

use super::client::LspClient;
use super::config::{LspServerConfig, load_server_configs};
use super::uri::path_to_uri;
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

const MAX_OPEN_FILES: usize = 50;
pub const MAX_FILE_BYTES: u64 = 10_000_000;

struct ServerSlot {
    name: String,
    config: LspServerConfig,
    client: Arc<LspClient>,
    started: bool,
    crash_count: u32,
}

pub struct LspManager {
    workspace: PathBuf,
    servers: HashMap<String, ServerSlot>,
    extension_map: HashMap<String, String>,
    open_files: HashMap<String, String>,
    open_order: Vec<String>,
    diagnostics: HashMap<String, Value>,
}

impl LspManager {
    pub fn new(workspace: PathBuf) -> Self {
        let configs = load_server_configs(&workspace);
        Self::from_configs(workspace, configs)
    }

    pub fn from_configs(workspace: PathBuf, configs: HashMap<String, LspServerConfig>) -> Self {
        let mut servers = HashMap::new();
        let mut extension_map = HashMap::new();

        for (name, config) in configs {
            for ext in config.extension_to_language.keys() {
                let normalized = if ext.starts_with('.') {
                    ext.to_ascii_lowercase()
                } else {
                    format!(".{}", ext.to_ascii_lowercase())
                };
                extension_map
                    .entry(normalized)
                    .or_insert_with(|| name.clone());
            }
            let client = LspClient::new(name.clone());
            servers.insert(
                name.clone(),
                ServerSlot {
                    name,
                    config,
                    client,
                    started: false,
                    crash_count: 0,
                },
            );
        }

        Self {
            workspace,
            servers,
            extension_map,
            open_files: HashMap::new(),
            open_order: Vec::new(),
            diagnostics: HashMap::new(),
        }
    }

    pub fn configured_server_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.servers.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn server_for_path(&self, path: &Path) -> Option<&str> {
        let ext = path.extension()?.to_str()?;
        let key = format!(".{}", ext.to_ascii_lowercase());
        self.extension_map.get(&key).map(String::as_str)
    }

    async fn ensure_started(&mut self, server_name: &str) -> Result<Arc<LspClient>> {
        let slot = self
            .servers
            .get_mut(server_name)
            .ok_or_else(|| anyhow!("unknown LSP server '{server_name}'"))?;

        if slot.started && slot.client.is_initialized() {
            return Ok(Arc::clone(&slot.client));
        }

        let max_restarts = slot.config.max_restarts.unwrap_or(3);
        if slot.crash_count > max_restarts {
            return Err(anyhow!(
                "LSP server '{server_name}' exceeded max restarts ({max_restarts})"
            ));
        }

        let workspace = slot
            .config
            .workspace_folder
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.workspace.clone());
        let workspace_uri = path_to_uri(&workspace);
        let folder_name = workspace
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("workspace");

        let client = Arc::clone(&slot.client);
        client
            .on_notification("textDocument/publishDiagnostics", move |params| {
                if let Some(uri) = params
                    .get("uri")
                    .and_then(|u| u.as_str())
                    .map(str::to_owned)
                {
                    store_diagnostics(&uri, params);
                }
            })
            .await;

        if let Err(err) = client
            .start(
                &slot.config.command,
                &slot.config.args,
                &slot.config.env,
                &workspace,
            )
            .await
        {
            slot.crash_count += 1;
            return Err(err);
        }

        let init_params = json!({
            "processId": std::process::id(),
            "clientInfo": {
                "name": "next-code",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "rootPath": workspace.to_string_lossy(),
            "rootUri": workspace_uri,
            "workspaceFolders": [{
                "uri": workspace_uri,
                "name": folder_name,
            }],
            "capabilities": {
                "workspace": {
                    "configuration": false,
                    "workspaceFolders": false,
                },
                "textDocument": {
                    "synchronization": {
                        "dynamicRegistration": false,
                        "willSave": false,
                        "willSaveWaitUntil": false,
                        "didSave": true,
                    },
                    "definition": { "linkSupport": true },
                    "references": {},
                    "hover": {
                        "contentFormat": ["markdown", "plaintext"],
                    },
                    "documentSymbol": {
                        "hierarchicalDocumentSymbolSupport": true,
                    },
                    "implementation": { "linkSupport": true },
                    "publishDiagnostics": {},
                    "callHierarchy": {},
                },
                "window": {
                    "workDoneProgress": false,
                },
            },
            "initializationOptions": slot.config.initialization_options.clone().unwrap_or(json!({})),
        });

        let timeout_ms = slot.config.startup_timeout_ms.unwrap_or(30_000);
        match tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            client.initialize(init_params),
        )
        .await
        {
            Ok(Ok(_)) => {
                slot.started = true;
                Ok(client)
            }
            Ok(Err(err)) => {
                slot.crash_count += 1;
                let _ = client.stop().await;
                Err(err)
            }
            Err(_) => {
                slot.crash_count += 1;
                let _ = client.stop().await;
                Err(anyhow!(
                    "LSP server '{server_name}' initialize timed out after {timeout_ms}ms"
                ))
            }
        }
    }

    pub async fn open_file(&mut self, path: &Path, content: &str) -> Result<()> {
        let server_name = self
            .server_for_path(path)
            .ok_or_else(|| no_server_err(path))?
            .to_string();

        let client = self.ensure_started(&server_name).await?;
        let uri = path_to_uri(path);
        if self.open_files.contains_key(&uri) {
            return Ok(());
        }

        while self.open_order.len() >= MAX_OPEN_FILES {
            if let Some(old_uri) = self.open_order.first().cloned() {
                let _ = self.close_file_uri(&old_uri).await;
            } else {
                break;
            }
        }

        let language_id = self
            .servers
            .get(&server_name)
            .and_then(|s| s.config.language_id_for_path(path))
            .unwrap_or_else(|| "plaintext".into());

        client
            .notify(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": uri,
                        "languageId": language_id,
                        "version": 1,
                        "text": content,
                    }
                }),
            )
            .await?;

        self.open_files.insert(uri.clone(), server_name);
        self.open_order.push(uri);
        Ok(())
    }

    pub fn is_file_open(&self, path: &Path) -> bool {
        self.open_files.contains_key(&path_to_uri(path))
    }

    async fn close_file_uri(&mut self, uri: &str) -> Result<()> {
        if let Some(server_name) = self.open_files.remove(uri) {
            self.open_order.retain(|u| u != uri);
            if let Some(slot) = self.servers.get(&server_name)
                && slot.started
            {
                let _ = slot
                    .client
                    .notify(
                        "textDocument/didClose",
                        json!({ "textDocument": { "uri": uri } }),
                    )
                    .await;
            }
            self.diagnostics.remove(uri);
        }
        Ok(())
    }

    pub async fn send_request(
        &mut self,
        path: &Path,
        method: &str,
        params: Value,
    ) -> Result<Value> {
        let server_name = self
            .server_for_path(path)
            .ok_or_else(|| no_server_err(path))?
            .to_string();
        let client = self.ensure_started(&server_name).await?;
        client.request(method, params).await
    }

    pub fn diagnostics_for(&mut self, path: &Path) -> Value {
        drain_stored_diagnostics_into(&mut self.diagnostics);
        let uri = path_to_uri(path);
        self.diagnostics
            .get(&uri)
            .cloned()
            .unwrap_or_else(|| json!({ "uri": uri, "diagnostics": [] }))
    }

    pub async fn shutdown_all(&mut self) -> Result<()> {
        let uris: Vec<_> = self.open_files.keys().cloned().collect();
        for uri in uris {
            let _ = self.close_file_uri(&uri).await;
        }
        for slot in self.servers.values_mut() {
            if slot.started {
                let _ = slot.client.stop().await;
                slot.started = false;
            }
        }
        self.diagnostics.clear();
        Ok(())
    }
}

fn no_server_err(path: &Path) -> anyhow::Error {
    anyhow!(
        "No LSP server available for file type: {}",
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_else(|| "(none)".into())
    )
}

static DIAG_INBOX: OnceLock<std::sync::Mutex<HashMap<String, Value>>> = OnceLock::new();

fn diag_inbox() -> &'static std::sync::Mutex<HashMap<String, Value>> {
    DIAG_INBOX.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn store_diagnostics(uri: &str, params: Value) {
    if let Ok(mut guard) = diag_inbox().lock() {
        guard.insert(uri.to_string(), params);
    }
}

fn drain_stored_diagnostics_into(target: &mut HashMap<String, Value>) {
    if let Ok(mut guard) = diag_inbox().lock() {
        for (uri, params) in guard.drain() {
            target.insert(uri, params);
        }
    }
}

static GLOBAL: OnceLock<Mutex<Option<(PathBuf, LspManager)>>> = OnceLock::new();

pub async fn global_manager(workspace: PathBuf) -> tokio::sync::MutexGuard<'static, Option<(PathBuf, LspManager)>> {
    let cell = GLOBAL.get_or_init(|| Mutex::new(None));
    let mut guard = cell.lock().await;
    let needs_new = match guard.as_ref() {
        Some((ws, _)) => ws != &workspace,
        None => true,
    };
    if needs_new {
        if let Some((_, mut old)) = guard.take() {
            let _ = old.shutdown_all().await;
        }
        *guard = Some((workspace.clone(), LspManager::new(workspace)));
    }
    guard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_extension_to_server() {
        let mut configs = HashMap::new();
        configs.insert(
            "fake".into(),
            LspServerConfig {
                command: "fake".into(),
                args: vec![],
                extension_to_language: HashMap::from([(".rs".into(), "rust".into())]),
                env: HashMap::new(),
                initialization_options: None,
                workspace_folder: None,
                startup_timeout_ms: None,
                max_restarts: None,
            },
        );
        let mgr = LspManager::from_configs(PathBuf::from("."), configs);
        assert_eq!(mgr.server_for_path(Path::new("a.rs")), Some("fake"));
        assert_eq!(mgr.server_for_path(Path::new("a.py")), None);
    }
}
