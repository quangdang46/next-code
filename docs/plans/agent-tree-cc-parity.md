# Plan: Agent Tree / Teammate View — Claude Code Parity

> **Branch:** `feat/agent-tree`  
> **Baseline HEAD (plan authored):** `bfac54414`  
> **Implementation progress (this doc):** Phase 0–4 partially landed on `feat/agent-tree` after plan commit.  
> **Primary reference:** [claude-code-best/claude-code](https://github.com/claude-code-best/claude-code) (local clone: `/tmp/feature-research/claude-code`)  
> **Secondary:** jcode `docs/SWARM_ARCHITECTURE.md`, `docs/AGENT_TREE_CC_GAP_ANALYSIS.md` (partially stale — re-baselined below)  
> **Goal:** User can freely navigate team-lead ↔ subagents with CC-grade UX, see **real** agent work (not spawn-meta spam), and always know how to return.

### Implementation status (rolling)

| Phase | Status | Notes |
|-------|--------|-------|
| 0 Ops | **Done** | `scripts/restart_local_serve.sh`, Agents.md install note, git hash in view chrome |
| 1 Stabilize | **Done** | `step_selected_index` pure + tests; soft/hard chrome honesty |
| 2 Protocol | **MVP done** | `SwarmMemberMessage` wire + server emit from output_tail/tools + client buffer |
| 3 Soft real + key flip | **Done** | Enter = soft (buffer when live); Shift+Enter = hard full session |
| 4 Polish | **Partial** | Stable name→color hash; kill/pills/preview still open |
| 5 Harden | **Partial** | Unit tests for selection; full multi-agent stress still open |

---

## 1. Executive summary

Claude Code treats “enter subagent” as an **in-process view switch**: `spawnInProcessTeammate` creates a task with `messages[]`; the runner appends every user/assistant/tool chunk; `enterTeammateView` only sets `viewingAgentTaskId` and the REPL swaps `displayedMessages = task.messages`. Esc / Enter on team-lead exits. No socket resume.

jcode swarm members are **separate remote sessions**. Today we have:

| Mode | What it is | Real transcript? | Free switch? |
|------|------------|------------------|--------------|
| **Hard attach** (`Enter` / `f`) | `resume_session(child)` + roster snapshot | **Yes** (full child history) | Yes (tree snapshot + Esc / Enter lead) |
| **Soft preview** (`Shift+Enter`) | Rebuild UI from `SwarmStatus` | **No** (`output_tail` / detail only) | Yes (stay on lead) |

**Chosen product strategy (hybrid, recommended):**

1. **Keep hard attach as the default “open agent” path** (only path that is truly real today).  
2. **Harden free-switch chrome + bare-↓ selection** until rock-solid (Phase 0–1).  
3. **Build a real soft-view protocol** (`SwarmMemberTranscript` events → client buffer ≈ CC `task.messages`) so soft Enter can match CC mental model without leaving the lead session (Phase 2–3).  
4. Polish (pills, kill fidelity, colors) after the data path is real (Phase 4).

**Calendar (1 senior-ish engineer, full-time):**

| Milestone | Duration |
|-----------|----------|
| M0 Stabilization (ship-quality hard path) | **1–2 days** |
| M1 Soft “rich preview” without new wire (optional) | **1.5–3 days** |
| M2 Protocol: per-member message stream | **4–8 days** |
| M3 Soft view = real buffer + input/kill parity | **2–4 days** |
| M4 Polish + hardening | **2–4 days** |
| **Total to >90% CC model** | **~2–3 weeks** |
| **MVP “no more stuck UX” only** | **~2–4 days** |

---

## 2. Source of truth — Claude Code Best

### 2.1 Mental model

```
spawnInProcessTeammate(config)
  → AppState.tasks[taskId] = { type: 'in_process_teammate', messages: [], identity, ... }
  → runWithTeammateContext → inProcessRunner appends every Message to task.messages

Shift+↑/↓  → viewSelectionMode = 'selecting-agent', selectedIPAgentIndex = -1..n (hide)
Enter / f  → enterTeammateView(taskId)
             viewingAgentTaskId = taskId
             REPL: displayedMessages = task.messages   // REAL

Esc (viewing, running) → abort currentWorkAbortController only
Esc (viewing, terminal) → exitTeammateView → leader messages again

Footer path (separate surface):
  ↓ manage → focus tasks/bg_agent pill
  bare ↑/↓  → step agents
  Enter     → enterTeammateView / exitTeammateView
```

### 2.2 Key files (read before coding)

| Concern | Path |
|---------|------|
| Enter / exit view | `src/state/teammateViewHelpers.ts` |
| Shift+↑/↓, Enter, f, k, Esc | `src/hooks/useBackgroundTaskNavigation.ts` |
| Tree render | `src/components/Spinner/TeammateSpinnerTree.tsx`, `TeammateSpinnerLine.tsx` |
| Select hint | `src/components/Spinner/teammateSelectHint.ts` → `"shift + ↑/↓ to select"` |
| Header | `src/components/TeammateViewHeader.tsx` → `Viewing @name · esc return` |
| Message swap | `src/screens/REPL.tsx` (`displayedMessages` / `viewedAgentTask`) |
| Real spawn | `src/utils/swarm/spawnInProcess.ts` |
| Stream → messages | `src/utils/swarm/inProcessRunner.ts` (~1650 LOC) |
| Footer bare arrows | `src/keybindings/defaultBindings.ts` (context `Footer`), `PromptInput.tsx` `footer:down` |
| Auto-exit | `src/hooks/useTeammateViewAutoExit.ts` |

### 2.3 Keyboard contract (two surfaces)

| Surface | Keys | Notes |
|---------|------|-------|
| Spinner tree | **Shift+↑/↓** only to select | Bare ↓ does **not** step tree |
| Footer pills | bare **↓** manage, then bare **↑/↓** | After footer focus |
| Confirm | Enter / f | `enterTeammateView` |
| Exit view | Esc (or Enter on team-lead while selecting) | |

jcode currently merges: Shift always + **bare ↓ when chat already at bottom** + bare ↑/↓ while selecting/viewing. That is intentional ergonomics, not a 1:1 CC tree map — document in UI hints.

---

## 3. Current jcode baseline (as of `bfac54414`)

### 3.1 What works

- Agent tree under input; single-line glyphs; only running (+ hard-attach snapshot) members.  
- Selection: Shift+↑/↓; bare ↓ at bottom; bare ↑/↓ while selecting/viewing.  
- **Enter / f → hard attach** (`begin_teammate_hard_attach` + `queue_resume_session`).  
- **Shift+Enter → soft preview** (SwarmStatus rebuild).  
- Hard attach: freeze `teammate_view_swarm_snapshot`, keep return `session_id`, Esc / Enter team-lead resumes lead.  
- Chrome: header `Viewing @… · esc return`, separator, status spans, input hint.  
- Soft submit: `CommMessage` DM / `notify_session` while soft-viewing.  
- Interrupt: does not wipe hard-attach return state.

### 3.2 What is still wrong / incomplete

| ID | Gap | Severity |
|----|-----|----------|
| G-DATA | Soft view ≠ agent transcript; often empty `output_tail` | P0 product honesty |
| G-PROTO | No per-member structured message stream to client | P0 for CC soft model |
| G-HARD | Hard attach leaves lead session; tree depends on snapshot fidelity | P1 |
| G-RESUME | Serve/client version skew; reload fails → user tests dead binary | P0 ops |
| G-KILL | `k` / stop fidelity vs CC `InProcessTeammateTask.kill` | P1 |
| G-ABORT | Esc abort turn is notify hack, not native abort controller | P1 |
| G-FOOTER | No true footer pills surface | P2 |
| G-STATS | Tool uses / tokens partial | P2 |
| G-CLICK | No click-to-enter | P2 |
| G-COLOR | Weak per-agent identity colors | P2 |
| G-TEST | Few integration tests for nav state machine | P1 |

### 3.3 Architecture constraint (do not ignore)

```
CC:   one process  → task.messages[]  → UI swap
jcode: multi-session → SwarmStatus snapshots + resume_session
```

Soft view cannot become “real” without either:

- **(A)** streaming structured messages for each member to the **lead** client, or  
- **(B)** always hard-attaching (CC mental model lost; free switch is session hopping).

Plan chooses **A for soft**, **B as default Enter until A ships**.

---

## 4. Architecture decision

### Chosen approach: Hybrid (hard default → soft real later)

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| Soft-only (CC clone) | Free switch, stay on lead | Needs protocol; today fake | Phase 2–3 target |
| Hard-only | Real transcript now | Resume cost; tree must snapshot | Default Enter now |
| Hybrid | Best of both; honest labels | Two modes to maintain | **Chosen** |

### State machine (target)

```
                    ┌─────────────────────────────────────────┐
                    │ viewSelectionMode                       │
                    │  none | selecting | soft_view | hard_attach │
                    └─────────────────────────────────────────┘

none ──(↓@bottom | Shift+↓)──► selecting
selecting ──Esc──► none
selecting ──Enter on lead──► none (or exit view)
selecting ──Enter on agent──► hard_attach  (M0–M1 default)
selecting ──Shift+Enter──► soft_view       (preview)
selecting ──f──► same as Enter

soft_view ──Esc──► none
soft_view ──↑/↓──► selecting (stay soft until Enter)
soft_view ──Enter agent──► hard_attach     (upgrade to real)
soft_view ──Enter lead──► none

hard_attach ──Esc | Enter lead──► resume lead ──► none
hard_attach ──↑/↓ + Enter other──► resume other (keep return=original lead)

After Phase 3:
selecting ──Enter──► soft_view (real buffer)
selecting ──Shift+Enter | 'o'──► hard_attach
```

### Client fields (existing + planned)

```rust
// Existing (App)
viewing_teammate_session_id: Option<String>
teammate_view_agent_name: Option<String>
teammate_view_messages: Vec<DisplayMessage>          // soft only
teammate_view_return_session_id: Option<String>      // hard only
teammate_view_hard_attached: bool
teammate_view_swarm_snapshot: Vec<SwarmMemberStatus>
agent_tree_selecting / selected_agent_tree_index / agent_tree_hidden

// Phase 2+ (new)
// sid → live message buffer (CC task.messages analogue on lead client)
teammate_transcripts: HashMap<String, TeammateTranscript>
// TeammateTranscript { messages: Vec<DisplayMessage>, updated_at, capped }
```

### Protocol (Phase 2)

Extend wire (names illustrative; bikeshed at implement time):

```rust
// Server → clients attached to coordinator / swarm
ServerEvent::SwarmMemberMessage {
    swarm_id: String,
    session_id: String,
    // Stable message identity for dedupe
    message_id: String,
    role: String,           // user | assistant | system | tool
    content: String,        // capped plain text / rendered summary
    tool_name: Option<String>,
    ts_ms: u64,
}

// Optional bootstrap when entering soft view
Request::GetSwarmMemberTranscript {
    id: u64,
    session_id: String,
    limit: u32,             // e.g. last 200 messages
}
ServerEvent::SwarmMemberTranscript {
    id: u64,
    session_id: String,
    messages: Vec<SwarmMemberMessageDto>,
}
```

**Producer side:** tap the same places that already emit text/tool for a session (`dispatch_swarm_output_tail` is tail-only; Phase 2 hooks turn stream / tool bus into structured append + fanout to swarm peers). Cap payload (e.g. 2–4 KB/message, 100–200 msgs retained per member).

---

## 5. Phased implementation plan

### Phase 0 — Ops & measurement (0.5 day) — **do first every time**

**Why:** User repeatedly tested stale serve (`lsof` showed old binary). All UX work is worthless without current client+serve.

| Task | Detail | Done when |
|------|--------|-----------|
| 0.1 Install discipline | After every ship: `scripts/install_release.sh --fast` | Symlink → new hash |
| 0.2 Kill stale serve | Document one-liner; optional `jcode serve --force` | `lsof -p $PID` → new binary |
| 0.3 Version surface | Status notice or `/debug` shows short hash | User can confirm build |
| 0.4 Manual script | Checklist in §8 | Run once green |

**Files:** `scripts/install_release.sh`, optionally `crates/jcode-tui` status line / `Agents.md` note.

---

### Phase 1 — M0 Stabilization (1–2 days)

Goal: **Hard attach free-switch is boringly reliable.** No new protocol.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 1.1 | Audit bare-↓ only when `!auto_scroll_paused` + empty input; no steal of prompt history | `tui_state.rs` `handle_agent_tree_navigation_key` | History Up/Down still works when not at bottom |
| 1.2 | Unit tests: selection indices dense; viewing paints return hint; hard attach keeps `return_session_id` across agent hop | `agent_tree.rs`, new tests on pure helpers | `cargo test -p jcode-tui free_switch selection` |
| 1.3 | Hard attach: never clear snapshot until home; leader row always `esc/enter return` | `tui_state.rs`, `agent_tree.rs`, `remote.rs` | After Enter agent, tree still shows lead+agent |
| 1.4 | Esc path single owner (no double-clear return id) | `key_handling.rs`, `tui_state.rs` | Esc from hard → lead once |
| 1.5 | Soft labels honest: “status preview (not full transcript)” | `teammate_view.rs`, notices | No claim of full history on soft |
| 1.6 | Document key contract in tree hints | `agent_tree.rs` constants | Matches product table §2.3+jcode merge |

**Exit criteria Phase 1:** Manual §8 path 1–6 passes on freshly installed binary; no stranded hard attach without chrome.

---

### Phase 2 — Protocol: member message stream (4–8 days)

Goal: Lead client can hold a **real-ish** buffer per agent without resuming.

| # | Task | Files (expected) | Acceptance |
|---|------|------------------|------------|
| 2.1 | Design wire DTO + caps + versioning | `jcode-protocol` | Serialize tests |
| 2.2 | Server: on assistant/tool events for swarm members, append + broadcast | `jcode-app-core` server bus / swarm | Coordinator receives events when worker streams |
| 2.3 | Cap store per member in swarm state | swarm member struct | Memory bounded |
| 2.4 | Client: `teammate_transcripts` map; apply events | `server_events.rs`, `app.rs` | Buffer grows while on lead |
| 2.5 | Bootstrap: `GetSwarmMemberTranscript` or piggyback first soft enter | backend + server | Enter soft mid-run shows history not empty |
| 2.6 | Dedupe by `message_id` | client apply | No duplicate lines on rebroadcast |

**Risks:**  
- Bandwidth if full tool JSON — **summarize tool rows** (name + short intent), full text only for assistant.  
- History size on resume — cap + disk optional later.

**Exit criteria Phase 2:** With 1 worker streaming, lead client buffer has ≥N assistant chunks without hard attach; unit + 1 integration test.

---

### Phase 3 — Soft view = real buffer; product flip (2–4 days)

Goal: **Enter soft becomes CC-like**; hard is opt-in.

| # | Task | Files | Acceptance |
|---|------|-------|------------|
| 3.1 | Soft build from `teammate_transcripts[sid]` first, fallback SwarmStatus | `teammate_view.rs` | Soft shows real stream when buffer present |
| 3.2 | Flip keys: Enter soft, Shift+Enter / `o` hard | `tui_state.rs` | Documented + hints updated |
| 3.3 | Input while soft: keep CommMessage DM; echo local user line into buffer | `input_dispatch.rs` | Typed line appears in soft transcript |
| 3.4 | Esc abort: prefer structured interrupt API if exists; else keep notify | server + client | Running agent stop turn, view stays |
| 3.5 | Auto-exit soft on killed/failed (CC `useTeammateViewAutoExit`) | `refresh_teammate_soft_view` | View closes on kill |
| 3.6 | Hard path unchanged as escape hatch | — | Full session still available |

**Exit criteria Phase 3:** User can Shift-free or ↓ select → Enter → see live agent text without session switch; Esc returns lead transcript instantly; Shift+Enter still hard.

---

### Phase 4 — Polish & parity extras (2–4 days)

| # | Task | Notes |
|---|------|-------|
| 4.1 | Footer-style pills (optional second surface) | CC `BackgroundTaskStatus` / footer tasks |
| 4.2 | `k` kill = `CommStop` reliable + UI feedback | Already partial |
| 4.3 | Per-agent color identity | Cycle palette → stable hash(name) |
| 4.4 | Tree preview lines (last 1–2 buffer lines under child) | Needs Phase 2 |
| 4.5 | Unify tree vs swarm strip (one primary live surface) | Product call |
| 4.6 | Click row to enter (if mouse path exists) | Optional |

---

### Phase 5 — Hardening (1.5–3 days, parallelizable)

| # | Task |
|---|------|
| 5.1 | State-machine unit tests (pure fn extract from `handle_agent_tree_navigation_key` if needed) |
| 5.2 | Protocol round-trip tests |
| 5.3 | Multi-agent stress (3 workers, soft switch thrash) |
| 5.4 | Resume race: hard attach during streaming |
| 5.5 | Install/serve version mismatch detection in TUI |

---

## 6. File map (where to work)

| Area | Path |
|------|------|
| Nav / enter / exit | `crates/jcode-tui/src/tui/app/tui_state.rs` |
| Keys remote | `crates/jcode-tui/src/tui/app/remote/key_handling.rs` |
| Soft input | `crates/jcode-tui/src/tui/app/remote/input_dispatch.rs` |
| Resume lifecycle | `crates/jcode-tui/src/tui/app/remote.rs` |
| Server events | `crates/jcode-tui/src/tui/app/remote/server_events.rs` |
| Soft render | `crates/jcode-tui/src/tui/teammate_view.rs` |
| Tree render | `crates/jcode-tui/src/tui/agent_tree.rs` |
| Layout chrome | `crates/jcode-tui/src/tui/ui.rs`, `ui_input.rs` |
| App fields | `crates/jcode-tui/src/tui/app.rs` |
| Wire | `crates/jcode-protocol/src/{lib,wire}.rs` |
| Tail + future stream | `crates/jcode-app-core/src/server/background_tasks.rs`, swarm broadcast |
| Resume API | `crates/jcode-tui/src/tui/backend.rs` (`resume_session`, `comm_message_dm`) |

---

## 7. Pseudocode — core flows

### 7.1 Select step (current + target)

```
FUNCTION step_agent_selection(app, dir):
  children = selectable_children(app.agent_trees())  // all builder children, dense indices
  if children empty and not viewing: return
  app.agent_tree_hidden = false
  max = viewing ? children.len-1 : children.len  // hide row only when not viewing
  if not selecting:
    selecting = true
    index = viewing ? index_of(viewing_sid) : -1  // park lead first (CC)
    return
  index = wrap(index + dir, -1..max)
```

### 7.2 Enter confirm (Phase 1 vs Phase 3)

```
// Phase 1 (now)
ON Enter while selecting:
  if index == lead: exit_or_resume_lead()
  if Shift: soft_preview(sid)   // honest non-real
  else: hard_attach(sid)        // real session

// Phase 3 (after protocol)
ON Enter while selecting:
  if index == lead: exit_or_resume_lead()
  if Shift or 'o': hard_attach(sid)
  else: soft_view_from_buffer(sid)  // real buffer
```

### 7.3 Hard attach

```
FUNCTION hard_attach(sid, label):
  leader = return_id OR remote_session_id OR session.id
  if no leader: notice error; return false
  if snapshot empty: snapshot = remote_swarm_members.clone()
  return_id = leader  // keep original across agent hops
  hard = true
  viewing_sid = sid
  selecting = true
  queue_resume(sid)
  return true
```

### 7.4 Soft from buffer (Phase 3)

```
FUNCTION soft_view_from_buffer(sid):
  msgs = teammate_transcripts.get(sid)
  if msgs empty: msgs = build_from_swarm_status(sid)  // fallback
  teammate_view_messages = msgs
  hard = false
  viewing_sid = sid
  // stay on lead remote_session_id
```

### 7.5 Server append (Phase 2)

```
ON worker TextDelta/ToolEvent for session S in swarm:
  if S not in swarm_members: return
  msg = summarize(event)  // cap size
  append_capped(member.transcript, msg)
  broadcast SwarmMemberMessage to clients watching swarm
```

---

## 8. Manual acceptance script (every milestone)

**Prep:** install release, kill old serve, start serve, open **new** client, confirm binary hash.

1. Spawn **one** long-running agent (30–60s work).  
2. Tree visible under input with `@name`.  
3. Scroll chat to bottom; press **↓** → selection appears (or Shift+↓).  
4. Select agent → **Enter** → **hard**: full child transcript loads.  
5. Chrome visible: header + team-lead row return path + Esc hint.  
6. **Esc** → back on team-lead; leader history restored.  
7. Repeat 3–4; **Shift+Enter** → soft preview labeled non-full.  
8. Soft: type short DM → agent receives (or clear error).  
9. Soft Esc → lead.  
10. Interrupt while hard-attached → still Esc home.  
11. Two agents: free switch agent A → B → lead without losing return.

**Phase 3 extra:** Enter soft shows live assistant text without `resume_session` (network: no session switch log).

---

## 9. Automated tests

### Unit (now / Phase 1)

- `agent_tree`: dense indices Idle+Running; viewing keeps tree; return hint on lead.  
- `teammate_view`: meta-spawn filter; chrome strings.  
- Pure nav: extract `step_selection` / `confirm_selection` if tests need isolation.

### Integration (Phase 2–3)

- Mock server emits `SwarmMemberMessage` → client buffer.  
- Soft display_messages reads buffer.  
- Hard attach mock resume queues correct sid + return.

### Property / fuzz (Phase 5)

- Index wrap never panics for child_count 0..N.  
- Message_id dedupe.

---

## 10. Benchmarks / budgets

| Metric | Baseline | Target | How |
|--------|----------|--------|-----|
| Soft switch latency (lead→view) | n/a | < 16 ms UI (1 frame) | Instant state swap; no network |
| Hard attach resume | wall clock | < 500 ms local serve p50 | Log resume start/end |
| SwarmMemberMessage size | — | ≤ 4 KiB/event | Cap in producer |
| Retained msgs / member | — | ≤ 200 | Ring buffer |
| SwarmStatus broadcast rate | existing | no >2× regression | Count events under load |
| Client RSS with 5 agents streaming | — | +< 30 MB | `ps` / jemalloc stats |

---

## 11. Rollout & flags

| Flag / behavior | Default | Notes |
|-----------------|---------|-------|
| Hard Enter | **on** until Phase 3 | Current |
| Soft real buffer | off until Phase 2 ships | Feature flag `teammate_transcript_stream` optional |
| Bare ↓ at bottom | on | Document; can disable if conflicts |
| Serve version mismatch notice | strengthen | Avoid stale UX reports |

Migration: no data migration; protocol additive. Old servers ignore new events → soft falls back to SwarmStatus.

---

## 12. Risks & mitigations

| Risk | Mitigation |
|------|------------|
| Stale binary testing | Phase 0 ops + hash in UI |
| Soft still empty after Phase 2 | Bootstrap GetTranscript + tail fallback |
| Wire bloat | Summarize tools; cap text |
| Hard attach races | Keep chrome until resume Ok/Err; snapshot |
| Key conflicts (history vs ↓) | Only bare ↓ when at bottom + empty input |
| Scope creep (pills/click) | Park in Phase 4 |

---

## 13. Success criteria

### MVP (end Phase 1) — “đừng bắt làm lại vì kẹt UX”

- [ ] Fresh install + serve always used for test  
- [ ] ↓ at bottom / Shift+↓ selects agents  
- [ ] Enter opens **real** child session  
- [ ] Esc / Enter team-lead always returns  
- [ ] Soft never claims full transcript  
- [ ] Unit tests for selection + chrome green  

### >90% CC model (end Phase 3)

- [ ] Soft Enter shows live agent messages without resume  
- [ ] Free switch soft lead↔agents without socket hop  
- [ ] Hard attach remains available (Shift+Enter / o)  
- [ ] Input routes to viewed agent  
- [ ] Esc abort / exit matches CC semantics for soft  
- [ ] Protocol tests + manual script §8  

### Stretch (Phase 4)

- [ ] Footer pills or equivalent  
- [ ] Kill / colors / preview lines  

---

## 14. Suggested execution order (calendar)

```
Week 1:
  Day 1–2   Phase 0 + Phase 1 (stabilize hard free-switch)
  Day 3–5   Phase 2 start (wire + server append + client buffer)

Week 2:
  Day 1–3   Phase 2 finish + bootstrap
  Day 4–5   Phase 3 soft flip + input/abort

Week 3:
  Day 1–3   Phase 4 polish (priority-ordered)
  Day 4–5   Phase 5 hardening + release notes
```

**2 engineers:** split Phase 2 (server wire vs client buffer) in parallel after Phase 1.

---

## 15. Out of scope (this plan)

- Making jcode teammates true in-process ALS like CC Node (would be a different architecture project).  
- Desktop/web UI for teammate view.  
- Replacing swarm multi-session model.  
- Full tool_result fidelity in soft buffer (summaries OK for v1).

---

## 16. One-page cheat sheet for implementers

1. **CC enter = swap `task.messages`.** jcode multi-session ⇒ hard resume is real today; soft needs Phase 2 stream.  
2. **Do not flip Enter to soft until buffer is real.**  
3. **Always ship install + restart serve before claiming UX fixed.**  
4. **Free switch = tree selection + return id + snapshot**, not more chrome spam.  
5. **Read CC files in §2.2 before each phase.**  
6. **Pass §8 manual script** before marking phase done.

---

## 17. References

- Local CC: `/tmp/feature-research/claude-code`  
- Upstream: https://github.com/claude-code-best/claude-code  
- jcode swarm: `docs/SWARM_ARCHITECTURE.md`  
- Prior gap notes: `docs/AGENT_TREE_CC_GAP_ANALYSIS.md` (status table outdated; use this plan)  
- Branch work: `feat/agent-tree` commits through `bfac54414`
