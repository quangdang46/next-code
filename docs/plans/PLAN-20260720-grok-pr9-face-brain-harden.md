# Plan Report — PR9 Face brain harden (ACP bridge)

## Summary (read this first)
- **You asked:** Face must use next-code brain for tools, permissions, streaming.
- **What is going on:** `NextCodeFaceAgent` mostly `emit_text` after `Message`. Face expects ACP tool/permission/thinking updates.
- **We recommend:** **Wire-only** at ACP↔daemon seam. **Copy** stock Face’s expected ACP shapes (not rewrite Face). **Delete** nothing of old TUI here.
- **Risk:** High  
- **Status:** Scope approved for home implement — fill Evidence before coding.

## Workflow map (required)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | Match stock Face `SessionUpdate` / permission option UX from grok-build + vendored `acp_handler` / `dispatch/permissions.rs` | Invent a parallel UI in `next-code-tui` |
| **Wire** | `ServerEvent` → ACP notifications; ACP permission reply → daemon approve/deny | Add `GrokHost` |
| **Delete** | None this PR | Do not delete Face permission modal |

## Research first (LOOK)
1. DeepWiki / grok-build: agent→client tool and permission notifications.
2. Vendored Face: `crates/xai-grok-pager/src/app/acp_handler/`, `dispatch/permissions.rs`.
3. next-code: `ServerEvent` / permission requests in protocol + server.
4. Seam: `src/cli/pager_agent.rs` `prompt` loop.

Every row in the mapping table below must be `verified` (path:line) or `unverified — needs X`.

## Evidence (fill before BUILD)

| Claim | Citation | Status |
|-------|----------|--------|
| Current prompt path only emits text chunks | `src/cli/pager_agent.rs` | verified (pre-audit) |
| Face permission UI driven by ACP session updates | `…/dispatch/permissions.rs` | unverified — needs line |
| Daemon emits tool/permission events during turn | `src/protocol` / server | unverified — needs enum list |

## Copy / wire / delete
| Action | What |
|--------|------|
| **Copy pattern** | Stock Face permission option ids; YOLO ↔ `enable-always-approve` if daemon uses different id |
| **Wire** | Full `ServerEvent` set used in a tool turn → ACP |
| **Wire** | Cancel both ACP + `Request::Cancel` |
| **Delete** | — |

## Implementation steps
1. [ ] Inventory `ServerEvent` during tool turn → mapping table (Evidence).
2. [ ] Replace coarse `emit_text` with typed ACP updates.
3. [ ] Permission round-trip.
4. [ ] Cancel path.
5. [ ] Unit tests for mapper; `cargo check -p next-code`.
6. [ ] Manual smoke below.

## Files
- `src/cli/pager_agent.rs` (+ optional `pager_acp_map.rs`)
- Prefer **not** editing Face unless option-id mismatch forces it

## Manual verify
1. Tool call visible in Face.  
2. Deny / allow.  
3. Ctrl+C cancel.  
4. Resume after quit.

## Open questions (≤3)
1. Which daemon events are authoritative for permissions (exact enum)?  
2. Does YOLO live only in Face, only in daemon, or both?  
3. Images/multimodal in `PromptRequest` — in or defer?

## Out of scope
TUI delete, settings, session dashboard, slash brand.

## Done when
Tools + permissions + stream work on Face via next-code daemon; Evidence table filled.
