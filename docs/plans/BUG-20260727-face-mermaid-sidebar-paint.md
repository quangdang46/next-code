# BUG: Face mermaid sidebar paint wrong vs origin

**Status:** implemented on `face-mermaid-sidebar`  
**Branch:** `face-mermaid-sidebar`  
**Risk:** medium (Kitty/iTerm placement; side-panel layout)

## Summary

`SidePanelState::Diagram` dumps unpositioned `render_kitty_image` (`a=T`, no CUP). Image does not land in the right-hand column. Origin side-panel mermaid + Face inline media both CUP to a `Rect` and `fit_image_to_cells`.

## Verified root cause

1. `crates/xai-grok-pager/src/views/side_panel.rs` Diagram arm → `render_kitty_image` without `\x1b[y;xH` (verified).
2. Face inline path correctly uses `place_inline_image` + CUP (`agent_view/media.rs`).
3. Origin `ui_pinned.rs` paints mermaid with fit/viewport into content `Rect`.
4. Sidebar render hardcodes `.min(36)` instead of live `side_panel_width`.

## Fix (copy)

| From | Into |
|------|------|
| `place_inline_image` / `fit_image_to_cells` | Diagram paint in `side_panel.rs` |
| Origin fit sizing | `side_panel_height` Diagram + scroll crop |
| Live panel width | `request_mermaid_sidebar_render` cols |
| Focus/visible stamp | `show_diagram_in_side_panel` |

## Out of scope (follow-on)

Full origin Crop (replace chat ASCII with sidebar-only placeholder) — paint must work first.
