# BUG — Face startup “Starting session…” / MCP seed (SECONDARY)

Branch: `pr-face-startup-mcp-clear` (abandoned as primary — see corrected bug)  
Related: **`BUG-20260723-face-model-list-empty-on-start.md`** ← **user’s real symptom**  
Also: `BUG-20260723-face-model-persist-mismatch.md` / PR #69  
Status: **Superseded as primary** — MCP claim demoted

## Correction (2026-07-23 pushback)

User: **「này liên quan gì đến MCP?? vấn đề là mới vào gõ /model không hiện ra list model」**

Agree: treating MCP spinner as the main bug was a **wrong primary**. Real complaint = **`/model` empty list on cold enter** (Popular providers all `connect`, footer `unknown`).

| Topic | Role |
|-------|------|
| Empty `/model` / `unknown` | **Primary** → `BUG-20260723-face-model-list-empty-on-start.md` |
| `Starting session…` MCP seed never cleared | **Secondary** embed UX only — does **not** fill model inventory |
| kimi / OPENROUTER / bad `opencode go` pin | Separate — #69 |

**Do not** open or merge an MCP-clear-only PR as the fix for `/model`. Implementer work on MCP clear was **interrupted / aborted**; tree should not ship that as the answer to this pushback.

---

## Original MCP finding (kept for secondary work)

Face seeds `McpInitProgress { total: 0 }` on `CreateSession` and expects shell ACP `x.ai/mcp/init_progress` + `x.ai/mcp_initialized`. `NextCodeFaceAgent` historically never emitted those → spinner up to **30s** after session ready.

That can make startup *feel* unloaded and widen the race where the user types `/model` before `SessionModelState` binds — but clearing MCP **without** fixing catalog bridge still leaves Popular-connect if `ModelState` is empty.

### If revisiting MCP later (optional)

| Kind | Change |
|------|--------|
| **Wire** | After create/attach: emit `x.ai/mcp/init_progress` then `x.ai/mcp_initialized` (stock parity) |
| **Prove** | Cold open → no hung `Starting session…` |
| **Not a substitute for** | `AvailableModelsUpdated` → `x.ai/models/update` + non-empty cold `SessionModelState` |

## Decision gate

**Superseded** — follow `BUG-20260723-face-model-list-empty-on-start.md` for BUILD; MCP only as optional follow-up.
