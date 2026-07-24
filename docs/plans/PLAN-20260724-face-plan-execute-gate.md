# PLAN — Face Plan → execute gate

**Date:** 2026-07-24  
**Branch:** `pr-face-plan-execute-gate`  
**LOOK:** `LOOK-20260724-claude-code-ux-gaps-for-face.md` gap #2  
**Status:** done

---

## Goal

Claude-style **Plan → execute gate**: enter plan-only mode → durable `plan.md` → ExitPlanMode review UI (approve / revise / abandon) → unlock mutating tools and continue with the plan as contract.

---

## LOOK findings (what already exists)

Face client already has substantial UX:

| Piece | Location |
|-------|----------|
| `/plan` enter | `crates/xai-grok-pager/src/slash/commands/plan.rs` |
| `/view-plan` | `…/slash/commands/view_plan.rs` |
| Approval overlay (a/s/q, comments) | `…/views/plan_approval_view.rs`, `handle_exit_plan_mode` |
| Wire types | `xai-grok-tools/.../exit_plan_mode/types.rs` |
| Docs | `crates/xai-grok-pager/docs/user-guide/19-plan-mode.md` |

**Thin / missing vs Claude (this PR):**

1. Daemon never sends `x.ai/exit_plan_mode` (types only; no AskUserQuestion-style bridge).
2. `NextCodeFaceAgent::set_session_mode` uses ACP default no-op → no `CurrentModeUpdate` → plan mode never confirms / unlocks on daemon.
3. No `prePlanMode` stash/restore (OFF → hard `Default`).
4. `approve_plan` does not optimistically clear `plan_mode_pending` (unlike abandon).
5. No `/plan open` → `$EDITOR` (Claude `plan.tsx`).
6. No plan.md write exception under DCG `Mode::Plan`.

---

## Claude references used

| Aspect | Path |
|--------|------|
| `/plan` + `/plan open` | `.tmp-research-plugins/claude-code/src/commands/plan/plan.tsx` |
| Plan artifact | `…/src/utils/plans.ts` |
| ExitPlanMode tool | `…/packages/builtin-tools/src/tools/ExitPlanModeTool/ExitPlanModeV2Tool.ts` |
| EnterPlanMode tool | `…/packages/builtin-tools/src/tools/EnterPlanModeTool/EnterPlanModeTool.ts` |
| Safety overview | `…/docs/safety/plan-mode.mdx` |
| Pattern mirror in-repo | `AskUserQuestion` Face bridge (`src/cli/face_ask_user.rs` + app-core tool) |

---

## Ship design

```
/plan | EnterPlanMode tool
        → set_session_mode(plan) + stash prePlanMode
        → CurrentModeUpdate(plan)
        → DCG Mode::Plan (writes denied except plan.md)

agent writes plan.md → ExitPlanMode tool
        → ServerEvent::ExitPlanMode
        → Face ACP x.ai/exit_plan_mode
        → PlanApprovalView (a approve / s revise / q abandon)

approved  → restore prePlanMode (or Default) + CurrentModeUpdate
            + plan_mode_pending=false (Face optimism)
cancelled → stay in plan; feedback to model
abandoned → exit plan without build
```

### Explicit deferrals

- EnterPlanMode confirmation dialog (Claude EnterPlanModePermissionRequest)
- ExitPlanMode `allowedPrompts` / accept-edits picker on approve
- Session `/diff` review (LOOK follow-on)
- Hooks auto-approve ExitPlanMode
