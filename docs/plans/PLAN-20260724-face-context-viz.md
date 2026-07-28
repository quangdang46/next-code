# PLAN — Face `/context` API-true visualization

**Date:** 2026-07-24  
**Branch:** `pr-face-context-viz`  
**Gap:** LOOK `docs/plans/LOOK-20260724-claude-code-ux-gaps-for-face.md` §5

---

## Problem

Face `/context` already has Claude-style chrome (`ContextInfoBlock`: categorical 100-cell grid + legend + tool/skills info rows). The ACP brain behind it is a stub:

`src/cli/face_ext.rs::session_info_payload` sets `used = input+output`, `total = used`, zeros for system/tools, and no compact-aware message view. The viz therefore cannot reflect what the model actually sees.

## Claude reference (local plugin)

| Piece | Path | Behavior |
|-------|------|----------|
| `/context` command | `.tmp-research-plugins/claude-code/src/commands/context/context.tsx` | `toApiView` = compact boundary + optional collapse `projectView`, then `microcompactMessages`, then `analyzeContextUsage` |
| Grid UI | `…/src/components/ContextVisualization.tsx` | Colored squares by category (system, tools, messages, free, autocompact buffer) |
| Analysis | `…/src/utils/analyzeContext.ts` | Categories aligned with API payload buckets |

Key Claude comment: *“Apply the same context transforms query.ts does before the API call, so /context shows what the model actually sees rather than the REPL's raw history.”*

## next-code equivalent view model

| Bucket | Source of truth |
|--------|-----------------|
| System prompt | `prompt::build_system_prompt_full` (+ AGENTS.md, overlays, skills listing, memory) for session `working_dir` |
| Messages | Session messages **after** `StoredCompactionState.compacted_count`, plus compact `summary_text` when present (same skip pattern as TUI `context_snapshot`) |
| Tool definitions | Cold estimate (builtin-scale) when live `Registry::definitions` is unavailable; count + chars/4 |
| Skills | Skill listing section size from prompt builder / `SkillRegistry` → `usage_categories` |
| Window total | `provider::models::context_limit_for_model(session.model)` (fallback 128k) |
| Used | `system + messages + tools` (chars/4 via `xai_token_estimation` / `estimate_tokens`), floored by last observed input tokens when present |
| Compact stats | `compaction.compacted_count` + turn/tool-call counts from the API view |

Face chrome already renders the grid from `xai_grok_shell::session::ContextInfo` — **no fake pie chart**; extend the wire payload so the existing block becomes API-true.

## Scope

**In**

1. Pure aggregator: session → Face `ContextInfo` (+ model label) matching the view model above.
2. Wire `x.ai/session/info` through that aggregator (same path `/context` and session-info already use).
3. Unit tests: compact boundary reduces message tokens; system/tools/skills populate; wire JSON camelCase shape.
4. Short plan doc (this file).

**Out**

- Parallel Face work (plan-execute-gate, agent-team, background-tasks, AskUser).
- Full async MCP tool enumeration / token-counting API (Claude’s per-tool API count).
- Redesigning `ContextInfoBlock` layout (already grid+legend).
- Legacy TUI `/context` text dump rewrite.

## Implementation sketch

```
face_context_viz.rs
  aggregate_api_view_context(session) -> FaceContextSnapshot
    - resolve working_dir, model, context window
    - build system prompt + SkillInfo list → system_prompt_tokens, skills category
    - skip compacted_count; add summary_text chars → message_tokens
    - tool estimate → tool_definitions_*
    - used / free / usage_pct / auto_compact_threshold_percent (85)
session_info_payload
  - serialize Face ContextInfo into existing SessionInfoResponse shape
```

## Verification

- Unit tests in aggregator module.
- `cargo test -p next-code --lib face_context` (or path filter).
- Rebuild+install from worktree when feasible; smoke: open Face session, a few turns, `/context` shows system/messages/tools/free with real window total (not `used/used`).

## Done when

`/context` after conversation shows breakdown aligned with compact-aware history + system + tools + skills, not cumulative billing totals as a fake 100% pie.
