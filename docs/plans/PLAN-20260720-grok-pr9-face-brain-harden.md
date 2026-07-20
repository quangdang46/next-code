# Plan Report — PR9 Face brain harden (ACP bridge)

## Summary (read this first)
- **You asked:** Next PR after entry cutover so Face can really use next-code brain (tools, permissions, streaming).
- **What is going on:** `NextCodeFaceAgent` (`src/cli/pager_agent.rs`) does `Subscribe` / `ResumeSession` / `Message` / `Cancel` and mostly `emit_text` for replies. Face expects full ACP session updates (tool calls, permissions, thinking chunks). Without this, UI looks “chat-only” and permissions never appear.
- **We recommend:** Extend the ACP↔daemon adapter only — **do not** introduce `GrokHost`. Map daemon `ServerEvent` → ACP `SessionUpdate` / permission requests the way stock Face expects.
- **Risk:** High (ordering, cancel, permission deadlock)
- **Status:** Waiting for implementer — reply **go ahead** if working in-session; at home just follow this file.

## Goal for this PR
Face + provider: user can chat, see streaming tokens, run tools with Face permission UI (or YOLO), cancel mid-turn, without falling back to legacy TUI.

## Research first (LOOK)
1. DeepWiki / grok-build: how Face handles `SessionUpdate` tool / permission notifications.
2. Read next-code: `src/protocol` `ServerEvent` variants (tool, permission, stream, done).
3. Read Face: `crates/xai-grok-pager/src/app/acp_handler/` + `dispatch/permissions.rs`.
4. Read current: `src/cli/pager_agent.rs` end-to-end `prompt` loop.

Mark every mapping `verified` with file:line before coding.

## Copy / wire / delete
| Action | What |
|--------|------|
| **Wire** | `ServerEvent::*` → ACP notifications on `AcpGatewaySender` |
| **Wire** | ACP permission response → daemon approve/deny request |
| **Copy pattern** | Stock Face permission option ids; map YOLO ↔ `enable-always-approve` if needed |
| **Delete** | None of old TUI |

## Implementation steps
1. [ ] Inventory all `ServerEvent` variants used during a tool-heavy turn; table → ACP update type.
2. [ ] Replace coarse `emit_text` loop with typed mapping (assistant text chunks, tool start/end, errors).
3. [ ] Implement permission request path: daemon asks → Face modal → user choice → daemon continue.
4. [ ] Ensure `cancel` aborts both ACP turn and daemon `Request::Cancel`.
5. [ ] Tests: unit map tables + one integration if harness exists; else manual script below.
6. [ ] `cargo test` targeted + `cargo check -p next-code`.

## Files (expected)
- `src/cli/pager_agent.rs` (primary)
- Possibly `src/protocol` helpers / thin mappers module `src/cli/pager_acp_map.rs`
- Face: only if option id strings must match — prefer change adapter not Face

## Manual verify
1. Fresh `nextcode` → prompt that needs a tool (e.g. list dir) → see tool UI in Face.
2. Deny once → turn stops cleanly; allow → tool runs.
3. Mid-stream Ctrl+C → cancel; no hung spinner.
4. Quit → `nextcode --resume <id>` restores transcript.

## Out of scope
- Deleting `next-code-tui*`
- Settings persistence
- Session dashboard list

## Done when
Tool + permission + stream work on Face with next-code daemon; no GrokHost.
