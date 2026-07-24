# PLAN — Face sticky prompt chrome (Claude Code–style)

**Date:** 2026-07-24  
**Branch:** `pr-face-sticky-prompt`  
**Status:** implemented (awaiting review)

## Summary

Claude Code pins a fixed 1-row `❯ preview` sticky prompt above the transcript when the user scrolls up (`StickyPromptHeader` in `FullscreenLayout.tsx`). Face already had multi-row sticky section headers (`sticky_headers`); this change adds Claude-style **sticky chrome** as the default presentation so scrolling history shows a compact breadcrumb instead of a tall pinned prompt block.

## Evidence

| Source | Finding |
|--------|---------|
| Claude `.tmp-research-plugins/claude-code/src/components/FullscreenLayout.tsx` | Fixed 1-row header; truncate-end; click jumps to prompt |
| Claude `VirtualMessageList.tsx` StickyTracker | First paragraph, whitespace collapse, 500-char cap |
| Face `scrollback/sticky.rs` + `scrollback_pane.rs` | Existing iOS-style multi-row sticky headers (default on) |
| LOOK `LOOK-20260724-claude-code-ux-gaps-for-face.md` §9 | Listed sticky prompt chrome as Claude comfort gap |

## Approach

| Kind | Change |
|------|--------|
| **Wire** | `sticky_chrome = true` (default) collapses sticky layout to 1 row and renders `❯ preview` |
| **Keep** | `sticky_chrome = false` restores Face multi-row sticky headers |
| **Nav** | Page/scroll/hit-test use chrome-aware header height |
| **Click** | Single-click pinned chrome prompt → `scroll_to_entry_top` |

## Files

- `crates/xai-grok-pager-render/src/appearance/config.rs` — `sticky_chrome` setting
- `crates/xai-grok-pager/src/scrollback/sticky.rs` — chrome helpers + tests
- `crates/xai-grok-pager/src/scrollback/scrollback_pane.rs` — chrome render
- `crates/xai-grok-pager/src/scrollback/state/{nav,layout}.rs` — chrome-aware heights
- `crates/xai-grok-pager/src/app/agent_view/selection.rs` — click-to-jump

## Smoke

1. Start Face with a multi-turn session (several user prompts + long replies).
2. Scroll up through history (wheel / PgUp) until a prior user prompt leaves the viewport top.
3. Expect a single highlighted row at the top of scrollback: `❯ <truncated prompt text>`.
4. Click that row → viewport jumps so that prompt is at the top; chrome dismisses once the prompt is visible again.
5. Optional: set `[scrollback.display] sticky_chrome = false` to restore classic Face sticky blocks.

## Risk

Low–medium. Default chrome changes sticky visual height (multi-row → 1). Classic behavior remains behind `sticky_chrome = false`.
