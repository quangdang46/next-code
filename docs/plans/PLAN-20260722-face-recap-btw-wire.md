# Plan Report — Face `/recap` + `/btw` wire

## Summary (read this first)
- **You asked:** Wire `/btw` and `/recap` (stub/ack only today) to real daemon brain answers — Face UI stays stock.
- **What is going on:** Face already paints `/btw` overlay and `/recap` scrollback. Daemon `face_ext::btw_payload` returns a “later turn” stub; `x.ai/recap` is unsupported and `sessionRecap` is not advertised (Face fail-closed).
- **We recommend:** Advertise `sessionRecap: true`; replace `/btw` stub with a tool-free `MultiProvider::complete` over session transcript; handle `x.ai/recap` in `pager_agent` → model summary → `SessionRecap` / `SessionRecapUnavailable` via existing `x.ai/session_notification`.
- **Risk:** Medium — live provider call; keep auto-recap best-effort; no new Face chrome.
- **Status:** Implementing now (user: wire luôn / implement now).

### Copy / wire / delete map

| Kind | What |
|------|------|
| **Keep** | Face `/btw` overlay + `/recap` slash + `SessionRecap` paint |
| **Wire** | `InitializeResponse.meta.sessionRecap = true` |
| **Wire** | `x.ai/btw` → session-context side answer `{result.answer}` |
| **Wire** | `x.ai/recap` → ack + emit `SessionRecap` / `SessionRecapUnavailable` |
| **Delete** | Stub “later turn” /btw copy |
| **Do not touch** | AskUserQuestion / permission-confirm PRs |

## Evidence
1. Face `Effect::SendBtw` / `SendRecap` → `x.ai/btw` / `x.ai/recap` — verified `crates/xai-grok-pager/src/app/effects/mod.rs`
2. Stock brain: tool-free side question + async `SessionRecap` — verified grok-build `acp_session_impl/recap.rs`
3. next-code stub ack — verified `src/cli/face_ext.rs` `btw_payload`
4. Fail-closed `/recap` until meta — verified `parse_session_recap_available`
5. Emit pattern — verified `pager_agent::emit_model_auto_switched` (`x.ai/session_notification`)
6. One-shot model path — verified `YoloClassifier` → `MultiProvider::complete(..., &[], ...)`

## Steps
1. [x] LOOK (stock + stubs)
2. [ ] Advertise `sessionRecap`
3. [ ] Real `/btw` answer from session context
4. [ ] Handle `/recap` + emit notification
5. [ ] Unit tests for clean/usage paths
6. [ ] `cargo check` targeted; push + PR

## Files to touch
- `src/cli/face_ext.rs` — side-call helpers + real btw
- `src/cli/pager_agent.rs` — meta + recap emit
- `docs/plans/PLAN-20260722-face-recap-btw-wire.md` — this file
