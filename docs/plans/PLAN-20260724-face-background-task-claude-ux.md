# Plan ΓÇö Face background-task UI/UX ΓåÆ Claude Code parity

**Status:** Implemented (Face chrome) — deferred items listed at end
**Date:** 2026-07-24  
**Branch:** `docs/background-task-claude-ux-plan`  
**Scope:** Background-task **chrome** ΓÇö list/panel, badges, expand output, cancel/stop, completion cues, non-blocking chat.  
**Not in scope:** Agent-team / teammate swarm UX ┬╖ Cursor `/multitask` ┬╖ AskUserQuestion multi-prompt ┬╖ full Agent View (`claude agents`) session fleet.

Cross-links (sibling plans; do not collide filenames):

| Plan | Focus |
|------|--------|
| `PLAN-20260724-face-agent-team-claude-ux.md` | Agent teams / teammates (separate) |
| `PLAN-20260724-face-multitask-mvp.md` | Cursor-style multitask (separate) |
| `PLAN-20260724-face-ask-user-multiquestion.md` | AskUserQuestion wire (separate) |
| `PLAN-20260721-face-info-widget-floats.md` | Info floats incl. BackgroundTasks card |
| `PLAN-20260721-face-status-chrome.md` | Welcome / status chrome (adjacent) |
| `crates/xai-grok-pager/docs/user-guide/20-background-tasks.md` | Current Face/Grok bg-task product docs |

---

## Summary

Claude Code treats background work as a **first-class, always-visible layer**: footer pill + `/tasks` dialog + demote-with-`Ctrl+B` + transcript chips, while the prompt stays free. Face/Grok already has a strong backend (ACP `task_backgrounded` / `task_completed`, `TasksPane`, watching line, BgTask scrollback, kill, live stdout store) but the **presentation grammar** still reads more Grok-native than Claude: no Claude-style footer pill CTA, `/tasks` is a dump not a dialog, info-float is count-only, and key chord semantics differ (`Ctrl+B` panel vs Claude demote).

**Goal:** Make Face background-task presentation/interaction *feel* like Claude Code ΓÇö list, badges, expand, cancel, completion, non-blocking chat ΓÇö without rebuilding agent-team or Agent View.

---

## LOOK ΓÇö Evidence

### 1. Claude Code (local research tree)

Path: `.tmp-research-plugins/claude-code/`

| Area | Path / symbol | What it does |
|------|----------------|--------------|
| Task model | `src/tasks/types.ts` ΓÇö `BackgroundTaskState`, `isBackgroundTask` | Union: `local_bash`, `local_agent`, `remote_agent`, `in_process_teammate`, `local_workflow`, `monitor_mcp`, `dream`; only running/pending + backgrounded count as bg |
| Shell tasks | `src/tasks/LocalShellTask/` | `run_in_background` Bash; kill; output on disk |
| Stop | `src/tasks/stopTask.ts` ΓÇö `stopTask` | Shared kill for Tool + UI; suppresses noisy bash exit-137 notifs |
| Footer pill | `src/tasks/pillLabel.ts` ΓÇö `getPillLabel`, `pillNeedsCta` | Compact labels: `1 shell`, `2 monitors`, `Γùç ultraplan needs your input`, mixed ΓåÆ `N background tasks`; CTA only for attention states |
| Footer wire | `src/components/tasks/BackgroundTaskStatus.tsx` | Renders pill(s) in prompt footer; opens dialog |
| Footer host | `src/components/PromptInput/PromptInputFooterLeftSide.tsx` | Hosts `BackgroundTaskStatus` |
| `/tasks` | `src/commands/tasks/tasks.tsx` ΓåÆ `BackgroundTasksDialog` | Interactive modal list |
| Dialog | `src/components/tasks/BackgroundTasksDialog.tsx` | List Γåö detail; groups bash / remote / agent / teammate / workflow / monitor / dream; Enter opens detail; kill from detail |
| Row chrome | `src/components/tasks/BackgroundTask.tsx` + `ShellProgress.tsx` | Title + `(running)` / `(done)` / `(error)` / `(stopped)` |
| Shell detail | `src/components/tasks/ShellDetailDialog.tsx` | Status, runtime, command; **tails last 8KB** of output file every 1s while running; `x` = stop; ΓåÉ back |
| Checklist vs bg | Docs + `Ctrl+T` | Todo checklist Γëá background tasks |

### 2. Claude Code (docs / Exa)

| Source | UX claim |
|--------|----------|
| [Interactive mode ΓÇö Background Bash](https://code.claude.com/docs/en/interactive-mode) | `Ctrl+B` backgrounds running Bash/agents (tmux: press twice); async task ID; output to file; cleanup on exit; optional env disable |
| Same page ΓÇö Task list | `Ctrl+T` = **todo checklist**; bg work via `/tasks` |
| [Commands ΓÇö `/tasks`](https://code.claude.com/docs/en/commands) | `/tasks` (alias `/bashes`): view/manage session background work incl. finished subagents |
| [Tools reference](https://code.claude.com/docs/en/tools-reference) | `run_in_background: true`; timeout auto-moves to background; `TaskStop` |
| [Desktop ΓÇö Watch background tasks](https://code.claude.com/docs/en/desktop) | Tasks pane: subagents + bg shells + workflows; click ΓåÆ output / stop |
| [wmedia tip ΓÇö Tasks panel](https://wmedia.es/en/tips/claude-code-tasks-panel) | Navigation: ΓåæΓåô list, Enter open (output + Stop), ΓåÉ/Esc close; do not confuse with `Ctrl+T` checklist |
| GitHub #57079 | Tasks panel historically: title + status + Bash badge + cancel; **live stdout expand was a gap** (user blind vs agent `BashOutput`) ΓÇö Face already streams stdout into store/viewer (advantage) |
| GitHub #51490 / #62745 | Stale ΓÇ£RunningΓÇ¥ / Stop no-op when harness and UI diverge ΓÇö Face should keep ACPΓåöUI reconcile as a polish risk |

**Claude interaction grammar (distilled):**

1. **Non-blocking chat** ΓÇö bg shells/agents run while user types next prompt.  
2. **Ambient badge** ΓÇö footer pill with typed counts (`N shells`, `N monitors`, ΓÇª).  
3. **Open hub** ΓÇö `/tasks` or footer ΓåÆ modal list (not only a side pane).  
4. **Row ΓåÆ detail** ΓÇö Enter: status, runtime, command, **output tail**, Stop (`x`).  
5. **Demote** ΓÇö `Ctrl+B` moves foreground Bash/agent to background mid-run.  
6. **Completion** ΓÇö notify / unread suffix on finished agent/workflow rows; shells get quieter kill handling.  
7. **Separate checklist** ΓÇö todos (`Ctrl+T`) are not the bg hub.

Out of **this** planΓÇÖs chrome: Agent View (`claude agents`), `/background` whole-session detach, ultraplan/ultrareview cloud rows (optional later; belong nearer agent-team / multitask plans).

### 3. Face / next-code today (Grok pager)

| Surface | Path | Behavior |
|---------|------|----------|
| ACP lifecycle | `acp_handler/background.rs` ΓÇö `handle_task_backgrounded`, `handle_task_completed`; `route_bg_task_stdout` | Creates `BgTaskState`, maps tool_callΓåÆtask, streams stdout into central store |
| Ext methods | `x.ai/task_backgrounded`, `x.ai/task_completed`, `x.ai/task/kill` | Wire protocol for start/end/kill |
| Tasks pane | `views/tasks_pane.rs` ΓÇö `TasksPane` | Unified overlay: Subagents / Tasks / Watchers; kill; open output; line-count badges `(N)` / `(1.9k+)`; groups + collapse |
| Toggle | `ActionId::ToggleTasks` default **`Ctrl+B`** (`actions/defaults.rs`) | Side pane (Claude uses Ctrl+B for **demote**) |
| Demote | `Action::DemoteToBackground` ΓÇö **`Ctrl+G`** (user guide) | Foreground execute ΓåÆ bg (Claude: Ctrl+B) |
| Watching cue | `views/turn_status.rs` ΓÇö `watching_label` | Idle: `watching ┬╖ 1 command ┬╖ 2 monitors ┬╖ 1 loop ┬╖ 1 subagent` |
| Completion chip | `scrollback/blocks/bg_task.rs` ΓÇö `BgTaskBlock` Started/Completed/Failed | Collapsed transcript blocks; Enter/Ctrl-F ΓåÆ block viewer with live stdout |
| Info float | `info_floats/widgets.rs` ΓÇö `BackgroundInfo`, `render_background_lines` | `Background ┬╖ N running` + up to 3 truncated titles; **no open/kill** |
| `/tasks` | `slash/commands/tasks.rs` ΓåÆ `dispatch_show_tasks` | **Read-only system block** dump (esp. minimal mode) ΓÇö not ClaudeΓÇÖs interactive dialog |
| Docs | `docs/user-guide/20-background-tasks.md` | `background: true`, monitor, `/loop`, Ctrl+G demote, Ctrl+B pane |

**Face strengths already ahead of classic Claude panel complaints:** live stdout in central store + `block_viewer` BgTask viewer; unified subagents+tasks+watchers pane; watching status line; ACP-driven lifecycle.

### 4. Gap matrix

| Claude affordance | Face today | Gap |
|-------------------|------------|-----|
| Footer pill with typed counts + open CTA | Watching line above prompt; info-float count card | No Claude-style **footer pill** glued to prompt; float not clickable/openable |
| `/tasks` interactive listΓåödetail | `/tasks` = system text dump; interactive = `Ctrl+B` pane | Discoverability + Claude muscle memory; minimal mode lacks pane parity |
| Row badges `(running)` / `(done)` / `(error)` / `(stopped)` | Running/done via pane + duration; line-count badges | Align status vocabulary + color with Claude `ShellProgress` |
| Enter ΓåÆ output tail + Stop | Pane ΓåÆ viewer with full buffered stdout; kill actions exist | Ensure **one-click path** matches Claude detail (status/runtime/command/output/stop hints) |
| `Ctrl+B` demote foreground | **`Ctrl+G` demote**; **`Ctrl+B` toggle pane** | Chord semantics inverted vs Claude ΓÇö document mapping; optional Claude-keymap preset later (not MVP) |
| Completion toast / unread | `BgTaskBlock` completed/failed in scrollback; no ΓÇ£unreadΓÇ¥ pill suffix | Optional unread/attention on finished rows until viewed |
| Non-blocking chat while bg runs | Supported (ACP + watching cue) | Keep; regression-smoke only |
| Auto-bg on Bash timeout | Product/daemon behavior | Out of UI plan unless Face already surfaces it ΓÇö verify later |
| Stale Running / Stop no-op | ACP reconcile | Polish: dismiss dead rows; donΓÇÖt leave stuck Running |
| Cloud / ultraplan / dream rows | N/A or partial | Non-goal for this chrome plan |
| Todo checklist `Ctrl+T` | Separate todos pane | Keep separate (already) |

---

## Target UX (Claude-first)

### Principles

1. **Chat stays free** ΓÇö background work never owns the prompt.  
2. **Always know something is running** ΓÇö ambient badge with **typed** counts.  
3. **One hub** ΓÇö list all session-local parallel jobs (shells, monitors/loops, subagents).  
4. **Expand to see** ΓÇö output without asking the model.  
5. **Stop is one action** ΓÇö kill from list or detail; clear completed when dismissed.  
6. **Completion is quiet but visible** ΓÇö chip in transcript; optional attention on badge until opened.

### Target surfaces

| Surface | Claude analogue | Face target |
|---------|-----------------|-------------|
| Ambient | Footer `BackgroundTaskStatus` pill | Prompt-adjacent pill **or** strengthen watching line to Claude pill grammar (`N shells ┬╖ M monitors`) + ΓÇ£open tasksΓÇ¥ hint |
| Hub | `/tasks` dialog | Prefer: `Ctrl+B` / `/tasks` both open the **same** interactive hub (`TasksPane` or Claude-shaped modal). Minimal: `/tasks` should not be dump-only if pane unavailable ΓÇö upgrade dump or open overlay |
| Detail | `ShellDetailDialog` | Existing BgTask viewer + pane row actions; add Claude-like footer hints (Esc back, `x`/key stop) |
| Demote | `Ctrl+B` | Keep Face **`Ctrl+G`**; show hint when demoting (ΓÇ£running in background ┬╖ Ctrl+B tasksΓÇ¥) |
| Float | ΓÇö | Keep scroll float as glance; optionally deep-link click ΓåÆ hub (polish) |

### Non-goals

- Agent View / whole-session `/background` detach (ΓåÆ agent-team or later).  
- Cursor multitask / AskUserQuestion UI.  
- Changing daemon tool schemas (`background: true` already works).  
- Forcing Claude keybindings as default (optional preset only).  
- Cloud ultraplan/ultrareview rows.

---

## Phases

### MVP (ship feel)

1. **Ambient badge parity** ΓÇö Typed counts matching Claude pill vocabulary (shells / monitors / loops / subagents); visible whenever idle+watching or while turn runs with bg work.  
2. **Single hub entry** ΓÇö `/tasks` opens interactive `TasksPane` (or equivalent overlay) in full Face; keep text dump only where overlay impossible, or make dump actionable (ΓÇ£press Ctrl+BΓÇ¥).  
3. **Detail path smoke** ΓÇö From hub: open running shell ΓåÆ see live/tail output ΓåÆ Stop works ΓåÆ completed shows success/fail + elapsed.  
4. **Demote cue** ΓÇö After `Ctrl+G`, toast/chip: task backgrounded + how to open hub.  
5. **Docs** ΓÇö Update Face user-facing bg-task help with ClaudeΓåÆFace chord map.

### Polish

1. Status badge vocabulary `(running)` / `(done)` / `(error)` / `(stopped)` aligned with `ShellProgress`.  
2. Unread / attention after completion until detail opened.  
3. Info-float BackgroundTasks: open hub on activate; show status colors.  
4. Reconcile stuck Running (harness gone ΓåÆ dismiss/failed).  
5. Optional keymap preset ΓÇ£Claude chordsΓÇ¥ (`Ctrl+B` demote, alternate for pane).  
6. Finished-but-listed rows like Claude `/tasks` including finished subagents (toggle show_done already partial).

### Later / other plans

- Teammate pills / swarm footer ΓåÆ `PLAN-20260724-face-agent-team-claude-ux.md`  
- Parallel session fleet ΓåÆ multitask / Agent View research  

---

## Files likely to touch (implementation PR ΓÇö not this docs PR)

| File | Why |
|------|-----|
| `crates/xai-grok-pager/src/views/turn_status.rs` | Watching / pill grammar |
| `crates/xai-grok-pager/src/views/tasks_pane.rs` | Hub list, badges, detail entry, show_done |
| `crates/xai-grok-pager/src/views/block_viewer.rs` | BgTask detail chrome / hints |
| `crates/xai-grok-pager/src/scrollback/blocks/bg_task.rs` | Completion chip copy |
| `crates/xai-grok-pager/src/slash/commands/tasks.rs` + `dispatch/status.rs` | `/tasks` ΓåÆ open pane |
| `crates/xai-grok-pager/src/views/info_floats/widgets.rs` | Float ΓåÆ hub; richer rows |
| `crates/xai-grok-pager/src/actions/defaults.rs` | Hints / optional keymap notes |
| `crates/xai-grok-pager/docs/user-guide/20-background-tasks.md` | Chord map + Claude parity notes |
| Tests under `views/turn_status.rs`, `tasks_pane`, `acp_handler/tests/background_tasks.rs` | Regressions |

Daemon/ACP brain only if kill/complete events missing ΓÇö prefer Face-only first.

---

## Smoke (after implementation)

1. Start Face; ask agent to run a long `sleep` / build with `background: true` (or demote with `Ctrl+G`).  
2. Confirm chat prompt still accepts input (non-blocking).  
3. See ambient watching/pill with typed count.  
4. `Ctrl+B` opens hub; row shows Running + timer/line badge.  
5. Open detail ΓåÆ live/tail output; Stop ΓåÆ status updates; completed chip in transcript.  
6. `/tasks` opens same hub (or clear CTA), not an opaque dump only.  
7. Spawn monitor or `/loop` ΓåÆ appears under Watchers; kill works.  
8. Idle with watchers ΓåÆ watching line persists until empty.  
9. Regression: todos pane still separate from bg hub.

---

## Decision log

| Decision | Choice | Why |
|----------|--------|-----|
| Key chords | Keep Face defaults (`Ctrl+B` pane, `Ctrl+G` demote) for MVP | Already shipped + documented; Claude feel Γëá blind remapping |
| Hub shape | Prefer existing `TasksPane` over new Ink-style modal | Less churn; match Claude *affordances* not exact Ink Dialog |
| Scope vs agent-team | Exclude teammate pill tree | Separate plan owns swarm UX |
| Live stdout | Keep Face streaming store | Already better than Claude issue #57079 baseline |

---

## Research checklist

- [x] Claude local: `src/tasks/*`, `src/components/tasks/*`, `/tasks` command  
- [x] Exa/docs: interactive-mode, commands, desktop tasks pane, community `/tasks` tip, GH issues on panel/output  
- [x] Face: `tasks_pane`, `turn_status` watching, ACP background handler, info float, `/tasks` dump, user-guide 20  
- [x] Gap matrix + MVP slice  
- [x] Implementation (this PR: pr-face-background-task-claude-ux)

---

## Return blurb (for PR / handoff)

Plan path: `docs/plans/PLAN-20260724-face-background-task-claude-ux.md`  
MVP slice: typed ambient badge + `/tasks`ΓåÆinteractive hub + demote CTA + detail/stop smoke ΓÇö keep Face key chords.


---

## Shipped vs deferred (implementation PR)

### Shipped
- Claude-style ambient footer pill (1 shell · 2 monitors · Ctrl+B, unread · !)
- /tasks (+ /bashes) opens interactive TasksPane hub (non-minimal); minimal keeps dump + hub CTA
- Demote toast CTA after Ctrl+G (Running in background · Ctrl+B tasks) — Face chords kept (Ctrl+B hub, Ctrl+G demote)
- Status vocab on hub rows (
unning / done / error / stopped); unread mark on complete + clear on detail open
- Info-float header uses pill grammar; block-viewer Stop hint polish
- Docs: 20-background-tasks.md chord map + pill copy

### Deferred (honest)
- Info-float **click → open hub** (hint-only; floats may not be clickable yet)
- Optional Claude keymap preset (Ctrl+B = demote) — explicitly out of scope
- Stuck Running / harness reconcile polish (Claude GH #51490 class)
- Agent-team / cloud ultraplan / teammate rows — other plans
