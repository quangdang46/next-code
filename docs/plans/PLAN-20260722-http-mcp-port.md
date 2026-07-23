# Plan Report

## Summary (read this first)
- **You asked:** Port HTTP MCP from grok so Face connects exa/deepwiki/etc. and tools work.
- **What is going on:** Lean streamable-HTTP JSON-RPC is in `next-code-base` (POST + JSON/SSE). Stdio kept. `load_for_dir` keeps HTTP. Face `x.ai/mcp/list` probes HTTP for ready/tools.
- **We recommend:** Quit Face + `next-code serve`, reopen `/mcp` on installed binary (`20260722f-http-mcp`).
- **Risk:** Medium (protocol edge cases)
- **Status:** Done — installed; smoke Face UI locally

## Evidence
1. **Grok:** `grok-build/.../xai-grok-mcp/src/servers.rs` `build_http_transport` + `HttpConfig.headers`
2. **Ours:** `mcp/http.rs`, dual transport in `client.rs`, Face async list

## Gaps vs full grok
- No OAuth / AuthClient / credential store
- No GET SSE stream / delete session / liveness poller
- Face list probes HTTP each open (not shared pool)

## Verify
- `cargo test -p next-code-base --lib mcp::` (HTTP mock + config)
- `cli::face_auth::tests::mcp_list_includes_http_servers_from_user_mcp_json` ok (~2s live)
- Install: SHA256 `02E56B16…`, wire `20260722f-http-mcp`
