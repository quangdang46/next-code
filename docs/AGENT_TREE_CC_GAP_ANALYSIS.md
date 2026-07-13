# Agent Tree / Subagent View: Claude Code vs jcode — Gap Analysis

**Date:** 2026-07-13  
**Branch:** `feat/agent-tree`  
**CC clone:** `/tmp/feature-research/claude-code` (`claude-code-best/claude-code`)  
**User report:** Tree UI ok-ish, but **cannot truly jump into subagent session**.

---

## 1. What Claude Code actually does (source of truth)

### 1.1 Data model

| Concept | Implementation |
|---------|----------------|
| Teammates | `AppState.tasks[id]: InProcessTeammateTaskState` |
| Running list | `getRunningTeammatesSorted(tasks)` — only `status === 'running'` |
| View target | `viewingAgentTaskId?: string` |
| Selection mode | `viewSelectionMode: 'none' \| 'selecting-agent' \| 'viewing-agent'` |
| Selection index | `selectedIPAgentIndex: -1` (leader) / `0..n-1` (teammates) / `n` (hide row) |
| Per-teammate transcript | `task.messages: Message[]` (UI mirror, capped) |
| Live append | `appendTeammateMessage` / stream in `inProcessRunner.ts` |
| Identity | `task.identity.agentName`, `.color`, `.agentId` |

Key files:

- `src/state/teammateViewHelpers.ts` — `enterTeammateView` / `exitTeammateView`
- `src/state/selectors.ts` — `getViewedTeammateTask`, `getActiveAgentForInput`
- `src/hooks/useBackgroundTaskNavigation.ts` — Shift+↑/↓, Enter, Esc, f, k
- `src/hooks/useTeammateViewAutoExit.ts` — eject on kill/fail/evict
- `src/components/Spinner/TeammateSpinnerTree.tsx` + `TeammateSpinnerLine.tsx`
- `src/components/TeammateViewHeader.tsx`
- `src/screens/REPL.tsx` ~5525–5612, ~5897–5926, ~4470 — **message swap + input inject**
- `src/tasks/InProcessTeammateTask/*` — task state + inject
- `src/utils/swarm/inProcessRunner.ts` — mirrors stream into `task.messages`

### 1.2 Enter teammate view (the real “jump in”)

```text
enterTeammateView(taskId):
  viewingAgentTaskId = taskId
  viewSelectionMode = 'viewing-agent'
  (local_agent: retain=true so messages not evicted)
```

Then in **REPL** (critical):

```tsx
// displayedMessages SWITCHES entirely to the agent's message array
const displayedMessages = viewedAgentTask
  ? (viewedAgentTask.messages ?? [])   // empty until bootstrap/stream
  : leaderMessages;

// Messages component renders THAT array as the full transcript
<Messages messages={displayedMessages} ... />

// Header above transcript
<TeammateViewHeader />  // "Viewing @name · esc return" + task prompt

// Input routing
if (viewing teammate) injectUserMessageToTeammate(taskId, input)
else send to leader
```

So **“enter session” in CC is not a popover**. It is:

1. **Full transcript takeover** of the main scroll area  
2. **Live message stream** of that teammate (`task.messages`)  
3. **Input redirected** into that teammate  
4. **Spinner/tree still visible** with leader backgrounded  
5. **Esc** → `exitTeammateView` → leader transcript again  

In-process: same Node process, AsyncLocalStorage isolation — no socket session switch.

### 1.3 Keyboard / UX contract (CC)

| Input | Effect |
|-------|--------|
| `Shift+↓/↑` | Enter selecting mode; step leader ↔ teammates ↔ hide |
| `Enter` on teammate | `enterTeammateView` → full transcript of that agent |
| `Enter` on leader | Exit view → leader |
| `Enter` on hide | Collapse tree (`expandedView = none`) |
| `Esc` selecting | Exit selection only |
| `Esc` viewing + running | Abort **current turn** of teammate (not kill) |
| `Esc` viewing + terminal | Exit view |
| `f` selecting | Same as Enter (view) |
| `k` selecting | Kill selected running teammate |
| Type while viewing | Message goes to **that** teammate |
| Tree only if | `getRunningTeammatesSorted.length > 0` |
| Alt surface | Footer **pills** when tree mode off |

### 1.4 Placement (CC)

```
[ full transcript: leader OR viewed agent messages ]
[ Spinner + TeammateSpinnerTree ]   ← above input when active
[ PromptInput ]                      ← colored when viewing teammate
[ Footer: mode / pills / hints ]     ← below input
```

---

## 2. What jcode does today

### 2.1 Data model

| Concept | Implementation |
|---------|----------------|
| Swarm members | `remote_swarm_members: Vec<SwarmMemberStatus>` (server snapshot) |
| Subagent status string | `subagent_status: Option<String>` (tool bus, rarely named) |
| Explicit trees | `agent_trees: Vec<AgentTreeNode>` (almost unused) |
| “Viewing” flag | `viewing_teammate_session_id: Option<String>` |
| Selecting | `agent_tree_selecting` + `selected_agent_tree_index` |
| Per-agent messages | **None on client** — only `output_tail` / detail on `SwarmMemberStatus` |
| Hard session switch | `RemoteConnection::resume_session` / `queue_resume_session` (exists, **not wired to tree Enter**) |

### 2.2 Enter “view” (current)

```rust
// handle_agent_tree_navigation_key — Enter on child:
self.viewing_teammate_session_id = Some(sid);
self.view_teammate_selection = true;
self.set_status_notice("Viewing → @label (Esc to exit)");
// NO resume_session
// NO message swap
// NO input reroute
```

Render:

```rust
// ui.rs — small floating panel (~60×8) corner overlay
if viewing_teammate_session_id.is_some() {
    draw_teammate_view_panel(...);  // status + detail + session id + Esc hint
}
```

`draw_teammate_view_panel` reads **running_items** by session id — if member not in running_items list, panel **returns early (blank / nothing)**. Content is static-ish status lines, **not a transcript**.

### 2.3 What user experiences

| Action | Result |
|--------|--------|
| See tree | Yes (after chrome/glyph fixes) |
| Shift+↑/↓ | Selection moves (if keys reach handler) |
| Enter | Status notice + tiny overlay **or nothing useful** |
| See subagent chat history | **No** |
| Type to subagent | **No** — still goes to leader/coordinator |
| Live stream of subagent tools | **No** in view mode |
| Esc | Clears viewing flag |

**Conclusion:** User is correct — jcode has **not** implemented real “jump into session”. Only a flag + stub panel.

---

## 3. Gap matrix (priority)

| # | Gap | Claude Code | jcode now | Severity | Effort |
|---|-----|-------------|-----------|----------|--------|
| **G1** | **Transcript takeover** | Main `Messages` shows `task.messages` | Leader transcript unchanged | **P0** | L |
| **G2** | **Live message mirror** | Stream → `task.messages` every tool/assistant chunk | Only `output_tail` string on SwarmStatus | **P0** | L (server+protocol) |
| **G3** | **Input routing** | Type → inject into viewed teammate | Type → leader only | **P0** | M |
| **G4** | **View header** | Full-width “Viewing @name · esc” + prompt | Tiny corner panel | **P0** | S |
| **G5** | **Enter does real enter** | Sets view + retains messages | Sets id only | **P0** | M |
| **G6** | **Hard attach path** | N/A (in-process) | `resume_session(sid)` exists but unused by tree | **P0 alt** | M |
| **G7** | Panel data source | `task.messages` | `running_items` match — often **miss** for swarm-only members | **P0** | S |
| **G8** | Esc while viewing running | Abort teammate turn | Just exit flag | P1 | M |
| **G9** | Auto-exit on kill/fail | `useTeammateViewAutoExit` | Clear only on interrupt | P1 | S |
| **G10** | Kill selected (`k`) | Yes | No | P1 | M |
| **G11** | Hide tree row | Yes | No | P2 | S |
| **G12** | Footer pills mode | Alt surface | Swarm strip / running_items partial | P2 | M |
| **G13** | Tool uses · tokens on line | From `task.progress` | Always 0 | P2 | M |
| **G14** | Click row to enter | Ink Box click | No | P2 | M |
| **G15** | Preview lines under child | Last 3 message lines | No | P2 | S after G2 |
| **G16** | Color identity per agent | `identity.color` | Cycle palette only | P2 | S |
| **G17** | Tree vs swarm strip triple UI | One primary live surface | Tree + strip + cards | P2 product | — |

---

## 4. Architecture fork (decide before coding)

Claude Code teammates are **in-process**. jcode swarm members are usually **separate remote sessions** (`session_id` on server). Two valid product modes:

### Option A — Soft view (closest CC UX, keep coordinator session)

```
Enter child:
  viewing_teammate_session_id = sid
  display_messages ← reconstructed from:
    - SwarmMemberStatus.output_tail (now)
    - NEW: ServerEvent::SwarmMemberTranscript / message stream (needed)
  header: Viewing @duck
  input → remote soft_interrupt / swarm dm / inject to that session
Esc → back to coordinator transcript (cached leader messages)
```

**Pros:** Matches CC mental model; stay on coordinator session.  
**Cons:** Needs richer server events than `output_tail`; input inject protocol.

### Option B — Hard attach (true session jump)

```
Enter child:
  stash coordinator remote_session_id
  remote.resume_session(child_sid)
  full TUI is now that agent (real history + tools + input)
Esc / “return to team-lead”:
  resume_session(coordinator_sid)
```

**Pros:** Real session, real history, already have `resume_session`.  
**Cons:** Leaves coordinator context; reconnect cost; less like CC “zoom” and more like session switcher.

### Option C — Hybrid (recommended)

```
Enter (default): soft view with full-height transcript pane from
  output_tail + future message events + DM inject
Enter+modifier or swarm panel `o`: hard resume_session (pop out)
```

Aligns with existing notice: swarm panel `o pop out`.

---

## 5. Minimum viable “really enter” (recommended sprint)

### Phase 1 — Make Enter feel real (1–2 days)

1. **Wire Enter → full-height view layer**, not 8-line overlay:
   - Replace main messages area content when `viewing_teammate_session_id` set.
   - Header bar: `Viewing @name · esc return` + prompt/task_label.
2. **Data for transcript now** (no new protocol yet):
   - Pull `output_tail`, `detail`, `task_label`, `todo_items` from matching `remote_swarm_members` entry (not only running_items).
   - Render as pseudo-transcript lines (assistant stream tail + status).
3. **Fix data miss (G7):**
   - Resolve member from `remote_swarm_members` by `session_id` first.
4. **Input while viewing (partial):**
   - Route typed submit → `swarm` dm / existing soft path to `to_session = viewing_sid`  
   - Or status_notice “view-only until inject wired” if no API — but prefer real dm.

### Phase 2 — Protocol parity with CC messages (2–4 days)

1. Server publishes per-member message events (or thicker tails):
   - user/assistant/tool_use summaries into client buffer `HashMap<session_id, Vec<DisplayMessage>>`.
2. Soft view uses that buffer like `task.messages`.
3. Live append on SwarmStatus / stream events.

### Phase 3 — Hard attach + polish

1. Tree Enter = soft view; `o` / `Enter+Shift` = `resume_session`.
2. Esc abort turn when viewing running (G8).
3. Auto-exit (G9), kill `k` (G10), hide row (G11).

---

## 6. Concrete jcode code map (implement here)

| Change | File(s) |
|--------|---------|
| Enter must not only set flag | `app/tui_state.rs` `handle_agent_tree_navigation_key` |
| Resolve member from swarm list | `agent_tree` helpers / `tui_state` |
| Full-height view instead of corner panel | `ui.rs` messages area + `ui_overlays.rs` rewrite or delete stub |
| Header | new `teammate_view_header` draw in `ui.rs` |
| Input routing | `remote/key_handling.rs` submit path, `input_dispatch.rs` |
| Optional hard attach | call `remote.resume_session(sid)` + stash leader id on App |
| Clear view on interrupt | already partial in `server_events` / `turn.rs` |

Existing assets to reuse:

- `RemoteConnection::resume_session` (`backend.rs`)
- `queue_resume_session` (`workspace_client.rs`)
- Swarm panel focus `Alt+W` / `o pop out` (hard attach pattern)
- `SwarmMemberStatus.output_tail`, `todo_items`, `task_label`

---

## 7. Test plan (after implement)

1. Spawn `@duck` with long prompt; tree shows under input.  
2. `Shift+↓` → `>` on duck → `Enter`.  
3. **Expect:** main pane says `Viewing @duck`, shows stream/tail, **not** only a tiny box; leader chat hidden.  
4. Type `ping` + Enter → appears on duck (or dm) not on leader.  
5. `Esc` → leader transcript restored.  
6. Optional: hard attach resumes child session fully; return restores leader.  
7. Interrupt leader → tree + view clear.

---

## 8. One-paragraph summary for implementers

Claude Code “enter subagent” is a **full transcript + input context switch** onto an in-process task’s `messages` array (`enterTeammateView` → `displayedMessages = task.messages` + `injectUserMessageToTeammate`). jcode only sets `viewing_teammate_session_id` and draws a **stub corner panel** from `running_items`, without swapping the main transcript, without a per-agent message buffer, and without routing input — so users correctly feel they never “jumped in.” Closing the gap requires either soft-view (swap display + stream tails + inject) or hard-attach (`resume_session`), ideally hybrid: soft view on Enter, hard pop-out on existing swarm `o` path.

---

## 9. CC file checklist (re-read before coding)

```
/tmp/feature-research/claude-code/src/state/teammateViewHelpers.ts
/tmp/feature-research/claude-code/src/state/selectors.ts
/tmp/feature-research/claude-code/src/hooks/useBackgroundTaskNavigation.ts
/tmp/feature-research/claude-code/src/hooks/useTeammateViewAutoExit.ts
/tmp/feature-research/claude-code/src/components/TeammateViewHeader.tsx
/tmp/feature-research/claude-code/src/components/Spinner/TeammateSpinnerTree.tsx
/tmp/feature-research/claude-code/src/components/Spinner/TeammateSpinnerLine.tsx
/tmp/feature-research/claude-code/src/screens/REPL.tsx  (displayedMessages, inject, header)
/tmp/feature-research/claude-code/src/tasks/InProcessTeammateTask/InProcessTeammateTask.tsx
/tmp/feature-research/claude-code/src/utils/swarm/inProcessRunner.ts
```
