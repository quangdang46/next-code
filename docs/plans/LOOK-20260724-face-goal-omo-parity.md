# LOOK — Face `/goal` vs oh-my-openagent

**Date:** 2026-07-24  
**Branch / worktree:** `pr-face-goal` @ `next-code-worktrees/face-goal`  
**Reference:** [code-yeongyu/oh-my-openagent](https://github.com/code-yeongyu/oh-my-openagent) `packages/omo-opencode/src/hooks/goal/`

**Status:** Full OMO-shaped session-goal parity **complete** (this PR). Durable initiatives (`initiative` / `/goals`) remain separate. Grok `goal_classifier` / multi-deliverable orchestrator deferred.

---

## OMO `/goal` (source of truth for UX)

| Piece | Behavior |
|-------|----------|
| Parse | `""` → show; `pause` / `resume` / `clear`; else → `setObjective` |
| Persist | OMO: `.omo/goal/{sessionID}.json` — next-code: `~/.next-code/session-goals/{urlencoded}.json` |
| Idle | EndTurn → continuation prompt while status `active` (max 100) |
| Tools | `create_goal` / `update_goal` / `get_goal` |
| Face chrome | ACP `GoalUpdated` (`active`/`user_paused`/`complete`/`cleared`) |

---

## Shipped (full scope)

| Layer | Status |
|-------|--------|
| Store / prompts | `next-code-base::session_goal` — set/get/pause/resume/clear/account_usage/bump + prompts |
| Config | `[goal] enabled` (default true), `max_continuations` (default 100) |
| Tools + bus | `create_goal` / `update_goal` / `get_goal` + `BusEvent::SessionGoalUpdated` |
| Face ACP | `x.ai/goal/{set,pause,resume,clear,status}` emit `GoalUpdated`; idle continuation after EndTurn |
| Face slash | Effects → ACP; optimistic chrome; set/resume still enqueue pursuit prompts |
| Initiatives | Untouched (`initiative` / `~/.next-code/goals/`) |

---

## Explicit non-goals (still deferred)

- Replacing `/goals` / `initiative` / ultragoal keyword skill
- Implementing `goal_classifier` verifier / multi-deliverable planner
- Persisting under `.omo/` (use `.next-code/session-goals/`)

---

## v1 (slash-only) — superseded

1. `slash/commands/goal.rs` — parse + run ✅
2. Local-only `dispatch/goal.rs` — replaced by ACP Effects ✅
3. Alias `mission` ✅

**Worktree:** `C:\Users\ADMIN\Documents\Projects\next-code-worktrees\face-goal`  
**Branch:** `pr-face-goal`
