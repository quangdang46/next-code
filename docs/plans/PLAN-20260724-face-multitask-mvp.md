# Plan Report — Face Multitask MVP (Cursor 3.2 light mirror)

## Summary (read this first)

- **You asked:** Temporary implementation plan only — mirror Cursor 3.2 `/multitask` (and light follow-ons) in next-code Face, without implementing the feature in this task.
- **Verdict (prior research + re-check):** Feasible now. Reuse swarm **light** workers + Face ACP **background subagents** + existing **prompt queue**. MVP = drain queued prompts as parallel light workers / background subagents with report-back; concurrency cap = `agents.swarm_max_concurrent_agents` (default 32) with light-mode default concurrency as the cheap fan-out floor.
- **Defer:** True multi-root workspaces; full Agents Window clone; auto-decompose of a single large prompt into a fleet (Cursor does this; we skip in v1).
- **Status:** Draft / temporary — **not implementing** in this task. Docs-only branch for review.

---

## Goal

Ship a Face-facing **Multitask** mode that matches Cursor’s primary UX promise:

> Queued (or explicitly multitasked) independent prompts run as **async background workers** in parallel, instead of draining the queue sequentially. Parent stays interactive; each worker reports a final summary back.

## Non-goals (explicit)

| Non-goal | Why park |
|----------|----------|
| **True multi-root** (one session, many `working_dir`s) | Session / daemon identity is single `working_dir` today; architectural change, not a slash command |
| **Full Agents Window clone** | Heavy chrome; Face already has tasks pane + subagent rows + `/queue` — enough for MVP |
| **Auto-decompose v1** | Cursor splits one large `/multitask …` into chunks; skip until queue-parallel path is solid |
| **Cloud / VM subagents** | Cursor `/in-cloud`; out of scope |
| **Dependency-aware scheduling** | Cursor does not promise this either; user supplies independent prompts |
| **Legacy TUI-only multitask** | Face-first; TUI may later reuse the same daemon/swarm path |

---

## LOOK — Cursor 3.2 surface (external)

Sources: [Cursor changelog 3.2 (2026-04-24)](https://cursor.com/changelog/04-24-26), [subagents docs](https://cursor.com/docs/subagents.md), [worktrees docs](https://cursor.com/docs/configuration/worktrees.md), [Agents Window](https://cursor.com/docs/agent/agents-window), forum/AgentPatterns writeups.

| Cursor behavior | Notes for next-code |
|-----------------|---------------------|
| `/multitask` → async background subagents instead of queue | Primary MVP target |
| Existing queue → “multitask these instead of waiting” | Drain `shared_prompt_queues` / local queue into parallel spawns |
| Auto-decompose large task → fleet | **Defer** (non-goal v1) |
| Context isolation; parent gets final summary only | Matches Face/Grok background subagent + swarm report-back |
| Overlapping writes → compose with worktrees | Phase 2 optional |
| Multi-root workspace | Phase 3 park |
| One-click promote worktree → foreground | Phase 2 minimal apply UX |

Cursor forum nuance (v0 honesty): no special conflict resolution yet; worktrees recommended when writes overlap.

---

## Phase 0 — Inventory map (Cursor → next-code primitive)

| Cursor feature | next-code primitive (paths) | Fit |
|----------------|----------------------------|-----|
| Queue of follow-up prompts | Face: `xai-grok-pager` `shared_prompt_queues` + `dispatch/queue.rs`; slash `/queue`; crate `xai_prompt_queue`. Legacy TUI: `queued_messages` in `next-code-tui` | **Reuse** — drain source for multitask |
| Async background subagents | Face ACP Task / `spawn_subagent` path: `app/subagent.rs`, `acp_handler` subagent tests, `scrollback/blocks/subagent.rs` (`is_background`), tasks pane (`views/tasks_pane.rs`, `views/turn_status.rs`) | **Reuse** — report-back + UI rows |
| Parallel worker fleet + cap | Swarm light: `next-code-app-core` `tool/communicate.rs` (`LIGHT_MODE_DEFAULT_CONCURRENCY = 4`, `run_plan` / `fill_slots`); config `agents.swarm_max_concurrent_agents` (default **32**) in `next-code-config-types`; DAG `next-code-plan` `dag/schedule.rs` (`LIGHT_MODE_SUGGESTED_WORKERS`); docs `docs/SWARM_TASK_GRAPH.md` | **Reuse** — concurrency + light fan-out |
| Plan graph / deep orchestrator | `run_plan`, swarm persistence, `EffortKind::SwarmLight` / `SwarmDeep` in `next-code-base` `prompt.rs` | **Optional later** — not required for queue-drain MVP |
| Worktree isolation | Persona `default_isolation = "worktree"`; Task `isolation: worktree`; `/fork --worktree`; ACP `x.ai/git/worktree/*` (`worktree_cmd/mod.rs`, user-guide 16/17) | **Core exists** — UX apply/promote still thin |
| Worktree apply / promote | Docs: create / **apply** / remove / list / gc; effects hit create/resume/list paths today; one-click Face promote still substantial | **Phase 2** — minimal ACP apply + Face affordance |
| Multi-root | Single session `working_dir` throughout prompt/session stack | **Avoid v1** |

### Recommended MVP backend (decision)

Prefer **Face background subagents** (one child per queued prompt, `background: true`) as the user-visible unit, with concurrency gated by the same number as `swarm_max_concurrent_agents` (or a dedicated `multitask_max_concurrent` that defaults to that value).

Use **swarm light `run_plan`** only if implementers need a single coordinator DAG for N identical-shaped nodes; do not force deep swarm UX into Multitask v1.

---

## Phase 1 — Multitask MVP (effort: **M**)

**Product shape**

1. Slash `/multitask` (and optional `[ui]` / settings toggle “Multitask mode”) on Face.
2. Behaviors:
   - **A. Drain queue:** If prompts are queued behind the running turn, `/multitask` (or “multitask queued”) converts them into parallel background workers instead of FIFO sequential drain.
   - **B. Inline args:** `/multitask <text>` treats the text as **one** worker prompt in v1 (no auto-split). Optional later: N newline-separated prompts → N workers.
3. Cap: do not exceed `agents.swarm_max_concurrent_agents` (default 32). Excess stay queued / wait for a free slot.
4. Parent session stays interactive; completions land as subagent/task chips + summary into parent (existing background-subagent completion path).
5. Default isolation: **`none`** (shared workspace) — matches Cursor multitask-without-worktree; warn in help text about overlapping writes.

**Suggested file touch list (implementation later — not this PR)**

| Area | Likely paths |
|------|----------------|
| Slash | `crates/xai-grok-pager/src/slash/commands/multitask.rs` + register in `mod.rs` |
| Dispatch | `crates/xai-grok-pager/src/app/dispatch/` (queue drain → spawn effects) |
| Settings | `settings/defs.rs`, `xai-grok-shared` `ui_config.rs` if toggle |
| Daemon / ACP | `src/cli/pager_agent.rs` / face ext only if new ext method needed; prefer existing Task/subagent spawn |
| Swarm reuse (if chosen) | `communicate.rs` `run_plan` light + server `swarm.rs` |
| Docs | Face user-facing note; this plan → mark Implemented when shipped |

**Acceptance (Phase 1)**

- With 3 independent prompts in queue + busy parent turn, `/multitask` starts ≤3 background children (subject to cap) without waiting for sequential queue drain.
- Parent can type a new prompt while workers run.
- Each child reports a terminal summary; tasks pane / turn status counts background subagents correctly.
- Cap respected (force low `swarm_max_concurrent_agents` in smoke).

---

## Phase 2 — Optional worktree isolation + minimal apply (effort: **L**)

When multitask workers will **write overlapping paths**, compose with existing isolation:

1. `/multitask --worktree` (or setting) → spawn each worker with `isolation: worktree`.
2. Minimal Face UX: list child worktrees; **Apply** via existing ACP `x.ai/git/worktree/apply` (wire if thin); no full “one-click promote to Agents Window” chrome.
3. Conflict honesty: document manual merge; no auto-rebase magic in v1.

**Non-goal in Phase 2:** Cursor-complete worktree Agents Window promote flow.

---

## Phase 3 — Park multi-root (effort: **L+**, do not start)

- Requires session model changes beyond a single `working_dir`.
- Cross-repo work today: separate sessions / separate Face roots, or monorepo checkout.
- Revisit only with an explicit architecture RFC (session identity, tools cwd, MCP roots, notepad path, etc.).

---

## Risks

| Risk | Mitigation |
|------|------------|
| Parallel edits corrupt shared tree | Default isolation `none` + docs warning; Phase 2 worktree opt-in |
| Double-drain: queue FIFO + multitask both claim entries | Single ownership: multitask consumes/removes from queue before spawn |
| Cap mismatch (light default 4 vs swarm_max 32) | Document chosen ceiling; prefer one config knob |
| Permission storms (N children ask) | YOLO / shared grants; or serialize permission UI — open question |
| Cost / token blowup | Cap + optional readonly explore-type for research-only multitask |
| Conflict with Face paste-token / other Face PRs | Implement on short feature branch from `dev`; docs-only this PR |

## Open questions

1. **Backend:** Face Task background subagents only vs light swarm `run_plan` coordinator?
2. **Toggle vs slash:** sticky “multitask mode” (all new prompts fan out) vs one-shot `/multitask` drain?
3. **Inline multi-prompt:** support newline/`---` separators in v1 or only queue drain?
4. **Permissions:** parent YOLO fans out to children automatically?
5. **Legacy TUI:** expose `/multitask` there in same PR or Face-only first?
6. **Apply UX:** modal vs tasks-pane action for worktree apply?

## Smoke checklist (when implementing)

- [ ] Queue 2–3 independent prompts mid-turn → `/multitask` → N background rows in tasks pane
- [ ] Parent remains responsive; `/queue` empty (or shows only overflow past cap)
- [ ] Completions: summary chips; no stuck “running” after child exit
- [ ] Cap=1: second/third wait until slot free
- [ ] Overlapping write scenario without worktree: observe conflict (expected); with `--worktree`: separate trees + apply one
- [ ] `/multitask` with empty queue + no args: friendly error
- [ ] `cargo check` / targeted Face queue + subagent tests
- [ ] No multi-root assumptions in prompts or tools

## Effort summary

| Phase | Effort | Status |
|-------|--------|--------|
| 0 Inventory | S (done in this plan) | Complete (doc) |
| 1 Multitask MVP | **M** | Not started |
| 2 Worktree + apply minimal | **L** | Optional / later |
| 3 Multi-root | **L+** | Parked |

---

## References

- Prior agent research: `d760245a-d025-4b1c-9bfa-72ad164a316d`
- Cursor: https://cursor.com/changelog/04-24-26
- Local: `docs/SWARM_TASK_GRAPH.md`, Face `docs/user-guide/16-subagents.md`, `17-sessions.md`, `20-background-tasks.md`

## Worktree / branch (this plan PR)

- Branch: `docs/multitask-mvp-plan`
- Worktree: `…/next-code-worktrees/docs-multitask-mvp-plan`
- Base: `origin/dev`
- Scope: **this markdown file only** — no feature code
