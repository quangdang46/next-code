//! Streamable-HTTP MCP transport (JSON-RPC over POST).
//!
//! Ported in spirit from grok `xai-grok-mcp` (`StreamableHttpClientTransport`):
//! POST JSON-RPC with `Accept: application/json, text/event-stream`, then parse
//! either a bare JSON body or SSE `data:` frames. Optional `Mcp-Session-Id`
//! is echoed on subsequent requests when the server sets it.
//!
//! Auth: static headers from mcp.json (`headers`) only for now — no OAuth
//! browser flow (that remains in grok's `xai-grok-mcp::oauth`).

use super::protocol::JsonRpcResponse;
use anyhow::{Context, Result, anyhow};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared HTTP backend for one MCP server URL.
#[derive(Clone)]
pub struct HttpBackend {
    client: reqwest::Client,
    url: String,
    session_id: Arc<Mutex<Option<String>>>,
}

impl HttpBackend {
    pub fn new(url: String, headers: &HashMap<String, String>) -> Result<Self> {
        let mut map = HeaderMap::new();
        for (key, value) in headers {
            let name = HeaderName::from_bytes(key.as_bytes())
                .with_context(|| format!("invalid MCP HTTP header name: {key}"))?;
            let val = HeaderValue::from_str(value)
                .with_context(|| format!("invalid MCP HTTP header value for {key}"))?;
            map.insert(name, val);
        }
        let client = reqwest::Client::builder()
            .default_headers(map)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to build MCP HTTP client")?;
        Ok(Self {
            client,
            url,
            session_id: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn request(&self, body: &Value) -> Result<JsonRpcResponse> {
        let mut req = self
            .client
            .post(&self.url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::ACCEPT,
                "application/json, text/event-stream",
            )
            .header("MCP-Protocol-Version", "2024-11-05");

        if let Some(sid) = self.session_id.lock().await.as_ref() {
            req = req.header("Mcp-Session-Id", sid.as_str());
        }

        let resp = req.json(body).send().await.with_context(|| {
            format!("MCP HTTP POST failed for {}", self.url)
        })?;

        if let Some(sid) = resp
            .headers()
            .get("mcp-session-id")
            .or_else(|| resp.headers().get("Mcp-Session-Id"))
            .and_then(|v| v.to_str().ok())
        {
            *self.session_id.lock().await = Some(sid.to_string());
        }

        let status = resp.status();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();
        let text = resp
            .text()
            .await
            .context("Failed to read MCP HTTP response body")?;

        if !status.is_success() {
            anyhow::bail!(
                "MCP HTTP {} returned {}: {}",
                self.url,
                status,
                truncate(&text, 240)
            );
        }

        let json_text = if content_type.contains("text/event-stream") || text.contains("data:") {
            extract_sse_json_rpc(&text)?
        } else {
            text
        };

        serde_json::from_str(&json_text).with_context(|| {
            format!(
                "Failed to parse MCP HTTP JSON-RPC from {}: {}",
                self.url,
                truncate(&json_text, 240)
            )
        })
    }

    /// Fire-and-forget notification (no JSON-RPC id).
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        let mut body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
        });
        if let Some(p) = params {
            body.as_object_mut()
                .unwrap()
                .insert("params".into(), p);
        }

        let mut req = self
            .client
            .post(&self.url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::ACCEPT,
                "application/json, text/event-stream",
            )
            .header("MCP-Protocol-Version", "2024-11-05");

        if let Some(sid) = self.session_id.lock().await.as_ref() {
            req = req.header("Mcp-Session-Id", sid.as_str());
        }

        let resp = req.json(&body).send().await.with_context(|| {
            format!("MCP HTTP notify '{method}' failed for {}", self.url)
        })?;
        // Many servers return 202/200 with empty or SSE ack; treat 2xx as ok.
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "MCP HTTP notify '{method}' {} → {}: {}",
                self.url,
                status,
                truncate(&text, 200)
            );
        }
        Ok(())
    }
}

/// Pull the first JSON-RPC object from an SSE body (`data: {...}` lines).
pub fn extract_sse_json_rpc(body: &str) -> Result<String> {
    let mut data_lines: Vec<&str> = Vec::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start());
        } else if line.is_empty() && !data_lines.is_empty() {
            break;
        }
    }
    if data_lines.is_empty() {
        // Some servers stream a single data line without a trailing blank.
        for line in body.lines() {
            if let Some(rest) = line.strip_prefix("data:") {
                return Ok(rest.trim_start().to_string());
            }
        }
        return Err(anyhow!(
            "no SSE data frame in MCP HTTP response: {}",
            truncate(body, 200)
        ));
    }
    Ok(data_lines.join("\n"))
}

fn truncate(s: &str, max: usize) -> String {
    let t = s.trim();
    if t.len() <= max {
        t.to_string()
    } else {
        format!("{}…", &t[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn extract_sse_json_rpc_from_deepwiki_shape() {
        let body = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}\n\n";
        let json = extract_sse_json_rpc(body).expect("sse");
        let v: Value = serde_json::from_str(&json).expect("json");
        assert_eq!(v["id"], 1);
        assert_eq!(v["result"]["ok"], true);
    }

    #[test]
    fn extract_sse_without_trailing_blank() {
        let body = "data: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{}}\n";
        let json = extract_sse_json_rpc(body).expect("sse");
        assert!(json.contains("\"id\":2"));
    }

    #[tokio::test]
    async fn http_backend_parses_sse_and_echoes_session_header() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let url = format!("http://{addr}/mcp");

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept");
            let mut buf = Vec::new();
            let mut tmp = [0u8; 1024];
            loop {
                let n = socket.read(&mut tmp).await.expect("read");
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&tmp[..n]);
                if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            let req = String::from_utf8_lossy(&buf).to_ascii_lowercase();
            assert!(req.contains("post "));
            assert!(req.contains("application/json, text/event-stream"));
            assert!(req.contains("authorization: bearer test-token"));

            let body = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"tools\":[]}}\n\n";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nMcp-Session-Id: sess-abc\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(resp.as_bytes()).await.expect("write");
        });

        let mut headers = HashMap::new();
        headers.insert("Authorization".into(), "Bearer test-token".into());
        let backend = HttpBackend::new(url, &headers).expect("client");
        let response = backend
            .request(&json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}))
            .await
            .expect("request");
        assert!(response.result.is_some());
        assert_eq!(
            backend.session_id.lock().await.as_deref(),
            Some("sess-abc")
        );
        server.await.expect("server");
    }
}
