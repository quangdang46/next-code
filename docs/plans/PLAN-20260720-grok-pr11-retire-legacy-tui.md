# Plan Report — PR11 Retire legacy next-code TUI

## Summary (read this first)
- **You asked:** Delete / stop shipping old UI after Face is the product UI.
- **What is going on:** Face is default, but `NEXT_CODE_LEGACY_TUI=1` still launches `tui_launch`, and `src/lib.rs` `pub use next_code_tui::*` keeps the old presentation crate on the critical path. Logo already uses `next-code-tui-anim` only.
- **We recommend:** Two commits/PRs-in-one if needed: (1) remove escape hatch + `tui_launch` from default product paths; (2) stop depending on `next-code-tui` from root, keep only `next-code-tui-anim` (and any tiny extract). Full crate deletion can be PR11b if CI too large.
- **Risk:** High (hidden `crate::tui::` imports via re-export)
- **Status:** Start only after PR9 smoke green.

## Goal for this PR
Product binary no longer offers old TUI; workspace no longer needs full `next-code-tui` to build `next-code` Face.

## Research first (LOOK)
1. `rg "next_code_tui|crate::tui|tui_launch|LEGACY_TUI" src crates --glob '!**/next-code-tui/**'`
2. What root still needs from `pub use next_code_tui::*` (video_export? pickers?).
3. Anim-only dependency: `xai-grok-pager` → `next-code-tui-anim` already.

## Copy / wire / delete
| Action | What |
|--------|------|
| **Delete** | `NEXT_CODE_LEGACY_TUI` path (or keep one release behind compile feature `legacy-tui` default off) |
| **Delete** | Root re-export of entire `next-code-tui` if possible |
| **Keep** | `next-code-tui-anim` for Face logo |
| **Wire** | Any leftover CLI that assumed TUI → Face or print “use Face” |

## Implementation steps
1. [ ] Feature-gate or remove `pager_launch::legacy_tui_requested` branch in `dispatch.rs`.
2. [ ] Break `pub use next_code_tui::*`: move still-needed items to smaller crates or duplicate thin helpers under `src/`.
3. [ ] Remove root Cargo dep on `next-code-tui` if unused; leave crates in workspace for now OR delete in same PR if `cargo check -p next-code` green.
4. [ ] Update AGENTS.md / README: Face is the UI; no legacy flag (or feature-only).
5. [ ] CI: ensure no job still runs old TUI e2e as required.

## Files (expected)
- `src/cli/dispatch.rs`, `pager_launch.rs`, `tui_launch.rs`
- `src/lib.rs`, root `Cargo.toml`
- Docs: SUMMARY, AGENTS.md

## Manual verify
1. `nextcode` → Face only.
2. `NEXT_CODE_LEGACY_TUI=1` either ignored or documented feature-off.
3. Welcome logo still animates.
4. `cargo check -p next-code` without building full old TUI if deleted.

## Out of scope
- Rewriting Face
- Stub→real shell (PR12)

## Done when
Old interactive TUI is not a supported product path; Face + anim-only deps.
