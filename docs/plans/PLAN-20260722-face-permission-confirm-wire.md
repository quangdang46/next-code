# Plan Report — Face permission confirm wire

## Summary (read this first)
- **You asked:** Wire tool-approval permission confirm end-to-end for Face (same migration shape as AskUserQuestion: copy → wire → delete old if needed). Parallel to AskUserQuestion; do not touch `question_view`.
- **What is going on:** Face already paints `permission_view` on ACP `session/request_permission`. Daemon never emits that ACP method. It publishes in-process `BusEvent::PermissionRequested` + `await_permission_response()` oneshot — which socket clients (Face / remote TUI) never see. Overlay is dead for Face.
- **We recommend:** Forward bus permission → `ServerEvent::PermissionRequest` → Face `pager_agent` calls `Client::request_permission` → map outcome → `Request::PermissionResponse` → daemon applies allow/deny side effects + `signal_permission_response`. Keep Face `permission_view` (already vendored). Do not re-home into `next-code-tui`.
- **Risk:** Medium — blocking reverse-request mid-turn; must not conflict with AskUserQuestion wire (`question_view` / `x.ai/ask_user_question`).
- **Status:** Implementing (user said **làm luôn**).

### Copy / wire / delete map

| Kind | What | Notes |
|------|------|--------|
| **Copy (keep)** | Face `permission_view` + `acp_handler::permissions` | Already vendored; do not rewrite |
| **Wire** | `ServerEvent::PermissionRequest` + `Request::PermissionResponse` | Daemon ↔ Face socket (stdin / ask_user pattern) |
| **Wire** | `client_lifecycle` bus → ServerEvent; response → `signal_permission_response` | Mirrors TUI dialog side effects on daemon |
| **Wire** | `face_permission.rs` + `pager_agent` event arm | ACP `request_permission` emit |
| **Delete / gate** | Do **not** invent Face-local permission UI | Keep Grok overlay |
| **Do not touch** | `question_view` / AskUserQuestion | Other PR |

---

## Evidence

| # | Claim | Status | Citation |
|---|-------|--------|----------|
| E1 | Face handles ACP `RequestPermission` → `permission_queue` | **verified** | `acp_handler/mod.rs` + `permissions.rs` |
| E2 | Gateway Agent→Client `request_permission` is blocking forward | **verified** | `xai-acp-lib/src/gateway.rs` |
| E3 | Daemon Prompt path: Bus + `await_permission_response` | **verified** | `turn_execution.rs` ~962–1012 |
| E4 | Bus PermissionRequested **not** forwarded to socket clients | **verified** | `client_lifecycle.rs` bus match — ModelsUpdated/Batch/… only |
| E5 | `pager_agent` has zero permission emit | **verified** | grep `src/cli/pager_agent.rs` |
| E6 | Stock Grok: permission is blocking ACP reverse-request | **verified** | grok-build `pending_interaction.rs` `PendingKind::Permission` |
| E7 | Legacy TUI applies side effects then `signal_permission_response` | **verified** | `next-code-tui/.../input.rs` ~2180+ |
| E8 | AskUserQuestion parallel pattern | **verified** | sibling worktree `face_ask_user.rs` + `ServerEvent::AskUserQuestion` |

---

## Architecture (target)

```text
dcg Prompt
  → BusEvent::PermissionRequested + await oneshot
  → client_lifecycle forward ServerEvent::PermissionRequest
  → Face pager_agent → gateway.request_permission (ACP)
  → permission_view (Allow once / Always / Allow all / Reject)
  → Request::PermissionResponse { outcome }
  → daemon: approve_* / deny + signal_permission_response
  → tool continues or fails
```

**Permission modes:** Face YOLO auto-approves `AllowOnce` locally (existing). Daemon `dcg_bridge` modes unchanged. Do not implement `SetPermissionMode` ACP→daemon (PR9 U5).

**Option ids (Face ACP):** `allow-once`, `allow-always`, `allow-all`, `reject-once` — mapped to TUI dialog actions.

---

## Steps
1. [x] Protocol wire variants
2. [x] Daemon bus forward + response handler
3. [x] Face bridge + pager_agent arm
4. [x] Unit tests for outcome mapping
5. [ ] `cargo check` / targeted tests
6. [ ] Push + PR

## Files to touch
- `crates/next-code-protocol/src/wire.rs`, `lib.rs`
- `crates/next-code-app-core/src/server/client_lifecycle.rs`, `client_actions.rs`
- `src/cli/face_permission.rs` (new), `mod.rs`, `pager_agent.rs`
- `docs/plans/PLAN-20260722-face-permission-confirm-wire.md`
