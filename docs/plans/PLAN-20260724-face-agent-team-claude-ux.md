# PLAN-20260724 — Face agent team UX (Claude Code–first)

> **Status:** Implemented (Face UI + ACP glue) — see §12 for shipped vs deferred.  
> **Implementation branch:** `pr-face-agent-team-claude-ux`  
> **Plan / research PR:** #85 (`docs/agent-team-claude-ux-plan`)  
> **Primary target UX:** Claude Code **agent teams + in-session agent panel** (not Cursor Agents Window / `/multitask`).  
> **Secondary reference:** Grok Build / Face subagent surfaces (tasks pane, fullscreen child, worktree isolation).  
> **Related (do not collide):** parallel multitask / ask-user plans use different filenames (`PLAN-20260724-face-multitask-mvp.md`, `PLAN-20260724-face-ask-user-multiquestion.md`).  
> **Prior TUI work:** `docs/plans/agent-tree-cc-parity.md`, `docs/AGENT_TREE_CC_GAP_ANALYSIS.md` (legacy TUI; Face gap is larger).

---

## 0. One-line verdict

next-code already has **swarm coordination (server + legacy TUI agent tree)** and Face already has **Grok-style subagent lifecycle UI** (scrollback block → fullscreen child, Ctrl+B tasks pane, SwarmStatus float). What’s missing for Claude-like “agent team” UX on Face is a **lead-centered agent panel + select/enter/message/kill + shared task strip**, wired to next-code swarm members — not a Cursor multitask clone and not a second dashboard of independent sessions.

---

## 1. LOOK — Claude Code (primary)

### 1.1 Official product surfaces (docs)

| Surface | URL | Role |
|--------|-----|------|
| Agent teams | https://code.claude.com/docs/en/agent-teams | Lead + teammates + shared task list + mailbox; experimental (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`) |
| Subagents | https://code.claude.com/docs/en/sub-agents | Same-session delegated workers; summary back to caller; custom `.claude/agents/*.md` |
| Parallel agents overview | https://code.claude.com/docs/en/agents | Compares subagents vs **agent view** (`claude agents`) vs **agent teams** vs workflows |
| Worktrees | https://code.claude.com/docs/en/worktrees | Isolation for parallel edits; **agent teams do not auto-worktree teammates** — partition files instead |

**Fetched evidence (2026-07-24 via Exa):**

- **Subagents vs teams:** subagents = own context, report only to caller, lower cost; teams = independent sessions, **peer messaging**, shared tasks, higher cost.
- **Spawn:** natural language to the lead; first teammate spawn forms the team (post–v2.1.178: no `TeamCreate` / `TeamDelete`).
- **UI — in-process (default):** agent panel under the prompt; ↑/↓ select; **Enter** open teammate transcript + type to message; **Esc** interrupt selected turn; **Ctrl+T** task list; **x** stop selected teammate.
- **UI — split panes (optional):** tmux / iTerm2 panes for simultaneous visible transcripts (`teammateMode`: `in-process` | `auto` | `tmux`).
- **Idle row UX (v2.1.199+):** idle rows stay while any agent works; hide after 30s when all idle; collapse many idle into `N idle agents`.
- **Same panel for both:** subagents and teammates share the agent panel — panel presence ≠ team formed.
- **Permissions:** teammates inherit lead permission mode; background subagents can surface prompts on the main session (changelog / DeepWiki notes); swarm path has leader-forwarded permission sync in source (`permissionSync.ts`).
- **Isolation:** teams → partition files; subagents / agent-view sessions → optional worktrees. Do **not** require worktrees for MVP team UX.

**Differentiate from agent view:** `claude agents` is a **research-preview dispatcher** for independent background sessions (Needs input / Working / Completed). That is closer to Cursor multitask / a session dashboard. **This plan targets agent-team + in-session panel UX, not cloning agent view.**

### 1.2 Local research clone (`.tmp-research-plugins/claude-code/`)

Key paths (swarm / teams implementation):

| Concern | Path |
|--------|------|
| In-process spawn | `src/utils/swarm/spawnInProcess.ts` |
| Stream → `task.messages` | `src/utils/swarm/inProcessRunner.ts` |
| Pane backends (tmux/iTerm/WT) | `src/utils/swarm/backends/*`, `teammateLayoutManager.ts` |
| Shared tasks | `src/utils/tasks.ts` (team-scoped task list) |
| Mailbox / idle | `src/utils/swarm/teammateInit.ts`, `teammateMailbox` usage |
| Permission forward to lead | `src/utils/swarm/permissionSync.ts`, `leaderPermissionBridge.ts` |
| Lifecycle tests | `src/utils/swarm/__tests__/agentTeamsLifecycle.test.ts` |

**Mental model (also documented in `docs/AGENT_TREE_CC_GAP_ANALYSIS.md` / `docs/plans/agent-tree-cc-parity.md`):**

```text
spawn → AppState.tasks[id] (in_process_teammate) with messages[]
Shift+↑/↓ → select in spinner/tree (agent panel)
Enter → enterTeammateView: displayedMessages = task.messages (full takeover)
type → inject into teammate (not lead)
Esc → abort turn or exit view (context-dependent)
```

Enter is an **in-process view switch**, not a socket “resume another session” (though next-code historically used hard-attach resume for remote swarm members).

### 1.3 Claude UX patterns to copy (Face MVP checklist)

1. **Agent panel** under (or docked beside) the lead prompt — roster of workers with status bullets / activity.
2. **Keyboard: select → Enter transcript → type to that agent → Esc back / interrupt.**
3. **Shared task list** toggle (CC: Ctrl+T) — pending / in progress / completed + dependencies.
4. **Lead synthesizes**; user can still **DM a worker** without leaving the mental “team”.
5. **Kill / stop** on selected worker from the panel.
6. **Honest chrome:** “Viewing @name · esc return” (parity with TUI plan).
7. **Defer:** split-pane tmux layout, `claude agents` dashboard, automatic worktrees for every teammate.

---

## 2. LOOK — Grok Build / Face (secondary)

### 2.1 Docs (vendored pager)

Primary: `crates/xai-grok-pager/docs/user-guide/16-subagents.md`

| Topic | Grok behavior |
|------|----------------|
| Spawn | Parent calls `spawn_subagent` (`prompt`, `description`, `subagent_type`, `background`, `capability_mode`, `isolation`, `resume_from`, `cwd`) |
| Types | `general-purpose`, `explore`, `plan` (+ custom agents / personas) |
| Depth | **Flat:** children cannot spawn children |
| Isolation | `none` (default) or `worktree` via `x.ai/git/worktree/*` |
| Tasks pane | **Ctrl+B** — subagents + background commands |
| Scrollback | Compact lifecycle block; Enter opens **fullscreen framed child transcript** |
| Child input | Largely observational (not full interactive peer like CC teammates) |
| Agents modal | `/config-agents` / `/personas` — library, not live team roster |

### 2.2 Face / pager implementation seams (already in tree)

| Concern | Path |
|--------|------|
| `SubagentInfo` + enrich | `crates/xai-grok-pager/src/app/subagent.rs` |
| Spawn/progress ACP fold | `crates/xai-grok-pager/src/app/acp_handler/session_notification.rs` (`SubagentSpawned` → `subagent_sessions` + `subagent_views`) |
| Child activity | `crates/xai-grok-pager/src/app/acp_handler/subagent_activity.rs` |
| Tasks pane | `crates/xai-grok-pager/src/views/tasks_pane.rs` (Ctrl+B) |
| SwarmStatus float | `docs/plans/PLAN-20260721-face-info-widget-floats.md` — wired from `subagent_sessions` |
| TeamView float | **stub only** (`legacy_deferred` / `has_data_for(TeamView) => false`) |
| `active_subagent` fullscreen | `AgentView` fields in `crates/xai-grok-pager/src/app/agent_view/mod.rs` |

**Grok vs Claude (UX):**

| | Claude agent team | Grok / Face subagent |
|--|-------------------|----------------------|
| Coordination | Lead + shared tasks + peer mailbox | Parent delegates; result summary; no peer team mailbox |
| Panel | Always-on agent panel under prompt | Tasks pane + scrollback chip + optional SwarmStatus float |
| Enter child | Interactive teammate session | Fullscreen observational child (limited prompt) |
| Nesting | Teams of peers; subagents can nest by product rules | Depth 1 only |
| Isolation | Teams: partition files; subagents: optional WT | Explicit `isolation: worktree` |

Face should **reuse Grok’s child transcript frame + tasks pane machinery**, but **reshape interaction toward Claude’s select/enter/message panel** when the backend is a next-code swarm/team.

---

## 3. LOOK — next-code today

### 3.1 Swarm backend (strong; Face UI weak)

| Doc / area | What exists |
|-----------|-------------|
| `docs/SWARM_ARCHITECTURE.md` | Coordinator, recursive spawn tree, DMs, broadcasts, lifecycle, optional worktrees, completion report-back |
| `docs/SWARM_TASK_GRAPH.md` | DAG / task-graph evolution of swarm plan |
| Effort modes | `swarm` (light) / `swarm-deep` in provider + prompt tests |
| Legacy TUI | Agent tree + soft/hard teammate view (`agent-tree-cc-parity.md` ~75–85% CC nav) |
| Face | SwarmStatus float from **Grok `subagent_sessions`**, not full swarm roster/DM/task UX; TeamView stub |
| Hooks | `SubagentStart` / `SubagentStop` configured but “not yet” at swarm spawn sites (`docs/HOOKS.md`) |

### 3.2 Architecture mismatch to design for

| | Claude in-process teammate | next-code swarm member | Grok Face subagent |
|--|---------------------------|------------------------|--------------------|
| Process | Same Node process, ALS | Separate remote session (daemon) | Child ACP session under parent |
| Transcript | `task.messages` local mirror | Need soft stream or hard `resume_session` | `subagent_views[child]` AgentView |
| User message | Inject into teammate task | DM / `notify_session` / hard attach | Limited / observational |

**Face strategy (recommended):** treat Claude UX as the **shell**; prefer Grok’s **child AgentView** buffer when available; for swarm members without Grok spawn events, reuse TUI soft-buffer / hard-attach patterns over ACP (`SwarmMemberMessage` / resume) — do not pretend all workers are in-process.

### 3.3 Prior multitask research (scope fence)

Prior conclusion (swarm light exists; no Agents Window clone) still holds:

- **In scope here:** Claude-like **in-session team panel + lead/worker + tasks**.
- **Out of scope:** Cursor `/multitask`, full `claude agents` multi-root session dispatcher, desktop multi-window session manager.

---

## 4. Compare matrix

| Surface | Claude Code | Grok Face | next-code Face / TUI |
|--------|-------------|-----------|----------------------|
| **Spawn** | NL → lead; Agent tool teammates / subagents; flag for teams | `spawn_subagent` tool; types + personas | Swarm tool / effort `swarm*`; Face shows Grok spawn notifs if agent emits them |
| **Team view** | Agent panel under prompt; optional tmux panes | Ctrl+B tasks pane + SwarmStatus float; TeamView stub | Legacy TUI agent tree; Face: no CC panel |
| **Per-agent transcript** | Enter → full message swap (`task.messages`) | Fullscreen framed child | TUI soft/hard attach; Face child view for Grok subagents only |
| **Lead vs worker** | Explicit lead session; workers messageable | Parent vs child; child mostly observe | Swarm coordinator + `report_back_to_session_id`; Face chrome incomplete |
| **Permissions** | Inherit lead mode; swarm forward to lead UI; bg prompts on main | `capability_mode` + parent permission path | Permission plans mention subagent provenance; Face confirm wire separate |
| **Worktree / isolation** | Teams: partition files; subagents/agent-view: WT optional | `isolation: worktree` first-class | Swarm optional WT + managers; Face worktree apply via Grok extensions when used |
| **Shared tasks** | Team task list (Ctrl+T) | Todo pane (Ctrl+T) separate from subagents | Swarm plan / task graph server-side; Face Plan/todos via ACP, not team claim UI |
| **Peer comms** | Mailbox / SendMessage | Parent←child summary | DMs / broadcasts in swarm protocol; Face not surfaced as team chat |

---

## 5. Gap vs next-code Face (Claude-first)

| Gap ID | Gap | Severity |
|--------|-----|----------|
| G1 | No Claude-style **agent panel** (select/enter/kill) on Face | P0 |
| G2 | Cannot **message a worker** from Face with CC affordances (only Grok observe / TUI DM) | P0 |
| G3 | No **shared team task strip** tied to swarm plan / claim states | P1 |
| G4 | Swarm roster not unified with Grok `subagent_sessions` in one mental model | P1 |
| G5 | Permission prompts from workers not clearly attributed + lead-forwarded like CC | P1 |
| G6 | TeamView float stub; idle collapse / color identity polish missing | P2 |
| G7 | Split-pane / tmux teammate layout | P3 / defer |
| G8 | True multi-root / agent-view dispatcher | Non-goal |

---

## 6. Target UX (Claude-first)

### 6.1 Primary composition (MVP)

```text
┌─────────────────────────────────────────────┐
│ Lead transcript (default)                   │
│  … tool / assistant …                       │
│  [Subagent/Teammate chip…]  ← optional      │
├─────────────────────────────────────────────┤
│ Agent panel: ● lead · ○ workerA · ○ workerB │  ← ↑/↓ select
│ Tasks: 2/5 in progress (toggle)             │  ← Claude Ctrl+T analogue
├─────────────────────────────────────────────┤
│ Prompt (routes to lead OR selected worker)  │
└─────────────────────────────────────────────┘

Enter on worker → transcript takeover (child AgentView or soft buffer)
Header: Viewing @workerA · esc return
Esc → back to lead (or interrupt if mid-turn, match CC rules)
```

### 6.2 Spawn flow

1. User enables swarm / team mode (existing effort or explicit setting — **one gate**, no dual “legacy team” flags).
2. Lead spawns workers (swarm tool or Grok `spawn_subagent` depending on agent stack).
3. Panel rows appear with status + activity; idle collapse rules follow Claude 2.1.199+ spirit.
4. Completion → report-back into lead (already swarm policy) + panel status ✓/✗.

### 6.3 Aggregate

- Lead remains primary user chat.
- Panel shows count working / needs input / failed.
- Optional SwarmStatus float remains scroll-glance only; **panel is the interactive surface**.

---

## 7. Phased plan

### Phase 0 — Spec freeze (0.5–1 d)

- [ ] Map CC keybindings → Face actions table (document deviations explicitly).
- [ ] Decide single roster model: union of swarm members + Grok `subagent_sessions` with source tag.
- [ ] Confirm non-goals with stakeholders (no Agents Window, no tmux MVP).

### Phase 1 — MVP Face agent panel (3–6 d)

- [ ] Render agent panel from unified roster under prompt (reuse tasks-pane row widgets where possible).
- [ ] Selection + Enter → open child transcript (`active_subagent` path for Grok; soft buffer / hard attach for swarm).
- [ ] Esc return chrome; kill/stop selected.
- [ ] Smoke: spawn 2 workers, switch, return, kill one.

### Phase 2 — Message + permissions (2–4 d)

- [ ] Typing while viewing worker → DM / notify / inject (match TUI soft-view behavior).
- [ ] Permission dialog shows `subagent: Name` provenance; optional lead-forward for swarm (CC `permissionSync` inspiration).
- [ ] Wire `SubagentStart`/`Stop` hooks at spawn sites if still missing.

### Phase 3 — Shared tasks + polish (3–5 d)

- [ ] Team task strip bound to swarm plan / task-graph nodes (claim / pending / done).
- [ ] Idle hide/collapse; stable color-by-name; TeamView float either deleted or powered by same roster.
- [ ] Docs: user-facing “Agent teams on Face” note; update gap analysis.

### Phase 4 — Optional later

- [ ] Split-pane / external terminal backends (CC tmux path).
- [ ] Deeper soft-stream fidelity (`SwarmMemberMessage` → Face) without hard attach.
- [ ] Worktree apply UX for Grok-isolated children (already partially documented).

---

## 8. Non-goals

- Do **not** clone Cursor Agents Window / `/multitask` as the primary UX.
- Do **not** build `claude agents` multi-root dispatcher in this track.
- Do **not** require worktrees for every teammate (Claude teams don’t).
- Do **not** make Face depend on legacy TUI binary (`NEXT_CODE_LEGACY_TUI`); port patterns, don’t revive UI.
- Do **not** invent a second spawn tool if `swarm` / `spawn_subagent` already cover the agent stack — unify presentation first.

---

## 9. Files likely to touch (implementation PRs; not this docs PR)

| Area | Likely paths |
|------|----------------|
| Panel UI | `crates/xai-grok-pager/src/views/tasks_pane.rs`, new `views/agent_panel.rs`, `app/agent_view/*` |
| ACP / roster | `app/acp_handler/session_notification.rs`, `app/subagent.rs`, pager_agent swarm event emit |
| Actions / keys | `crates/xai-grok-pager/src/actions/defaults.rs`, `app/actions.rs`, `app/dispatch/*` |
| Permissions | Face permission confirm + `docs/plans/permission-improvement.md` worker fields |
| Swarm protocol | `crates/next-code-protocol`, `next-code-app-core` swarm status / member message (if soft stream) |
| Docs | this plan; later user-guide; retire or narrow `TeamView` stub notes |

---

## 10. Smoke (when implementing)

1. Rebuild Face + **restart** `next-code serve`.
2. Lead session with swarm/subagents enabled; ask to spawn 2 independent workers.
3. Panel lists both with live activity; ↑/↓ select; Enter opens transcript; Esc returns to lead.
4. Message selected worker; confirm lead still receives completion report.
5. Stop one worker from panel; row shows cancelled/failed; no stuck “viewing” state.
6. Toggle task strip; statuses move pending → in progress → completed.
7. Regression: Ctrl+B tasks pane still lists background shell tasks; Grok-only subagent fullscreen still works.

---

## 11. References (bookmark)

**Claude**

- https://code.claude.com/docs/en/agent-teams  
- https://code.claude.com/docs/en/sub-agents  
- https://code.claude.com/docs/en/agents  
- Local: `.tmp-research-plugins/claude-code/src/utils/swarm/`  

**Grok / Face**

- `crates/xai-grok-pager/docs/user-guide/16-subagents.md`  
- `crates/xai-grok-pager/src/app/subagent.rs`  

**next-code**

- `docs/SWARM_ARCHITECTURE.md`, `docs/SWARM_TASK_GRAPH.md`  
- `docs/plans/agent-tree-cc-parity.md`, `docs/AGENT_TREE_CC_GAP_ANALYSIS.md`  
- `docs/plans/PLAN-20260721-face-info-widget-floats.md`  

**DeepWiki note:** `ask_question` on `anthropics/claude-code` conflated **agent view** (`claude agents`) with **in-session agent panel**; prefer official agent-teams doc + local swarm sources above for UI specifics.

---

## 12. Implementation status (2026-07-24)

**Verdict:** Face Claude-style agent-team shell is implemented on `pr-face-agent-team-claude-ux`. Plan research PR: #85.

### Keybindings (Face deviations)

| Claude Code | Face |
|-------------|------|
| Up/Down select roster | **Shift+Up/Down** (bare arrows remain scrollback) |
| Ctrl+T tasks | **Ctrl+Shift+T** (Ctrl+T stays todo pane) |
| Enter / Esc / x | Same while panel selecting |
| Shift+Enter claim | Claim selected team task then SendPrompt (task strip open; no roster selecting required) |
| (none) | **Shift+Left/Right** select team task |

### Shipped

- Unified roster: lead + Grok `subagent_sessions` + live swarm mirrors (`agent_roster.rs`)
- Under-prompt agent panel + status chips + idle collapse (`views/agent_panel.rs`)
- Enter opens Grok fullscreen (interactive prompt) or swarm soft transcript
- Esc clears selection / exits soft / exits fullscreen
- x kills Grok subagent; swarm stop via daemon `CommStop` (`x.ai/swarm/stop`)
- Message routing: Grok child via ACP `SendPrompt`; swarm DM via `x.ai/swarm/dm` (CommMessage, NotifySession fallback) + soft buffer echo
- Live ACP: `x.ai/swarm/status` / `member_message` / `plan` from pager idle pump + prompt loop
- Team task strip + claim (Shift+Left/Right; Shift+Enter without roster selecting; todo seed fallback)
- Permission provenance label includes swarm teammates (`Teammate "name":`)

### Deferred (hard blockers / non-goals)

- tmux / split-pane teammate layout (Claude click-into-pane path)
- True multi-root / Agents Window / Cursor `/multitask`
- TeamView float stub left as-is (panel is the interactive surface)
- `SubagentStart`/`Stop` hook wiring at swarm spawn sites
- Mouse click-to-enter on in-process panel rows (Claude in-process uses Enter)

### Verify

- Unit: `cargo test -p xai-grok-pager --lib agent_roster` / `agent_panel`
- Check: `cargo check -p xai-grok-pager -p next-code --bins`

