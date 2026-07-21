# Plan Report — Face conversation click-to-expand

## Summary (read this first)
- **You asked:** Keep rich agent detail (tools / thinking / memory / tokens) but improve conversation UX — **collapsed by default, expand on click**.
- **What is going on:** Face already has a full fold system (`DisplayMode`, `e` / `E` / `Ctrl+E`, double-click). Edits often stayed Expanded because the shell resolve stub forced `collapsed_edit_blocks=false`; memory tools lacked Face `Memory search:` titles.
- **We recommend / shipped:** Stock Face fold UX — denser resting defaults + memory title wire. No Cursor-style cards; no fold re-home into `next-code-tui`.
- **Risk:** Medium (default density change for edits)
- **Status:** Implemented — denser resting transcript via shell resolve stub + memory title wire. Stock double-click / `e` / `E` / `Ctrl+E` kept. No persisted prefs. PR: https://github.com/quangdang46/next-code/pull/48

## Product decisions (approved)
1. Gesture: stock Grok double-click + keys (`e` / `E` / `Ctrl+E`) — no single-click expand.
2. Defaults: denser resting — tools + thinking finish → Collapsed; `resolve_collapsed_edit_blocks` default **true**.
3. Prefs: session sticky only.
4. Memory: Face `Memory search:` title convention (ACP has no MemorySearch kind).
5. No Cursor-style cards; no fold re-home into `next-code-tui`.

## Evidence
1. Stock Face keys: `ToggleFold`=`e`, `ToggleExpandAll`=`E`, `ExpandAllThinking`=`Ctrl+E`.
2. Mouse: double-click toggles fold; single click selects.
3. Thinking: live Truncated; finish → sticky `thinking_display_mode` (default Collapsed).
4. Edits: `edit_default_display_mode` respects `collapsed_edit_blocks`; shell stub now resolves **true**.
5. Memory: `pager_agent` emits `Memory search:` titles → Face `MemorySearchToolCallBlock`.

## Files touched
- `crates/xai-grok-shell/src/util/config.rs` — `resolve_collapsed_edit_blocks` default true
- `src/cli/pager_agent.rs` — memory `Memory search:` titles
- `docs/plans/PLAN-20260721-face-conversation-expand.md` — status

## Follow-ups
- Operator smoke after rebuild/install both aliases + restart serve.
- MemoryActivity float is a separate plan (`PLAN-20260721-face-info-widget-floats.md`).
