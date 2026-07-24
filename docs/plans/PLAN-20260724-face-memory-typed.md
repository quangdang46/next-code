# PLAN — Face `/memory` typed memory (Claude memdir taxonomy)

**Date:** 2026-07-24  
**Branch:** `pr-face-memory-typed`  
**Worktree:** `C:\Users\ADMIN\Documents\Projects\next-code-worktrees\face-memory-typed`  
**Status:** Implemented in this PR (user requested implement + PR)

## Summary

Bring Face `/memory` closer to Claude Code’s typed memory browser: closed taxonomy (`user` / `feedback` / `project` / `reference`), browse + `$EDITOR` edit, and surface next-code **notepad** tiers instead of inventing a second store.

## Evidence

| Claim | Source |
|-------|--------|
| Claude memdir types | `.tmp-research-plugins/claude-code/src/memdir/memoryTypes.ts` — `MEMORY_TYPES` |
| Claude `/memory` opens selector → `$EDITOR` | `src/commands/memory/memory.tsx` + `MemoryFileSelector.tsx` |
| Face had browse modal but no local `/memory` slash; `OpenMemoryModal` sent prompt expecting ACP `MemoryFiles` that nothing produced | `dispatch/router.rs` (pre-change), `notification.rs` |
| next-code notepad tiers | `crates/next-code-base/src/notepad.rs` — priority / working / manual |

## Copy / wire / delete

| Kind | What |
|------|------|
| **Wire** | Local Face `/memory` slash → `Action::OpenMemoryModal` → scan disk catalog |
| **Wire** | Enter / `e` → `SuspendForEditor` (Claude edit path) |
| **Reuse** | Notepad files under `<cwd>/.next-code/notepad/` |
| **Reuse** | Existing `MemoryBrowser` modal chrome |
| **No delete** | Legacy Global/Workspace/Sessions grouping kept as fallback for untyped files |

## Files

- `crates/xai-grok-pager/src/views/memory_typed.rs` — taxonomy + catalog
- `crates/xai-grok-pager/src/views/memory_modal.rs` — typed sections + Enter edit
- `crates/xai-grok-pager/src/slash/commands/memory.rs` — `/memory` slash
- `crates/xai-grok-shell/.../notification.rs` — optional `memory_type` on `MemoryFileInfo`
- `dispatch/router.rs` — open modal from local catalog

## Smoke

```
/memory          → modal with Notepad · * sections
Enter on a row   → $EDITOR / $VISUAL
/memory on|off   → forwarded to agent session
```

## Non-goals

Bash mode, sticky prompt, permission cards (owned by other parallel agents).
