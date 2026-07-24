# Plan — Face background-task UI/UX → Claude Code parity

**Status:** LOOK complete · plan only (no product implementation in this PR)  
**Date:** 2026-07-24  
**Branch:** `docs/background-task-claude-ux-plan`  
**Scope:** Background-task **chrome** — list/panel, badges, expand output, cancel/stop, completion cues, non-blocking chat.  
**Not in scope:** Agent-team / teammate swarm UX · Cursor `/multitask` · AskUserQuestion multi-prompt · full Agent View (`claude agents`) session fleet.

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

**Goal:** Make Face background-task presentation/interaction *feel* like Claude Code — list, badges, expand, cancel, completion, non-blocking chat — without rebuilding agent-team or Agent View.

---

## LOOK — Evidence

### 1. Claude Code (local research tree)

Path: `.tmp-research-plugins/claude-code/`

| Area | Path / symbol | What it does |
|------|----------------|--------------|
| Task model | `src/tasks/types.ts` — `BackgroundTaskState`, `isBackgroundTask` | Union: `local_bash`, `local_agent`, `remote_agent`, `in_process_teammate`, `local_workflow`, `monitor_mcp`, `dream`; only running/pending + backgrounded count as bg |
| Shell tasks | `src/tasks/LocalShellTask/` | `run_in_background` Bash; kill; output on disk |
| Stop | `src/tasks/stopTask.ts` — `stopTask` | Shared kill for Tool + UI; suppresses noisy bash exit-137 notifs |
| Footer pill | `src/tasks/pillLabel.ts` — `getPillLabel`, `pillNeedsCta` | Compact labels: `1 shell`, `2 monitors`, `◇ ultraplan needs your input`, mixed → `N background tasks`; CTA only for attention states |
| Footer wire | `src/components/tasks/BackgroundTaskStatus.tsx` | Renders pill(s) in prompt footer; opens dialog |
| Footer host | `src/components/PromptInput/PromptInputFooterLeftSide.tsx` | Hosts `BackgroundTaskStatus` |
| `/tasks` | `src/commands/tasks/tasks.tsx` → `BackgroundTasksDialog` | Interactive modal list |
| Dialog | `src/components/tasks/BackgroundTasksDialog.tsx` | List ↔ detail; groups bash / remote / agent / teammate / workflow / monitor / dream; Enter opens detail; kill from detail |
| Row chrome | `src/components/tasks/BackgroundTask.tsx` + `ShellProgress.tsx` | Title + `(running)` / `(done)` / `(error)` / `(stopped)` |
| Shell detail | `src/components/tasks/ShellDetailDialog.tsx` | Status, runtime, command; **tails last 8KB** of output file every 1s while running; `x` = stop; ← back |
| Checklist vs bg | Docs + `Ctrl+T` | Todo checklist ≠ background tasks |

### 2. Claude Code (docs / Exa)

| Source | UX claim |
|--------|----------|
| [Interactive mode — Background Bash](https://code.claude.com/docs/en/interactive-mode) | `Ctrl+B` backgrounds running Bash/agents (tmux: press twice); async task ID; output to file; cleanup on exit; optional env disable |
| Same page — Task list | `Ctrl+T` = **todo checklist**; bg work via `/tasks` |
| [Commands — `/tasks`](https://code.claude.com/docs/en/commands) | `/tasks` (alias `/bashes`): view/manage session background work incl. finished subagents |
| [Tools reference](https://code.claude.com/docs/en/tools-reference) | `run_in_background: true`; timeout auto-moves to background; `TaskStop` |
| [Desktop — Watch background tasks](https://code.claude.com/docs/en/desktop) | Tasks pane: subagents + bg shells + workflows; click → output / stop |
| [wmedia tip — Tasks panel](https://wmedia.es/en/tips/claude-code-tasks-panel) | Navigation: ↑↓ list, Enter open (output + Stop), ←/Esc close; do not confuse with `Ctrl+T` checklist |
| GitHub #57079 | Tasks panel historically: title + status + Bash badge + cancel; **live stdout expand was a gap** (user blind vs agent `BashOutput`) — Face already streams stdout into store/viewer (advantage) |
| GitHub #51490 / #62745 | Stale “Running” / Stop no-op when harness and UI diverge — Face should keep ACP↔UI reconcile as a polish risk |

**Claude interaction grammar (distilled):**

1. **Non-blocking chat** — bg shells/agents run while user types next prompt.  
2. **Ambient badge** — footer pill with typed counts (`N shells`, `N monitors`, …).  
3. **Open hub** — `/tasks` or footer → modal list (not only a side pane).  
4. **Row → detail** — Enter: status, runtime, command, **output tail**, Stop (`x`).  
5. **Demote** — `Ctrl+B` moves foreground Bash/agent to background mid-run.  
6. **Completion** — notify / unread suffix on finished agent/workflow rows; shells get quieter kill handling.  
7. **Separate checklist** — todos (`Ctrl+T`) are not the bg hub.

Out of **this** plan’s chrome: Agent View (`claude agents`), `/background` whole-session detach, ultraplan/ultrareview cloud rows (optional later; belong nearer agent-team / multitask plans).

### 3. Face / next-code today (Grok pager)

| Surface | Path | Behavior |
|---------|------|----------|
| ACP lifecycle | `acp_handler/background.rs` — `handle_task_backgrounded`, `handle_task_completed`; `route_bg_task_stdout` | Creates `BgTaskState`, maps tool_call→task, streams stdout into central store |
| Ext methods | `x.ai/task_backgrounded`, `x.ai/task_completed`, `x.ai/task/kill` | Wire protocol for start/end/kill |
| Tasks pane | `views/tasks_pane.rs` — `TasksPane` | Unified overlay: Subagents / Tasks / Watchers; kill; open output; line-count badges `(N)` / `(1.9k+)`; groups + collapse |
| Toggle | `ActionId::ToggleTasks` default **`Ctrl+B`** (`actions/defaults.rs`) | Side pane (Claude uses Ctrl+B for **demote**) |
| Demote | `Action::DemoteToBackground` — **`Ctrl+G`** (user guide) | Foreground execute → bg (Claude: Ctrl+B) |
| Watching cue | `views/turn_status.rs` — `watching_label` | Idle: `watching · 1 command · 2 monitors · 1 loop · 1 subagent` |
| Completion chip | `scrollback/blocks/bg_task.rs` — `BgTaskBlock` Started/Completed/Failed | Collapsed transcript blocks; Enter/Ctrl-F → block viewer with live stdout |
| Info float | `info_floats/widgets.rs` — `BackgroundInfo`, `render_background_lines` | `Background · N running` + up to 3 truncated titles; **no open/kill** |
| `/tasks` | `slash/commands/tasks.rs` → `dispatch_show_tasks` | **Read-only system block** dump (esp. minimal mode) — not Claude’s interactive dialog |
| Docs | `docs/user-guide/20-background-tasks.md` | `background: true`, monitor, `/loop`, Ctrl+G demote, Ctrl+B pane |

**Face strengths already ahead of classic Claude panel complaints:** live stdout in central store + `block_viewer` BgTask viewer; unified subagents+tasks+watchers pane; watching status line; ACP-driven lifecycle.

### 4. Gap matrix

| Claude affordance | Face today | Gap |
|-------------------|------------|-----|
| Footer pill with typed counts + open CTA | Watching line above prompt; info-float count card | No Claude-style **footer pill** glued to prompt; float not clickable/openable |
| `/tasks` interactive list↔detail | `/tasks` = system text dump; interactive = `Ctrl+B` pane | Discoverability + Claude muscle memory; minimal mode lacks pane parity |
| Row badges `(running)` / `(done)` / `(error)` / `(stopped)` | Running/done via pane + duration; line-count badges | Align status vocabulary + color with Claude `ShellProgress` |
| Enter → output tail + Stop | Pane → viewer with full buffered stdout; kill actions exist | Ensure **one-click path** matches Claude detail (status/runtime/command/output/stop hints) |
| `Ctrl+B` demote foreground | **`Ctrl+G` demote**; **`Ctrl+B` toggle pane** | Chord semantics inverted vs Claude — document mapping; optional Claude-keymap preset later (not MVP) |
| Completion toast / unread | `BgTaskBlock` completed/failed in scrollback; no “unread” pill suffix | Optional unread/attention on finished rows until viewed |
| Non-blocking chat while bg runs | Supported (ACP + watching cue) | Keep; regression-smoke only |
| Auto-bg on Bash timeout | Product/daemon behavior | Out of UI plan unless Face already surfaces it — verify later |
| Stale Running / Stop no-op | ACP reconcile | Polish: dismiss dead rows; don’t leave stuck Running |
| Cloud / ultraplan / dream rows | N/A or partial | Non-goal for this chrome plan |
| Todo checklist `Ctrl+T` | Separate todos pane | Keep separate (already) |

---

## Target UX (Claude-first)

### Principles

1. **Chat stays free** — background work never owns the prompt.  
2. **Always know something is running** — ambient badge with **typed** counts.  
3. **One hub** — list all session-local parallel jobs (shells, monitors/loops, subagents).  
4. **Expand to see** — output without asking the model.  
5. **Stop is one action** — kill from list or detail; clear completed when dismissed.  
6. **Completion is quiet but visible** — chip in transcript; optional attention on badge until opened.

### Target surfaces

| Surface | Claude analogue | Face target |
|---------|-----------------|-------------|
| Ambient | Footer `BackgroundTaskStatus` pill | Prompt-adjacent pill **or** strengthen watching line to Claude pill grammar (`N shells · M monitors`) + “open tasks” hint |
| Hub | `/tasks` dialog | Prefer: `Ctrl+B` / `/tasks` both open the **same** interactive hub (`TasksPane` or Claude-shaped modal). Minimal: `/tasks` should not be dump-only if pane unavailable — upgrade dump or open overlay |
| Detail | `ShellDetailDialog` | Existing BgTask viewer + pane row actions; add Claude-like footer hints (Esc back, `x`/key stop) |
| Demote | `Ctrl+B` | Keep Face **`Ctrl+G`**; show hint when demoting (“running in background · Ctrl+B tasks”) |
| Float | — | Keep scroll float as glance; optionally deep-link click → hub (polish) |

### Non-goals

- Agent View / whole-session `/background` detach (→ agent-team or later).  
- Cursor multitask / AskUserQuestion UI.  
- Changing daemon tool schemas (`background: true` already works).  
- Forcing Claude keybindings as default (optional preset only).  
- Cloud ultraplan/ultrareview rows.

---

## Phases

### MVP (ship feel)

1. **Ambient badge parity** — Typed counts matching Claude pill vocabulary (shells / monitors / loops / subagents); visible whenever idle+watching or while turn runs with bg work.  
2. **Single hub entry** — `/tasks` opens interactive `TasksPane` (or equivalent overlay) in full Face; keep text dump only where overlay impossible, or make dump actionable (“press Ctrl+B”).  
3. **Detail path smoke** — From hub: open running shell → see live/tail output → Stop works → completed shows success/fail + elapsed.  
4. **Demote cue** — After `Ctrl+G`, toast/chip: task backgrounded + how to open hub.  
5. **Docs** — Update Face user-facing bg-task help with Claude→Face chord map.

### Polish

1. Status badge vocabulary `(running)` / `(done)` / `(error)` / `(stopped)` aligned with `ShellProgress`.  
2. Unread / attention after completion until detail opened.  
3. Info-float BackgroundTasks: open hub on activate; show status colors.  
4. Reconcile stuck Running (harness gone → dismiss/failed).  
5. Optional keymap preset “Claude chords” (`Ctrl+B` demote, alternate for pane).  
6. Finished-but-listed rows like Claude `/tasks` including finished subagents (toggle show_done already partial).

### Later / other plans

- Teammate pills / swarm footer → `PLAN-20260724-face-agent-team-claude-ux.md`  
- Parallel session fleet → multitask / Agent View research  

---

## Files likely to touch (implementation PR — not this docs PR)

| File | Why |
|------|-----|
| `crates/xai-grok-pager/src/views/turn_status.rs` | Watching / pill grammar |
| `crates/xai-grok-pager/src/views/tasks_pane.rs` | Hub list, badges, detail entry, show_done |
| `crates/xai-grok-pager/src/views/block_viewer.rs` | BgTask detail chrome / hints |
| `crates/xai-grok-pager/src/scrollback/blocks/bg_task.rs` | Completion chip copy |
| `crates/xai-grok-pager/src/slash/commands/tasks.rs` + `dispatch/status.rs` | `/tasks` → open pane |
| `crates/xai-grok-pager/src/views/info_floats/widgets.rs` | Float → hub; richer rows |
| `crates/xai-grok-pager/src/actions/defaults.rs` | Hints / optional keymap notes |
| `crates/xai-grok-pager/docs/user-guide/20-background-tasks.md` | Chord map + Claude parity notes |
| Tests under `views/turn_status.rs`, `tasks_pane`, `acp_handler/tests/background_tasks.rs` | Regressions |

Daemon/ACP brain only if kill/complete events missing — prefer Face-only first.

---

## Smoke (after implementation)

1. Start Face; ask agent to run a long `sleep` / build with `background: true` (or demote with `Ctrl+G`).  
2. Confirm chat prompt still accepts input (non-blocking).  
3. See ambient watching/pill with typed count.  
4. `Ctrl+B` opens hub; row shows Running + timer/line badge.  
5. Open detail → live/tail output; Stop → status updates; completed chip in transcript.  
6. `/tasks` opens same hub (or clear CTA), not an opaque dump only.  
7. Spawn monitor or `/loop` → appears under Watchers; kill works.  
8. Idle with watchers → watching line persists until empty.  
9. Regression: todos pane still separate from bg hub.

---

## Decision log

| Decision | Choice | Why |
|----------|--------|-----|
| Key chords | Keep Face defaults (`Ctrl+B` pane, `Ctrl+G` demote) for MVP | Already shipped + documented; Claude feel ≠ blind remapping |
| Hub shape | Prefer existing `TasksPane` over new Ink-style modal | Less churn; match Claude *affordances* not exact Ink Dialog |
| Scope vs agent-team | Exclude teammate pill tree | Separate plan owns swarm UX |
| Live stdout | Keep Face streaming store | Already better than Claude issue #57079 baseline |

---

## Research checklist

- [x] Claude local: `src/tasks/*`, `src/components/tasks/*`, `/tasks` command  
- [x] Exa/docs: interactive-mode, commands, desktop tasks pane, community `/tasks` tip, GH issues on panel/output  
- [x] Face: `tasks_pane`, `turn_status` watching, ACP background handler, info float, `/tasks` dump, user-guide 20  
- [x] Gap matrix + MVP slice  
- [ ] Implementation (follow-up PR)

---

## Return blurb (for PR / handoff)

Plan path: `docs/plans/PLAN-20260724-face-background-task-claude-ux.md`  
MVP slice: typed ambient badge + `/tasks`→interactive hub + demote CTA + detail/stop smoke — keep Face key chords.
