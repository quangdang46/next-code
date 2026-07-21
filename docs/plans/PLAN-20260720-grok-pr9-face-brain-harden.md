# Plan Report — PR9 Face brain harden (ACP bridge)

**Date:** 2026-07-20 (rewritten after ultracode research)
**Branch:** `pr-9-face-brain-harden`
**Base:** `dev`
**Skill:** `.agents/skills/grok-migration-workflow/SKILL.md`

---

## Summary (read this first)

- **The problem:** `NextCodeFaceAgent` (src/cli/pager_agent.rs) is a thin text-only bridge — its `prompt()` loop drops **19 of 22** `ServerEvent` variants. Tool calls, compaction, token usage, session rename, MCP status, image generation, and all ext notifications are silently swallowed. Face users see streaming text but no tool visualization, no permission flow, no title sync.
- **Research done:** DeepWiki (stock grok-build Face ACP) + vendored `xai-grok-pager` ACP handler + wire seam (`pager_agent.rs` + `pager_launch.rs` + `acp.rs` `EventMapper`) + daemon `ServerEvent` enum.
- **We recommend:** **Wire-only** at the ACP↔daemon seam. Copy the `EventMapper::map_event()` pattern from `src/cli/acp.rs` into the Face bridge. **No Face rewrites** — the vendored Face TUI ACP handler (`acp_handler/mod.rs`) is already fully capable; we just need to feed it the right events.
- **Risk:** High (touches core event loop; tool visualization is blocking for production use)
- **Status:** 🔬 Evidence required (fill "Unverified" cells below before BUILD)

---

## Workflow map (mandatory — per grok-migration-workflow)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | `EventMapper` structural patterns from `src/cli/acp.rs:760-875` (tool lifecycle mapping, compaction, session rename) | Invent parallel TUI rendering in `next-code-tui` |
| **Wire** | `ServerEvent` → typed ACP `SessionUpdate` calls in `prompt()` loop; `set_session_mode`/`set_session_model` trait methods; `cancel` path hardening | Add `GrokHost` trait |
| **Delete** | None in this PR | Do not delete Face permission modal or old TUI escape hatch |

---

## Evidence (fill before BUILD)

Every claim must be `verified` (path:line) or `unverified — needs X`.

### Verified (from research)

| # | Claim | Citation | Status |
|---|-------|----------|--------|
| 1 | `NextCodeFaceAgent::prompt()` only handles `TextDelta`, `TextReplace`, `Done`, `Error` — all other events dropped | `src/cli/pager_agent.rs:302-313` | ✅ verified |
| 2 | `DaemonSession` is duplicated in both `acp.rs` and `pager_agent.rs` | `src/cli/acp.rs` + `src/cli/pager_agent.rs:20-62` | ✅ verified |
| 3 | `EventMapper::map_event()` in `acp.rs` maps `ToolStart` → `{"sessionUpdate": "tool_call", ...}`, `ToolInput/Exec/Done` → `"tool_call_update"`, etc. | `src/cli/acp.rs:760-875` | ✅ verified |
| 4 | The Face `AcpGatewaySender<acp::AgentSide>` calls `self.gateway.session_notification()` to send typed notifications to Face TUI | `crates/xai-acp-lib/src/gateway.rs:447` | ✅ verified |
| 5 | `set_session_mode` and `set_session_model` trait methods are NOT implemented | `src/cli/pager_agent.rs:210-327` | ✅ verified |
| 6 | Face `acp_handler/mod.rs` can handle `SessionNotification`, `RequestPermission`, `ExtMethod`, `ExtNotification` | `crates/xai-grok-pager/src/app/acp_handler/mod.rs` | ✅ verified (vendored) |
| 7 | Daemon `Request` enum has `SetPermissionMode` and `SetModel` variants | `crates/next-code-protocol/src/wire.rs` | ✅ verified |
| 8 | `ServerEvent` enum has ~70 variants including `ToolStart/Input/Exec/Done`, `GeneratedImage`, `Compaction`, `SessionRenamed`, `TokenUsage`, `Notification`, `ModelChanged` | `crates/next-code-protocol/src/wire.rs:722-1444` | ✅ verified |
| 9 | `set_session_mode`, `set_session_model`, `ext_method`, `ext_notification` stubs do nothing | `src/cli/pager_agent.rs:210-327` | ✅ verified |
| 10 | Agent factory is installed in `pager_launch.rs` via `install_agent_factory()`; `no_leader=true` | `src/cli/pager_launch.rs` + `crates/xai-grok-pager/src/acp/spawn.rs` | ✅ verified |

### Verified (from research — Grok logic)

| # | Claim | Citation | Status |
|---|-------|----------|--------|
| U1 | `acp::SessionUpdate::ToolCall(ToolCall{id, title, kind, status})` is a typed variant accepted by Face `AcpUpdateTracker::handle_update()` | `xai-grok-pager/src/acp/tracker.rs:801-803` + test `.rs:1155-1168` | ✅ verified |
| U2 | `acp::SessionUpdate::ToolCallUpdate(ToolCallUpdate::new(id, fields).status(ToolCallStatus::*).raw_output(…))` is typed | `xai-grok-pager/src/app/acp_handler/tests/interjection.rs:259-263` | ✅ verified |
| U3 | `acp::SessionNotification::new(SessionId, SessionUpdate)` is the typed channel — not raw JSON | `test code` + `xai-acp-lib/src/gateway.rs:544-557` | ✅ verified |
| U4 | `AcpGatewaySender::session_notification()` is **fire-and-forget** (no response channel blocking) | `xai-acp-lib/src/gateway.rs:447 comment` | ✅ verified |

### Verified (open questions resolved)

| # | Claim | Citation | Status |
|---|-------|----------|--------|
| U5 | `Request::SetPermissionMode { .. } => {}` is a **no-op** in the daemon — comment says "handled via channels" but no handler exists. Permission mode is **entirely Face-local** — Face dispatch mutates `yolo_mode`, `permission_mode`, and `Effect::PersistPermissionMode` only persists config. No ACP message sent to agent. | `crates/next-code-app-core/src/server/client_lifecycle.rs:2650` + `crates/xai-grok-pager/src/app/dispatch/modes.rs:417-481` | ✅ verified |
| U6 | `Request::SetModel` IS handled by daemon via `handle_set_model()`. Model ID is a plain string passed to `agent.set_model(&model)` — accepts any model identifier. | `crates/next-code-app-core/src/server/provider_control.rs:499-543` | ✅ verified |
| U7 | No `ServerEvent::Permission*` variant exists (verified by grep). Permissions are handled entirely **server-side** within the daemon agent runtime — Face permission overlay never fires for daemon-side permission checks. | `crates/next-code-protocol/src/wire.rs` | ✅ verified |
| U8 | `x.ai/ask_user_question` is not handled by any daemon `Request` variant — would require daemon-side support. Defer to post-PR9. | grep daemon server handlers | ✅ verified (deferred) |

### Key insight: permission mode changes in Grok Face are **local-only**

Stock Grok Face handles permission mode entirely via Face-side dispatch:
```
SetPermissionMode(kind) → set_permission_mode(app, kind)
  → set_yolo_mode_inner(app, kind.is_always_approve())  // local state
  → app.current_ui.permission_mode = Some(kind.as_canonical())  // local state
  → Effect::PersistPermissionMode { canonical, session_id, ... }  // persist config, no ACP
```

No `set_session_mode` ACP message is sent to the agent for UI permission toggles. The `x.ai/yolo_mode_changed` ext notification is for settings UI sync, not for daemon routing. **Do NOT implement `set_session_mode` → `Request::SetPermissionMode`** — the daemon ignores it anyway.

**However**, `set_session_model` IS needed — the ACP gateway calls it when Face user selects a model, and the daemon DOES handle `Request::SetModel`. Implement `set_session_model` → `Request::SetModel { id, model: args.model.to_string() }`.

### Scope adjustment (after verification)

| Originally planned | Actual |
|-------------------|--------|
| Step 2: `set_session_mode` → `Request::SetPermissionMode` | ❌ **Skip** — Face handles locally, daemon no-ops |
| Step 3: `set_session_model` → `Request::SetModel` | ✅ **Keep** — daemon handles it |
| Step 1: Tool lifecycle mapping | ✅ **Keep** — still blocking |
| Step 4: Harden cancel | ✅ **Keep** — good practice |

---

## Gap analysis (ranked)

### 🔴 Blocking — Face unusable without these

| Gap | Symptom | Fix | Est. effort |
|-----|---------|-----|-------------|
| **B-Tool lifecycle dropped** | No tool visualization; agent looks broken | Map `ToolStart`/`ToolInput`/`ToolExec`/`ToolDone` → typed `session_notification()` calls in `prompt()` loop | 1-2 days |
| **B-Compaction dropped** | Stale scrollback after `/compress` or auto-compact | Map `Compaction` → `AgentMessageChunk` or clear-and-replay | 0.5 day |

### 🟠 High — significant degradation

| Gap | Symptom | Fix | Est. effort |
|-----|---------|-----|-------------|
| **A2 — `set_session_model`** | Face model picker has no effect | Implement: receive `SetSessionModelRequest` → send `Request::SetModel` to daemon socket | 0.5 day |
| **B-SessionRenamed** | Face title never updates on rename | Map `SessionRenamed{display_title}` → `SessionUpdate::SessionInfoUpdate` | 0.25 day |
| **B-TokenUsage** | No token feedback in Face | Map `TokenUsage` → ext notification or accumulate | 0.5 day |

### 🟡 Medium — edge cases / polish

| Gap | Risk | Fix | Priority |
|-----|------|-----|----------|
| **A3 — `ext_method`** | Plan-mode `ask_user_question` hangs | Implement `ext_method("x.ai/ask_user_question")` → needs daemon-side support too | Low unless plan-mode users |
| **A4 — `ext_notification`** | `yolo_mode_changed`, `permission_rules` don't reach daemon | Implement `ext_notification()` for key x.ai methods | Low |
| **D — DaemonSession** | Code duplication maintenance burden | Extract shared struct to `crates/next-code-protocol` | After PR9 |
| **U4 — Daemon permissions** | Face permission overlay may never fire | Investigate empirically; if daemon never emits permission ServerEvents, the vendored permission UI code is dead | Investigate before PR9 close |

### ⚪ Low

| Gap | Risk | Priority |
|-----|------|----------|
| `BatchProgress`, `MemoryActivity`, `SwarmStatus`, `StdinRequest` dropped | Niche event types | After PR9 |
| `ModelChanged`, `ReasoningEffortChanged` dropped | Stale labels in Face | After PR9 |
| `McpStatus`, `SidePaneImages`, `Notification`, `State` dropped | Minor info loss | After PR9 |
| `GeneratedImage` dropped | Image results invisible | Part of tool lifecycle fix |

---

## Implementation steps

### Step 1 — Extend `prompt()` loop with tool lifecycle mapping

**File:** `src/cli/pager_agent.rs`

Add structured `ServerEvent` → ACP notification mapping inside the `prompt()` loop (lines 293-313):

```rust
// After existing TextDelta/TextReplace arm, add:
ServerEvent::ToolStart { id, name } => {
    // Grok way: typed ACP channel, not JSON
    // Gateway sends SessionUpdate::ToolCall(ToolCall{...}) to Face
    self.emit_tool_call(&session_id, &id, &name).await;
}
ServerEvent::ToolInput { delta } => {
    self.emit_tool_input(&session_id, &delta).await;
}
ServerEvent::ToolExec { id, name } => {
    self.emit_tool_update(&session_id, &id, &name, acp::ToolCallStatus::InProgress).await;
}
ServerEvent::ToolDone { id, name, output, error } => {
    self.emit_tool_done(&session_id, &id, &name, &output, &error).await;
}
ServerEvent::GeneratedImage { id, path, output_format, revised_prompt, .. } => {
    self.emit_generated_image(&session_id, &id, &path, &output_format, revised_prompt.as_deref()).await;
}
ServerEvent::Compaction { trigger, .. } => {
    self.emit_text(&session_id, format!("\n[Context compacted: {trigger}]\n")).await;
}
ServerEvent::SessionRenamed { display_title, .. } => {
    self.emit_session_renamed(&session_id, &display_title).await;
}
ServerEvent::TokenUsage { .. } => {
    // Optional: emit as ext notification
}
```

**New helper methods** — Grok typed ACP (verified from Face test code + tracker.rs):

```rust
use acp::{ToolCallUpdate, ToolCallUpdateFields, ToolCallStatus, ToolCallId, SessionId};

async fn emit_tool_call(&self, session_id: &str, tool_id: &str, name: &str) {
    // Gateway.session_notification() is fire-and-forget (U4 verified)
    let _ = self.gateway.session_notification(
        acp::SessionNotification::new(
            SessionId::new(session_id),
            acp::SessionUpdate::ToolCall(
                acp::ToolCall::new(
                    ToolCallId::new(tool_id),
                    acp::ToolCallStatus::Pending,
                )
                .title(tool_title(name))
                .kind(tool_kind(name)),
            ),
        ),
    ).await;
}

async fn emit_tool_done(&self, session_id: &str, tool_id: &str, name: &str,
    output: &str, error: &Option<String>) {
    let fields = ToolCallUpdateFields::new()
        .status(ToolCallStatus::Completed)
        .title(tool_title(name))
        .kind(tool_kind(name))
        .raw_output(Some(serde_json::json!({
            "output": output,
            "error": error,
        })));
    let _ = self.gateway.session_notification(
        acp::SessionNotification::new(
            SessionId::new(session_id),
            acp::SessionUpdate::ToolCallUpdate(
                ToolCallUpdate::new(ToolCallId::new(tool_id), fields),
            ),
        ),
    ).await;
}

async fn emit_session_renamed(&self, session_id: &str, title: &str) { ... }
```

**Reference for typed shapes:** Face `AcpUpdateTracker::handle_update()` at `crates/xai-grok-pager/src/acp/tracker.rs:792` pattern-matches these typed variants. Face test code at `crates/xai-grok-pager/src/app/acp_handler/tests/mod.rs:1155` demonstrates exact construction.

**Helper to extract tool title/kind** (copy from `acp.rs`):

```rust
fn tool_title(name: &str) -> &str {
    if name.starts_with("Bash") { "Bash" }
    else if name.starts_with("Read") || name.starts_with("Glob") || name.starts_with("Grep") { "Read" }
    else if name.starts_with("Edit") || name.starts_with("Write") { "Edit" }
    else if name.starts_with("Web") { "Web" }
    else { name }
}
fn tool_kind(name: &str) -> &str {
    if name.starts_with("Bash") { "bash" } else if ... { ... } else { "tool" }
}
```

### Step 2 — Implement `set_session_model`

```rust
    async fn set_session_model(
        &self,
        args: acp::SetSessionModelRequest,
    ) -> acp::Result<acp::SetSessionModelResponse> {
        let model_id = args.model.to_string();
        let session_id = args.session_id.to_string();
        let session = self.sessions.borrow().get(&session_id).cloned();
        let Some(session) = session else { ... };
        let req_id = session.next_id();
        session.send(&Request::SetModel {
            id: req_id,
            model: model_id,
        }).await.map_err(|e| ...)?;
        Ok(acp::SetSessionModelResponse::new())
    }
```

⚠️ **Verify U3** — test with actual model IDs from daemon `History` response.

### Step 4 — Harden cancel path

Current `cancel()` sends `Request::Cancel` but doesn't drain the prompt loop. After cancel, stale `ServerEvent` values may arrive for the old prompt_id. Add a drain loop that discards events with the old ID:

```rust
async fn cancel(&self, args: acp::CancelNotification) -> acp::Result<()> {
    let session_id = args.session_id.to_string();
    if let Some(session) = self.sessions.borrow().get(&session_id).cloned() {
        let cancel_id = session.next_id();
        session.send(&Request::Cancel { id: cancel_id }).await.ok();
        // Drain stale events: read until Done/Error for cancel_id
        // (or use prompt_running flag)
    }
    Ok(())
}
```

### Step 5 — Unit tests

**File:** `src/cli/pager_agent.rs` or `src/cli/pager_acp_map.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_title_mapping() {
        assert_eq!(tool_title("Bash"), "Bash");
        assert_eq!(tool_title("Read"), "Read");
        assert_eq!(tool_title("Edit"), "Edit");
        assert_eq!(tool_title("WebSearch"), "Web");
    }
    
    #[tokio::test]
    async fn test_prompt_loop_tool_events() {
        // Mock daemon socket → inject ToolStart + TextDelta + ToolDone + Done
        // Verify gateway received 3+ notifications
    }
}
```

---

## Files to touch

| File | Action | Why |
|------|--------|-----|
| `src/cli/pager_agent.rs` | **Edit** | Add tool lifecycle helpers; implement `set_session_mode`, `set_session_model`; extend `prompt()` loop; harden cancel |
| `src/cli/pager_launch.rs` | **Read-only** (verify unchanged) | Entry point verified OK |
| `docs/plans/PLAN-20260720-grok-pr9-face-brain-harden.md` | **Edit** | Fill Evidence table as we go |

Future PRs:
- `src/cli/acp.rs` (PR14: extract shared `DaemonSession`)
- `crates/xai-grok-pager/` (PR10+ ⚠️ only if option-id mismatch)

---

## Open questions (all resolved)

- ✅ **Q1 (U5):** `Request::SetPermissionMode` is a **no-op** — daemon comment says "handled via channels" but no actual handler. Permission mode is entirely Face-local (dispatch → local state + toast). **Skip `set_session_mode` wire.**
- ✅ **Q2 (U6):** `Request::SetModel` IS handled by daemon (`apply_set_model()`). Accepts plain string model ID. **Implement `set_session_model` → `Request::SetModel`.**
- ✅ **Q3 (U7):** No `ServerEvent::Permission*` variant. Daemon handles permissions server-side. Face permission overlay never fires for daemon checks — **not a bug**, it's by design.

---

## Out of scope (this PR)

- Legacy TUI delete (`NEXT_CODE_LEGACY_TUI`)
- Slash command brand (groklink → next-code)
- Settings/slash UI
- Session dashboard
- `ext_method`/`ext_notification` (defer to PR10+)
- DaemonSession deduplication (defer to PR14)
- ModelChanged/ReasoningEffortChanged sync
- BatchProgress, MemoryActivity, SwarmStatus, StdinRequest
- Images as actual content in PromptRequest

---

## Manual smoke test (after BUILD)

1. **Tool call visible:** Run `Write a file` → Face shows tool lifecycle (pending → in_progress → completed with output)
2. **Text streaming:** Continue to work without visible regression
3. **Permission mode:** Toggle mode in Face → daemon actually changes mode
4. **Model switch:** Select different model in Face → daemon actually switches
5. **Cancel:** Ctrl+C during turn → clean stop, next prompt OK
6. **Resume:** Quit Face, relaunch → `--resume` works, tool results visible in scrollback

---

## Done when

- [x] Evidence table filled (verified citations for all claims)
- [x] Tools, text, and basic event flow work on Face via next-code daemon
- [x] `set_session_model` wired through
- [x] Unit tests for event mapper helpers
- [x] Manual smoke pass (tools/streaming/cancel/resume)
- [x] `cargo check -p next-code` passes
- [x] Release build + install (62b09230c-dirty)
