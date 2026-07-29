# PLAN — Face Best-of-N UI (see + pick candidates)

**Date:** 2026-07-24  
**Branch:** `pr-face-best-of-n-ui`  
**Status:** Implemented  
**LOOK:** `docs/plans/LOOK-20260724-codebuff-ux-parity-face.md`  
**Risk:** Medium — new ACP reverse method + scrollback block; engine path unchanged for `mode=auto`

## Summary

Engine BoN already fans out candidates and auto-selects. Face had no parallel candidate chrome, and `mode=show` silently auto-applied with a stub note. This plan ships Codebuff-like **visibility + pick/cancel**:

1. Live candidate rows/cards + progress while a BoN run is active  
2. Interactive picker for `mode=show` (select winner or cancel — no silent apply)  
3. Clear **Proposed** vs **Edit/Creating** chrome for `propose_*` vs applied edits  
4. ACP bridge so Face receives BoN progress and pick requests  

## Evidence (verified)

| Claim | Where |
|-------|-------|
| `mode=show` stub auto-applies | `crates/next-code-app-core/src/agent/turn_execution.rs` (~L107–110) |
| Progress only as TextDelta `[best-of-n] …` | `best_of_n_orchestrator.rs` `emit_progress` |
| AskUserQuestion reverse pattern | `face_ask_user.rs` → `x.ai/ask_user_question` → `question_view` |
| Edit block prefixes | `ToolCallBlock::from_name` / `EditToolCallBlock::with_prefix` |

## Copy / wire / delete map

| Kind | What |
|------|------|
| **Wire** | `ServerEvent::BestOfNProgress` + `BestOfNPickRequest` / `Request::BestOfNPickResponse` |
| **Wire** | Orchestrator emits structured progress; `show` waits for pick before apply |
| **Wire** | `pager_agent` → `face_best_of_n` → ACP `x.ai/best_of_n/progress` (notif) + `x.ai/best_of_n/pick` (blocking) |
| **Copy-shape** | Face `best_of_n_view` mirrors AskUserQuestion / plan-approval overlay pattern |
| **Wire** | Scrollback `BestOfNBlock` for running + completed candidate cards |
| **Wire** | `propose_*` → Edit block with `Proposed ` prefix |
| **Delete** | Stub note “picker UI is not implemented yet” |

## Non-goals / deferred

- Full masonry multi-column ImplementorCard layout (Codebuff React grid)  
- Per-file inline DiffViewer click-through inside a candidate card  
- `@Agent` mentions / `/review` scope picker  
- Keyword-highlight / `/btw` sidebar (collision avoidance)

## Test plan

- Unit: progress event → UI state reduce; pick Selected / Cancelled  
- Unit: `propose_*` maps to Proposed prefix; `edit`/`write` unchanged  
- Orchestrator: show mode does not apply until pick; cancel clears without apply  
- `cargo check -p next-code-app-core -p xai-grok-pager` + targeted tests  

## Smoke (manual)

1. Config `[best_of_n] mode = "show"`, count ≥ 2  
2. Trigger `$bestofn` edit request in Face  
3. See generating progress + candidate rows  
4. Arrow/Enter pick winner → files applied; Esc cancel → no apply  
5. Confirm propose tool rows say **Proposed**, applied edits say **Edit**/**Creating**
