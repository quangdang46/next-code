# NEXTCODE — EXISTING-FEATURE AUDIT & REPAIR PLAN

> Deep real-world audit of current nextcode functionality: trace complete user flows, find root causes, spec fixes with regression tests. Source of truth for the repair work. Companion to `NEXTCODE-UI-UX-DX-FULL-PARITY-RESEARCH.md`.

**Branch basis:** `review` (HEAD 351ee19f4, version 0.32.0).
**Method:** 7 trace/source agents produced 34 candidate findings; **20 survive as confirmed findings (R1–R20)** — 3 P0, 8 P1, 9 P2. Porting errors: **0** found across the OpenCode / Grok-Build-ACP / JCode / oh-my-openagent ports (all byte-faithful).
**Status:** Research complete. Fixes pending implementation (batches below).

---

## 1. What Was Audited (flows traced end-to-end)

| Area | Flow traced |
|---|---|
| Auth | login → token persist → restart → authenticated state → provider connection |
| Provider | startup resolution → activation → selection → request-time use |
| Model | routes → set_model → request model mapping → switch A→B→C |
| Message | communicate → provider.complete → request (headers/endpoint/body) → StreamEvent → render |
| Streaming | complete → event stream → turn loop → render |
| Tool calls | ToolDefinition → tool_use → execution → ToolResult → next request |
| Cancel/interrupt | user cancel → provider cancel → state reset → retry |
| Permission | request_permission → allow/session/always/deny → persist → next request |
| Errors/retry | network error → retry → recovery |
| Sessions | create → persist → restart → resume → continue |
| CLI | login/run/resume/model/config → output → exit code |
| Config | files → read → apply → precedence → persistence |
| Keybindings | binding → action dispatch |
| MCP | server config → connect → tool exposure |

## 2. Verified-Correct (PASS) — do not regress

- Auth persistence (Claude/OpenAI OAuth + API keys, restart) — `crates/next-code-base/src/auth/claude.rs:403,648-675,847-857`; `src/cli/provider_init.rs:1351-1354`.
- Provider resolution / forced-provider / OpenRouter multiplexing — `crates/next-code-provider-core/src/lib.rs`; `crates/next-code-base/src/provider/mod.rs:601-614`.
- Anthropic request construction (OAuth vs API-key, streaming, refresh) — `crates/next-code-provider-anthropic-runtime/src/lib.rs:1295-1390,1483-1512,1703-1746`.
- Legacy TUI model switching (profile-prefixed RouteSelection) — `crates/next-code-tui/src/tui/app/inline_interactive/helpers.rs:140-146`.
- OpenCode/Zen/Go endpoint + model port (byte-faithful) — `crates/next-code-provider-metadata/src/catalog.rs:6-28`; `provider-openrouter-runtime/src/lib.rs:1418-1450`.
- Grok Build ACP runtime (handshake/session/prompt/cancel/modelState) — `crates/next-code-provider-grok-runtime/src/lib.rs` (809 lines, byte-identical to jcode `659b8cc15`).
- JCode todo status normalize/reject + Codex quota dedup — `crates/next-code-base/src/todo.rs:7-41`; `crates/next-code-app-core/src/tool/todo.rs:64,302,373`; `crates/next-code-base/src/usage/openai_helpers.rs`.
- Streaming event forwarding (Text/Thinking/Tool deltas, keepalive) — `agent/turn_streaming_mpsc.rs:428-540,704-720`.
- Tool-result feed-back via sdk_tool_results — `agent/turn_streaming_mpsc.rs:553-574,1198-1217,1390-1395`.
- Provider cancel (GrokEventStream::drop → ACP session/cancel) — `crates/next-code-provider-grok-runtime/src/lib.rs:298-307,499-507`.
- Mid-stream retry/rollback (RetryRollback) — `crates/next-code-provider-anthropic-runtime/src/lib.rs:1580-1594`.
- JSON/headless CLI modes — `src/cli/commands.rs:2279-2301`; `src/cli/dispatch.rs:600-602`.
- Session resume journal+snapshot replay — `crates/next-code-base/src/session/persistence.rs:76-129`.
- OpenAI/Anthropic/OpenRouter endpoint + auth + request shape — `crates/next-code-provider-openai-runtime/src/lib.rs:41`; `crates/next-code-base/src/provider/openai.rs:16,70-132`; `crates/next-code-provider-anthropic-runtime/src/lib.rs:58-61,649-657,992-1012`; `crates/next-code-provider-openrouter-runtime/src/lib.rs:60`.

## 3. Confirmed Findings (R1–R20)

### P0 — model-visible history corruption / hard failure

**R1. OpenAI-compatible profile models misrouted to real OpenRouter via bare `set_model`** (PROVIDER_BUG/STATE_BUG, CONFIRMED)
- Mechanism: `MultiProvider::set_model` (`crates/next-code-base/src/provider/mod.rs:1825`) with a bare model → `set_model_on_provider(OpenRouter, model)` (`mod.rs:1928-1935`) → `clear_active_openai_compatible_profile()` + `needs_rebind` → re-instantiates OpenRouter API-key runtime (`mod.rs:1071-1113`) → bails "OPENROUTER_API_KEY not found" or silently reroutes to openrouter.ai. The Face bridge never sends the profile-prefixed spec (`src/cli/pager_agent.rs:3015-3032`).
- Fix: in `set_model`, when `self.active_compatible_profile_id().is_some()` and model is not provider-prefixed/known-built-in, route via `set_model_on_openai_compatible_profile(profile, model)` (`mod.rs:1138-1184`).
- Real-world: `next-code login --provider opencode-go` → Face → `/model` → pick model → next prompt must reach `opencode.ai/zen/go/v1` (not error, not openrouter.ai).
- Crate: `next-code-base`.

**R2. Cancel/abort poisons history — `repair_missing_tool_outputs` injects fabricated failures** (STATE_BUG, CONFIRMED)
- Mechanism: on cancel mid-tool, assistant ToolUse committed (`turn_streaming_mpsc.rs:1003-1017`) but ToolResult never lands (tool task aborted `:1427`). Next turn `repair_missing_tool_outputs` (`agent.rs:1022`) injects `[Tool output missing — tool execution did not complete]` with `is_error: Some(true)` (`agent.rs:1078-1093`).
- Fix: extend the existing `"[Skipped: user interrupted]"` pattern (urgent-interrupt `turn_streaming_mpsc.rs:1110-1121`, reload `1458-1469`) to the generic abort path before `return Ok(())` (`:1470`).
- Real-world: start long tool, Esc mid-run, next message → model must not claim a tool failed.
- Crate: `next-code-app-core`.

### P1 — provider capability / auth correctness

**R3. Dotted Copilot model ids switch provider via global heuristic** (PROVIDER_BUG, CONFIRMED)
- `set_model("claude-sonnet-4.6")` with Copilot active: `normalize_copilot_model_name` dots→hyphens → `provider_for_model` → `Some("claude")` → switches off Copilot (`mod.rs:1910-1935`).
- Fix: in `set_model`, when active provider is Copilot, route bare models to `set_model_on_provider(Copilot, …)` before the heuristic.
- Crate: `next-code-base`.

**R4. Grok Build / Claude CLI (`handles_tools_internally`) drop non-native tools silently** (PROVIDER_BUG, CONFIRMED)

**Root cause chain**
`tool_calls.retain(|tc| NEXT_CODE_NATIVE_TOOLS.contains(..))` (`turn_streaming_mpsc.rs:1089-1103`); `NEXT_CODE_NATIVE_TOOLS = ["selfdev","communicate"]` (`agent.rs:63`) while claude-cli advertises more (`claude-cli-runtime/lib.rs:56,78-79`). Grok ACP `ToolCall`/`ToolCallUpdate` → `StreamEvent::StatusDetail` only (`grok-runtime/lib.rs:665-673`), so no ToolResult is ever persisted → orphan ToolUse → R2 fabricates a failure.

**Exact ACP completion signal (Grok)**
- ACP emits `SessionUpdate::ToolCall(call)` on start and `SessionUpdate::ToolCallUpdate(update)` on progress. The *completion* is signaled by the ACP server either via a subsequent `ToolCallUpdate` with terminal status, or the end of the turn (`session/prompt` response). **Verification needed:** confirm which exact update the Grok CLI sends at completion (map-agent note: `ToolCallUpdate.fields.title` carries the status text; the terminal state may be indicated by a status field). See `agent-client-protocol` types in `grok-runtime/Cargo.toml` (features `unstable_session_*`).

**ToolResult mapping (target)**
- From an ACP `ToolCall`/`ToolCallUpdate` at completion, build `StreamEvent::ToolResult { tool_use_id, content, is_error }`:
  - `tool_use_id` ← the ACP call id (the id that will appear in `tc.id` of the next loop iteration).
  - `content` ← the tool's final output text (from the completion update's result/fields).
  - `is_error` ← `Some(true)` if the update reports a failure/error status, else `None`.
- The runtime's `GrokAcpClient::session_notification` (`grok-runtime/lib.rs:654-678`) is the producer; emit the `ToolResult` there (and a matching `ToolUseStart`/`ToolUseEnd` pair is optional — the loop keys results by `tc.id`, not by a prior start event).

**Feed-back into the turn loop**
- `sdk_tool_results` (`turn_streaming_mpsc.rs:1198-1217`) is keyed by `tool_use_id`; a `StreamEvent::ToolResult` emitted by the provider flows into `sdk_tool_results` and is added as a user `ToolResult` block, then `continue`s (no local execution). This is the intended path for `handles_tools_internally` providers.

**Retain-filter fix (widen to provider-advertised natives)**
- Replace the hardcoded `NEXT_CODE_NATIVE_TOOLS` retain (`turn_streaming_mpsc.rs:1090`) with the provider's own advertised native-tool set. Add a `Provider` method (or a constant) e.g. `fn native_tool_names(&self) -> &[&str]`; default `NEXT_CODE_NATIVE_TOOLS`; Grok/Claude-CLI override with their full advertised sets. Do NOT drop a tool the provider advertises as native.

**Ordering / parallel / cancel**
- **Ordering:** ToolResult blocks must be inserted in the same order as their ToolUse ids appear in the assistant message, so the transcript stays consistent (the loop already adds results in `tc` order; the SDK path adds at the matching `tc.id`).
- **Parallel tools:** if the Grok CLI emits multiple `ToolCall`s, each completion produces its own `ToolResult` keyed by its own id; the loop handles them in order.
- **Cancel between ToolCall and ToolResult:** a cancel/abort must still produce a result for the in-flight id (the R2 fix's interruption notice) so no orphan remains — R4 and R2 must land together.

**Expected persisted transcript (after fix)**
```
assistant: ToolUse{id: call_1, name: "read", ...}
user:      ToolResult{tool_use_id: call_1, content: "<file content>", is_error: None}
```
No orphan ToolUse; no fabricated `[Tool output missing]`.

**Test fixture (exact)**
- In `crates/next-code-provider-grok-runtime/tests/fake_acp.rs`, extend `fake_acp.rs` so `session/prompt` (or the matching update) emits a `ToolCall` + a completion `ToolCallUpdate` for a `read` tool, then assert:
  - the provider emits `StreamEvent::ToolResult { tool_use_id: <call_1>, content: "<file>", is_error: None }`;
  - a subsequent `complete()` turn's transcript contains the ToolResult and no orphan ToolUse.
- In `next-code-app-core`, unit-test the retain filter: a provider advertising `["selfdev","communicate","memory","session_search","bg"]` does NOT drop `memory`/`session_search`/`bg`.

**Files**
- `crates/next-code-provider-grok-runtime/src/lib.rs` (emit ToolResult; advertise native set),
- `crates/next-code-provider-claude-cli-runtime/src/lib.rs` (advertise full native set),
- `crates/next-code-app-core/src/agent/turn_streaming_mpsc.rs` (retain via provider set),
- `crates/next-code-app-core/src/agent.rs` (`NEXT_CODE_NATIVE_TOOLS` default).
- **Crates:** `grok-runtime`, `claude-cli-runtime`, `app-core`.

**R5. `login --provider anthropic-api`/`openai-api` not pinned — OAuth credential wins on standalone run** (AUTH_BUG, CONFIRMED)
- `login_anthropic_api_key_flow` doesn't pin the API-key route for standalone runs; OAuth wins. `src/cli/login.rs:492-516,340,368`.
- Fix: pin the credential route (API key) at login/bootstrap.
- Crates: `src/cli/login.rs`, `provider_init.rs`.

**R6. Face `/connect` default_provider not persisted** (STATE_BUG, CONFIRMED)
- `src/cli/face_auth.rs:287-341` — connect applies default provider to the daemon but doesn't persist it; restart loses it.
- Fix: persist the chosen default_provider at connect.

**R7. Session close/crash not persisted on disconnect** (PERSISTENCE_BUG, CONFIRMED)
- `client_disconnect_cleanup.rs:120-128` marks closed/crashed without saving; session stays `Active` on disk.
- Fix: persist state on disconnect cleanup.

**R8. Session creation not persisted before first turn** (PERSISTENCE_BUG, CONFIRMED)
- Server create path (`client_session.rs`) doesn't save; kill -9 idle → session not restorable.
- Fix: best-effort `agent.session.save()` after `mark_active()`.

### P2 — correctness / hardening

**R9. Gemini `thought_signature` dropped at tool-use persistence** — `turn_streaming_mpsc.rs:999-1000` builds `ToolUse` with `thought_signature: None`. Fix: `thought_signature: tc.thought_signature.clone()`.
**R10. Config `default_model` overrides CLI `--model`** — `apply_config_default_model` (`agent.rs:293-325,458`) clobbers the flag. Fix: gate on "no explicit model resolved".
**R11. Always-allow lost on session restore** — `init_session_allow_list` only at `create_agent` (`agent.rs:479`); `restore_session_with_working_dir` (`turn_execution.rs:1058`) never re-seeds. Fix: call `init_session_allow_list` in restore.
**R12. OpenRouter mid-stream SSE `error` bypasses retry classifier** — `openrouter_sse_stream.rs:247-276` forwards `StreamEvent::Error` with no `is_retryable_error` check. Fix: mirror OpenAI/Anthropic.
**R13. `await_permission_response` can hang forever headless** — `dcg_bridge.rs:334-357` blocks with no consumer/timeout. Fix: add timeout or auto-deny.
**R14. OpenAI 429 retry depends on literal "rate limit" phrase** — `openai_stream_runtime.rs:223-227,1542-1544`. Fix: short-circuit on status code.
**R15. MCP `import_from_external()` runs as side effect of every config read** — `mcp/protocol.rs:522` writes on load when `mcp.json` absent. Fix: move to explicit migration.
**R16. TUI toggles not persisted** (diff_mode/info_widget/centered/scroll_bookmark) — `next-code-tui/src/tui/app/input.rs:1777,1918,1985`. Fix: add config setters.
**R17. Keybinding default/env-override coverage gaps (8 of 26 fields)** — `keybindings.rs:195-336`; `env_overrides.rs:12-86`. Fix: complete coverage.
**R18. Anthropic `redacted_thinking_delta`/unknown deltas silently dropped** — `lib.rs:2204-2224,2356-2370`. Fix: catch-all + log; emit empty ThinkingDelta.
**R19. Embedded 401/403 text in error body classified retryable** — `failover.rs:435-499`. Fix: gate embedded-code matching on real HTTP status.
**R20. OpenAI-compatible `build_tools` sanitizes function names — HARDENING / DOCUMENTED INVARIANT (no-op today)** — `request.rs:63-76,214,284` renames any char outside `[a-zA-Z0-9_-]` to `_` (DeepSeek strict rule) and applies `sanitize_tool_id` to `call_id`. Today every nextcode tool name and `mcp__server__tool` id already complies, so it's a **no-op**. **Not an implementation task — do not spend a Rust build on it.** 
- **Action (docs only, no code change):** add a comment/guard binding the sanitize to `resolve_tool_name` exact-match (`crates/next-code-tool-types/src/lib.rs:81`), including a `functions.`-style strip for the sanitized form. If a user MCP tool ever uses `.`/`/`/`:` in its name, the wire name would diverge from the registry key and the model's follow-up call would fail `resolve_tool_name` — fix then would touch both `next-code-provider-openai` and `next-code-provider-openrouter-runtime` in lockstep. Documented invariant; no build/test needed.

## 4. Implementation Batches (by crate → one cargo build each)

| Batch | Crates | Findings | Build/test |
|---|---|---|---|
| B1 | next-code-base | R1, R3, R15 | `cargo test -p next-code-base` |
| B2 | next-code-app-core | R2, R9, R11, R13 | `cargo test -p next-code-app-core` |
| B3 | provider-openrouter-runtime | R12 | `cargo test -p next-code-provider-openrouter-runtime` |
| B4 | provider-openai-runtime | R14 | `cargo test -p next-code-provider-openai-runtime` |
| B5 | provider-anthropic-runtime | R18 | `cargo test -p next-code-provider-anthropic-runtime` |
| B6 | grok-runtime + claude-cli-runtime | R4 | `cargo build -p …` + app-core tests |
| B7 | src/cli (login.rs, face_auth.rs) | R5, R6 | `cargo test --bin next-code` |
| B8 | src/cli (commands, provider_init) + app-core | R10 | `cargo test --bin next-code` + app-core |
| B9 | app-core server | R7, R8 | `cargo test -p next-code-app-core` |
| B10 | next-code-tui + config-types + base | R16, R17 | `cargo test -p next-code-tui -p next-code-config-types -p next-code-base` |
| B11 | provider-core | R19 | `cargo test -p next-code-provider-core` |
| B12 | provider-openai + openrouter-runtime | R20 (HARDENING/INVARIANT — no-op) | docs/comment only, **no build** |

**Recommended order:** B1 → B2 → B7 → B9 → B3 → B5 → B4 → B6 → B8 → B10 → B11 → B12. (B12 is a documented invariant — fold into any adjacent build, do not build standalone.)

## 5. Real-World Verification Scenarios (acceptance)

1. **R1**: login opencode-go → Face `/model` → pick model → prompt reaches zen/go endpoint, no OPENROUTER_API_KEY needed.
2. **R2**: long tool, Esc mid-run, next message → no phantom tool failure.
3. **R3**: Copilot active → pick claude-sonnet-4.6 → provider stays Copilot.
4. **R4**: Grok Build calls read/grep → ToolResult persisted; no fabricated error.
5. **R5**: Claude subscription + API key → `login --provider anthropic-api && run` bills key (x-api-key in logs).
6. **R6**: Face-connect two compat profiles, restart → provider auto-selected.
7. **R7**: connect, close window, restart → session Closed in resume picker.
8. **R8**: launch, idle, kill -9 server, restart → `/resume` finds session.
9. **R9**: Gemini run with tools → resume → thought signatures intact.
10. **R10**: `next-code run -m gpt-5.5 "hi"` with default_model set → model gpt-5.5.
11. **R11**: always-allow bash, quit, resume, run bash → no prompt.
12. **R12**: openrouter mid-stream overload → turn retries.
13. **R16**: `/diff pinned`, quit, relaunch → still Pinned.

## 6. Feature Status Matrix

| Feature | Status | Evidence |
|---|---|---|
| Auth persistence | PASS | auth/claude.rs:403,648-675,847-857 |
| Provider resolution / forced / OpenRouter multiplex | PASS | provider-core lib.rs; provider/mod.rs:601-614 |
| Anthropic request construction | PASS | anthropic-runtime lib.rs:1295-1390,1483-1512,1703-1746 |
| Legacy TUI model switching | PASS | inline_interactive/helpers.rs:140-146 |
| Bare-model switching on OpenAI-compatible profiles | **BROKEN** | R1: provider/mod.rs:1071-1113,1825,1928-1935; pager_agent.rs:3015-3032 |
| Dotted Copilot model switch | **BROKEN** | R3: provider-core models.rs:179,185,196,440; provider/mod.rs:1910-1935 |
| Cancel/abort history integrity | **BROKEN** | R2: agent.rs:1022-1103; turn_streaming_mpsc.rs:92-98,1003-1017,1427,1470 |
| Grok Build / Claude CLI tool-result feedback | **BROKEN** | R4: turn_streaming_mpsc.rs:1089-1103,1198-1217; grok-runtime lib.rs:258-260,665-673 |
| anthropic-api / openai-api login pinning | **BROKEN** | R5: login.rs:492-516,340,368; anthropic-runtime lib.rs:762-766 |
| OpenCode/Zen/Go endpoint + model port | PASS | catalog.rs:6-28; openrouter-runtime lib.rs:1418-1450 |
| Grok Build ACP runtime | PASS | grok-runtime lib.rs (byte-identical) |
| JCode todo + quota dedup ports | PASS | todo.rs:7-41; usage/openai_helpers.rs (byte-identical) |
| Streaming event forwarding | PASS | turn_streaming_mpsc.rs:428-540,704-720 |
| Tool-result feed-back | PASS | turn_streaming_mpsc.rs:553-574,1198-1217 |
| Provider cancel | PASS | grok-runtime lib.rs:298-307,499-507 |
| Mid-stream retry/rollback | PASS | anthropic-runtime lib.rs:1580-1594 |
| JSON/headless CLI | PASS | commands.rs:2279-2301; dispatch.rs:600-602 |
| Session resume replay | PASS | session/persistence.rs:76-129 |
| OpenAI/Anthropic/OpenRouter contracts | PASS | (multiple anchors above) |
| OpenRouter mid-stream SSE error retry | **PARTIAL** | R12: openrouter_sse_stream.rs:247-276 |
| Face /connect default_provider persistence | **BROKEN** | R6: face_auth.rs:287-341 |
| Session close/crash persistence | **BROKEN** | R7: client_disconnect_cleanup.rs:120-128 |
| Session creation persistence | **BROKEN** | R8: session.rs:751-803 |
| Gemini thought_signature persistence | **BROKEN** | R9: turn_streaming_mpsc.rs:543-552,999-1000 |
| CLI --model vs config default_model | **BROKEN** | R10: commands.rs:2271-2276; agent.rs:293-325,458 |
| Always-allow across restore | **BROKEN** | R11: agent.rs:479; dcg_bridge.rs:1673; turn_execution.rs:1058-1090 |
| Headless permission hang | **PARTIAL** | R13: dcg_bridge.rs:334-357 |
| OpenAI 429 classifier | **PARTIAL** | R14: openai_stream_runtime.rs:223-227,1542-1544 |
| MCP import side-effect on read | **BROKEN** | R15: mcp/protocol.rs:286-350,522-533 |
| TUI layout toggle persistence | **BROKEN** | R16: input.rs:1777,1918,1985 |
| Keybinding default/env coverage | **PARTIAL** | R17: keybindings.rs:195-336; env_overrides.rs:12-86 |
| Anthropic redacted/delta lifecycle | **PARTIAL** | R18: anthropic-runtime lib.rs:2204-2224,2356-2370 |
| Embedded 401/403 retry classification | PARTIAL | R19: failover.rs:435-499 |
| build_tools sanitize vs registry | HARDENING / DOCUMENTED INVARIANT (no-op) | R20: request.rs:63-76,214,284 |

## 7. Remaining Issues / Next Steps

- Implement batches B1+B2 first (R1, R2, R3, R9, R11, R13) — they corrupt the model-visible transcript and block provider switching.
- Then B7/B9 (login + session persistence) before any release cut.
- Then the remaining P2 batches.
- Fold the fixed-defect statuses back into the parity report §10 specs where they overlap (e.g. permission flows).
