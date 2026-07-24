//! Real LSP tool — Claude-parity operations over local language servers.
//!
//! Starts language servers lazily per file extension (built-in defaults when
//! on PATH, plus `~/.next-code/lsp.json` and `.next-code/lsp.json`).

mod client;
mod config;
mod format;
mod manager;
mod uri;

use super::{Tool, ToolContext, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const OPERATIONS: &[&str] = &[
    "goToDefinition",
    "findReferences",
    "hover",
    "documentSymbol",
    "workspaceSymbol",
    "goToImplementation",
    "prepareCallHierarchy",
    "incomingCalls",
    "outgoingCalls",
    "diagnostics",
];

pub struct LspTool;

impl LspTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct LspInput {
    operation: String,
    file_path: String,
    line: u32,
    character: u32,
    #[serde(default)]
    query: Option<String>,
}

#[async_trait]
impl Tool for LspTool {
    fn name(&self) -> &str {
        "lsp"
    }

    fn description(&self) -> &str {
        "Run an LSP operation (go-to-definition, hover, references, diagnostics, symbols). \
         Requires a language server on PATH or configured in .next-code/lsp.json."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["operation", "file_path", "line", "character"],
            "properties": {
                "intent": super::intent_schema_property(),
                "operation": {
                    "type": "string",
                    "enum": OPERATIONS,
                    "description": "LSP operation."
                },
                "file_path": {
                    "type": "string",
                    "description": "File path."
                },
                "line": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "1-based line."
                },
                "character": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "1-based character."
                },
                "query": {
                    "type": "string",
                    "description": "Optional query for workspaceSymbol (empty = all)."
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: LspInput = serde_json::from_value(input)?;
        if !OPERATIONS.contains(&params.operation.as_str()) {
            return Err(anyhow::anyhow!(
                "Unsupported LSP operation: {}",
                params.operation
            ));
        }

        let path = ctx.resolve_path(Path::new(&params.file_path));
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", params.file_path));
        }
        if !path.is_file() {
            return Err(anyhow::anyhow!("Path is not a file: {}", params.file_path));
        }

        let workspace = workspace_root(&ctx, &path);
        let meta = std::fs::metadata(&path)?;
        if meta.len() > manager::MAX_FILE_BYTES {
            return Ok(ToolOutput::new(format!(
                "File too large for LSP analysis ({}MB exceeds 10MB limit)",
                (meta.len() + 999_999) / 1_000_000
            )));
        }

        let content = std::fs::read_to_string(&path)?;
        let mut guard = manager::global_manager(workspace.clone()).await;
        let mgr = &mut guard.as_mut().expect("manager just initialized").1;

        if mgr.server_for_path(&path).is_none() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_else(|| "(none)".into());
            let configured = mgr.configured_server_names();
            let hint = if configured.is_empty() {
                "No language servers configured. Install rust-analyzer / typescript-language-server / pyright-langserver / gopls, or add `.next-code/lsp.json`.".to_string()
            } else {
                format!(
                    "Configured servers: {}. None handle {ext}.",
                    configured.join(", ")
                )
            };
            return Ok(ToolOutput::new(format!(
                "No LSP server available for file type: {ext}\n{hint}"
            )));
        }

        if !mgr.is_file_open(&path) {
            mgr.open_file(&path, &content).await?;
            // Allow publishDiagnostics to arrive before diagnostics reads.
            if params.operation == "diagnostics" {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
        }

        let cwd = Some(workspace.as_path());
        let output = if params.operation == "diagnostics" {
            let result = mgr.diagnostics_for(&path);
            let formatted = format::format_operation("diagnostics", &result, cwd);
            format_tool_output(&params, &formatted)
        } else {
            let (method, req_params) =
                method_and_params(&params, &path, params.query.as_deref())?;
            let mut result = mgr.send_request(&path, method, req_params).await?;

            if params.operation == "incomingCalls" || params.operation == "outgoingCalls" {
                let items = result.as_array().cloned().unwrap_or_default();
                if items.is_empty() {
                    return Ok(ToolOutput::new(
                        "No call hierarchy item found at this position".to_string(),
                    ));
                }
                let call_method = if params.operation == "incomingCalls" {
                    "callHierarchy/incomingCalls"
                } else {
                    "callHierarchy/outgoingCalls"
                };
                result = mgr
                    .send_request(&path, call_method, json!({ "item": items[0] }))
                    .await?;
            }

            let formatted = format::format_operation(&params.operation, &result, cwd);
            format_tool_output(&params, &formatted)
        };

        Ok(ToolOutput::new(output))
    }
}

fn format_tool_output(params: &LspInput, formatted: &format::Formatted) -> String {
    let mut out = formatted.text.clone();
    if formatted.result_count > 0 {
        out.push_str(&format!(
            "\n\n({} result{}, {} file{})",
            formatted.result_count,
            if formatted.result_count == 1 { "" } else { "s" },
            formatted.file_count,
            if formatted.file_count == 1 { "" } else { "s" },
        ));
    }
    let _ = params;
    out
}

fn method_and_params(
    input: &LspInput,
    absolute_path: &Path,
    query: Option<&str>,
) -> Result<(&'static str, Value)> {
    let uri = uri::path_to_uri(absolute_path);
    let position = json!({
        "line": input.line.saturating_sub(1),
        "character": input.character.saturating_sub(1),
    });

    Ok(match input.operation.as_str() {
        "goToDefinition" => (
            "textDocument/definition",
            json!({ "textDocument": { "uri": uri }, "position": position }),
        ),
        "findReferences" => (
            "textDocument/references",
            json!({
                "textDocument": { "uri": uri },
                "position": position,
                "context": { "includeDeclaration": true },
            }),
        ),
        "hover" => (
            "textDocument/hover",
            json!({ "textDocument": { "uri": uri }, "position": position }),
        ),
        "documentSymbol" => (
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": uri } }),
        ),
        "workspaceSymbol" => (
            "workspace/symbol",
            json!({ "query": query.unwrap_or("") }),
        ),
        "goToImplementation" => (
            "textDocument/implementation",
            json!({ "textDocument": { "uri": uri }, "position": position }),
        ),
        "prepareCallHierarchy" | "incomingCalls" | "outgoingCalls" => (
            "textDocument/prepareCallHierarchy",
            json!({ "textDocument": { "uri": uri }, "position": position }),
        ),
        other => return Err(anyhow::anyhow!("Unsupported LSP operation: {other}")),
    })
}

fn workspace_root(ctx: &ToolContext, file: &Path) -> PathBuf {
    if let Some(ref wd) = ctx.working_dir {
        return wd.clone();
    }
    // Walk up for .git / .next-code; else file parent.
    let mut cur = file.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    loop {
        if cur.join(".git").exists() || cur.join(".next-code").exists() {
            return cur;
        }
        if !cur.pop() {
            break;
        }
    }
    file.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolContext;
    use std::collections::HashMap;
    use std::process::Stdio;
    use tokio::process::Command;

    #[test]
    fn tool_is_named_lsp() {
        assert_eq!(LspTool::new().name(), "lsp");
    }

    #[test]
    fn schema_lists_diagnostics() {
        let schema = LspTool::new().parameters_schema();
        let ops = schema["properties"]["operation"]["enum"]
            .as_array()
            .expect("enum");
        assert!(ops.iter().any(|v| v == "diagnostics"));
        assert!(ops.iter().any(|v| v == "goToDefinition"));
        assert!(ops.iter().any(|v| v == "hover"));
        assert!(ops.iter().any(|v| v == "findReferences"));
    }

    #[test]
    fn description_no_longer_stub() {
        let tool = LspTool::new();
        let desc = tool.description().to_lowercase();
        assert!(!desc.contains("stub"));
        assert!(desc.contains("lsp"));
    }

    #[tokio::test]
    async fn end_to_end_with_fake_python_server() {
        // Skip if python unavailable.
        let py_ok = Command::new("python")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false);
        if !py_ok {
            eprintln!("skip: python not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("fake_lsp.py");
        std::fs::write(&script, FAKE_LSP_PY).unwrap();
        let src = dir.path().join("sample.txt");
        std::fs::write(&src, "hello world\n").unwrap();

        let mut configs = HashMap::new();
        configs.insert(
            "fake".into(),
            config::LspServerConfig {
                command: "python".into(),
                args: vec![script.to_string_lossy().into_owned()],
                extension_to_language: HashMap::from([(".txt".into(), "plaintext".into())]),
                env: HashMap::new(),
                initialization_options: None,
                workspace_folder: None,
                startup_timeout_ms: Some(10_000),
                max_restarts: Some(1),
            },
        );

        let mut mgr = manager::LspManager::from_configs(dir.path().to_path_buf(), configs);
        mgr.open_file(&src, "hello world\n").await.expect("open");
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let uri = uri::path_to_uri(&src);
        let def = mgr
            .send_request(
                &src,
                "textDocument/definition",
                json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": 0, "character": 0 },
                }),
            )
            .await
            .expect("definition");
        let formatted = format::format_operation("goToDefinition", &def, Some(dir.path()));
        assert!(
            formatted.text.contains("sample.txt"),
            "got: {}",
            formatted.text
        );

        let hover = mgr
            .send_request(
                &src,
                "textDocument/hover",
                json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": 0, "character": 0 },
                }),
            )
            .await
            .expect("hover");
        let hover_fmt = format::format_operation("hover", &hover, None);
        assert!(hover_fmt.text.contains("fake hover"), "{}", hover_fmt.text);

        let refs = mgr
            .send_request(
                &src,
                "textDocument/references",
                json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": 0, "character": 0 },
                    "context": { "includeDeclaration": true },
                }),
            )
            .await
            .expect("refs");
        let refs_fmt = format::format_operation("findReferences", &refs, Some(dir.path()));
        assert!(refs_fmt.result_count >= 1, "{}", refs_fmt.text);

        let diags = mgr.diagnostics_for(&src);
        let diag_fmt = format::format_operation("diagnostics", &diags, Some(dir.path()));
        assert!(
            diag_fmt.text.contains("fake diagnostic") || diag_fmt.text.contains("No diagnostics"),
            "{}",
            diag_fmt.text
        );

        mgr.shutdown_all().await.ok();
    }

    #[tokio::test]
    async fn tool_execute_missing_file_errors() {
        let tool = LspTool::new();
        let err = tool
            .execute(
                json!({
                    "operation": "hover",
                    "file_path": "definitely-missing-xyz.rs",
                    "line": 1,
                    "character": 1,
                }),
                ToolContext::default(),
            )
            .await
            .expect_err("missing file");
        assert!(err.to_string().contains("not found"));
    }

    const FAKE_LSP_PY: &str = r#"
import json, sys

def read_message():
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        line = line.decode("utf-8")
        if line in ("\r\n", "\n"):
            break
        if ":" in line:
            k, v = line.split(":", 1)
            headers[k.strip().lower()] = v.strip()
    n = int(headers.get("content-length", "0"))
    body = sys.stdin.buffer.read(n)
    return json.loads(body.decode("utf-8"))

def send(msg):
    data = json.dumps(msg).encode("utf-8")
    sys.stdout.buffer.write(f"Content-Length: {len(data)}\r\n\r\n".encode("ascii"))
    sys.stdout.buffer.write(data)
    sys.stdout.buffer.flush()

open_uri = None
while True:
    msg = read_message()
    if msg is None:
        break
    method = msg.get("method")
    id = msg.get("id")
    params = msg.get("params") or {}
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": id, "result": {
            "capabilities": {
                "textDocumentSync": 1,
                "definitionProvider": True,
                "referencesProvider": True,
                "hoverProvider": True,
                "documentSymbolProvider": True,
            }
        }})
    elif method == "initialized":
        pass
    elif method == "textDocument/didOpen":
        open_uri = params["textDocument"]["uri"]
        send({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {
            "uri": open_uri,
            "diagnostics": [{
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
                "severity": 2,
                "message": "fake diagnostic"
            }]
        }})
    elif method == "textDocument/definition":
        uri = params["textDocument"]["uri"]
        send({"jsonrpc": "2.0", "id": id, "result": [{
            "uri": uri,
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 5}}
        }]})
    elif method == "textDocument/hover":
        send({"jsonrpc": "2.0", "id": id, "result": {
            "contents": {"kind": "markdown", "value": "fake hover"}
        }})
    elif method == "textDocument/references":
        uri = params["textDocument"]["uri"]
        send({"jsonrpc": "2.0", "id": id, "result": [
            {"uri": uri, "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 5}}},
            {"uri": uri, "range": {"start": {"line": 0, "character": 6}, "end": {"line": 0, "character": 11}}},
        ]})
    elif method == "shutdown":
        send({"jsonrpc": "2.0", "id": id, "result": None})
    elif method == "exit":
        break
    elif id is not None:
        send({"jsonrpc": "2.0", "id": id, "result": None})
"#;
}
