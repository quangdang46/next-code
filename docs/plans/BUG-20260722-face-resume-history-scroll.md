# Plan Report

## Summary (read this first)
- **You asked:** Why `nextcode --resume session_stallion_1784710338913_9e1828dd9444dfb0` opens Face but history never appears in the scroll view (binary `e32733d08`, PR12 + #59).
- **What is going on:** Session disk + daemon attach succeed (post-`d852037`), but `NextCodeFaceAgent::attach_session` fetches daemon `History` and **throws away `messages`**, returning `session/load` without the required ACP history replay. Face’s scrollback only fills from `session/update` chunks during load — so the pane stays empty.
- **We recommend:** In `pager_agent` `attach_session` / `load_session`, replay daemon history as ACP `user_message_chunk` / `agent_message_chunk` (with `_meta.isReplay: true`) **before** returning `LoadSessionResponse` — mirror `src/cli/acp.rs::replay_history` and stock Grok/ACP load semantics. Do not invent a second Face store path.
- **Risk:** Medium (wrong chunk shaping / missing `isReplay` can confuse Face dedup/adoption; tool-call history may need a follow-up pass).
- **Status:** Implemented — `attach_session` replays daemon `History.messages` as ACP `session/update` chunks with `_meta.isReplay: true` before chrome emits / `LoadSessionResponse`. Rebuild both aliases and smoke `nextcode --resume <id>`.

## Bug investigation
- **Verified root cause:** Face resume attaches the next-code session but never emits conversation `session/update` notifications during ACP `session/load`, so scrollback stays empty.
- **Hypotheses ranked:**
  1. **Load succeeds; Face never gets history events** — **verified**. `attach_session` matches `History { .. }` and discards `messages`; no `replay_history` / `UserMessageChunk` emits. ACP requires replay-before-response on `session/load`.
  2. **Wrong format flat session vs Grok nested `updates.jsonl`** — **verified as secondary design gap, not this session’s blocker**. Stock Grok replays persisted update lines; next-code has flat `Session.messages` + daemon `HistoryMessage`. Bridge must synthesize ACP chunks from daemon history (already done in `src/cli/acp.rs`, missing in Face agent).
  3. **Scrollback not refreshed / PR12 store incomplete** — **unverified as primary**; Face opens and chrome floats (provider/memory/git/todos/skills) emit — only transcript path is missing.
  4. **Journal not read** — **ruled out for this id**. No `.journal.jsonl` for this session; flat JSON already has 7 messages.
  5. **Empty / useless on-disk content** — **ruled out**. File has real user/assistant text (`"2"`, `"3"`, ready replies).
- **Ruled out:** Prior “Session does not exist” preflight (`d852037` / `defer_existence_to_agent`) — Face opens now. Quit-hint “resume-only tail” product choice (no transcript on quit print) is unrelated to in-Face resume scrollback.
- **Sub-agents used:** skipped — narrow wire bug with direct file + protocol proof.
- **Citations checked:** listed under Evidence.

## Feature planning
N/A (bug).

## Evidence
1. **On-disk session (verified):** `%USERPROFILE%\.next-code\sessions\session_stallion_1784710338913_9e1828dd9444dfb0.json` exists (~5413 bytes). Top-level flat next-code shape: `id`, `messages` (7), `provider_key`, `model`, `working_dir` = demo-retell-ai-dashboard, `status` = `Closed`, multiple `env_snapshots` with `reason: resume` and `next_code_git_hash: e32733d08`. Message texts include user `"2"`/`"3"` and assistant ready lines. Sibling `.bak` present. **No journal** for this id (only unrelated `session_ox_*.journal.jsonl` in tree).
2. **Face launch seam (verified):** `src/cli/pager_launch.rs` maps CLI resume → `pager_args.resume_session`, `no_leader`, installs `NextCodeFaceAgent`.
3. **Face ACP attach (verified):** `src/cli/pager_agent.rs` `attach_session` — `Subscribe { target_session_id }`, `wait_for_done`, `GetHistory` / `request_history`, then:
   ```rust
   ServerEvent::History {
       session_id, provider_name, provider_model, available_models, ..
   }
   ```
   Messages ignored. Emits provider/memory/git/todos/skills only. `load_session` → `attach_session` → empty `LoadSessionResponse` (+ models). No history chunk emits.
4. **Working ACP reference in same repo (verified):** `src/cli/acp.rs` `handle_session_load(..., replay_history: true)` → `attach_existing_session` → `replay_history` emits `session/update` with `user_message_chunk` / `agent_message_chunk` from `HistoryMessage` **before** result. Face agent never calls an equivalent.
5. **ACP contract / stock Grok (verified):** [ACP session setup](https://agentclientprotocol.com/protocol/v1/session-setup) — on `session/load`, agent **MUST** replay conversation via `session/update`, then respond. Face comments (`crates/xai-grok-pager/src/acp/meta.rs`) expect `_meta.isReplay` stamped on load replay (stock shell `forward_raw_replay_line` / updates storage). Exa/GitHub: grok-build stores ACP updates and reloads them for resume.
6. **Face scrollback fill path (verified):** `AcpUpdateTracker` / `session.handle_update` turn `SessionUpdate` into scrollback; resume uses `loading_replay` until `SessionLoaded`. Without chunks during load, scroll view stays empty after successful load.
7. **Origin next-code-tui (verified by code; live A/B unverified):** `crates/next-code-tui/.../server_events.rs` applies `ServerEvent::History` into `display_messages` when `should_apply_history_payload`. Same daemon history **would** populate legacy TUI. Live `NEXT_CODE_LEGACY_TUI=1 --resume <id>` A/B not run in this LOOK.
8. **Prior fix scope (verified):** `d852037` only deferred Grok disk existence + Subscribe handshake — explicitly did not add history replay. Prior BUG: `docs/plans/BUG-20260722-face-resume-session-does-not-exist.md`.
9. **Local `grok-build/`:** not present in workspace; stock behavior taken from vendored Face comments + Exa/ACP docs.

## Steps (simple checklist)
1. [x] In `NextCodeFaceAgent::attach_session`, bind `messages` from `ServerEvent::History` (stop discarding via `..`).
2. [x] Before returning from `load_session` / `attach_session`, replay each message as ACP `session/update`:
   - `user` → `UserMessageChunk`
   - `assistant` (and other) → `AgentMessageChunk`
   - stamp `_meta.isReplay: true` (`ReplayMetaStamp::replayed()` / equivalent on `SessionNotification`)
3. [x] Prefer reusing / sharing logic with `src/cli/acp.rs::replay_history` (single mapping from `HistoryMessage` → chunk).
4. [x] Decide tool-call / reasoning / system-reminder fidelity for v1 (text-only like `acp.rs` is enough to unblock scrollback).
5. [x] Regression test: attach_session / load_session emits N chunk notifications for a fixture History with N messages, then returns.
6. [x] Smoke: rebuild both aliases installed (`scripts/_tmp_rebuild_install.ps1`); unit tests pass. Operator: `nextcode --resume session_stallion_1784710338913_9e1828dd9444dfb0` — confirm prior turns in Face scrollback.
7. [ ] Optional A/B: `NEXT_CODE_LEGACY_TUI=1 --resume <same id>` confirms origin TUI still shows history.

## Files to touch
- `src/cli/pager_agent.rs` — **wire**: replay history on `load_session` / `attach_session`
- Optionally extract shared helper near `src/cli/acp.rs::replay_history` (DRY)
- Test under `src/cli/` or existing pager_agent tests
- No Face scrollback rewrite; no Grok disk store invent

## Copy / delete / wire map
| Kind | Action |
|------|--------|
| **Copy** | Reuse stock/ACP “replay updates during `session/load`” contract; reuse existing `acp.rs` chunk mapping |
| **Delete** | None |
| **Wire** | Daemon `History.messages` → ACP `session/update` before `LoadSessionResponse` in Face agent |
| **Do not** | Read Grok `updates.jsonl` for next-code sessions; patch Face to special-case next-code JSON |

## Open questions (≤3)
1. v1 text-only replay (like `acp.rs`) vs also emitting tool_call / thought chunks from richer session JSON?
2. Should `session/resume` (no replay) stay unused by Face, or do we ever want attach-without-scrollback?
3. Filter `display_role: system` / system-reminder blobs from scrollback, or show as today in TUI?

## If you want more detail
### Call chain
`nextcode --resume <id>` → `pager_launch::run_face_pager` → Face `materialize` Resume (`defer_existence_to_agent`) → ACP `session/load` → `NextCodeFaceAgent::load_session` → `attach_session` (Subscribe + GetHistory) → **missing replay** → `SessionLoaded` with empty scrollback.

### Why floats can work while transcript does not
Post-attach emits are metadata (`emit_provider_name`, `emit_memory_info`, `emit_git_status`, `emit_todos_plan`, `emit_available_skills`) — not conversation chunks. That matches “Face opens, chrome ok, history blank.”
