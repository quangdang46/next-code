# Plan Report — PR11 Retire legacy next-code TUI

## Summary (read this first)
- **You asked:** Clean old UI after Face is product.
- **What is going on:** Face default; `NEXT_CODE_LEGACY_TUI` + `pub use next_code_tui::*` still ship old presentation.
- **We recommend:** This is the primary **Delete** PR. No Face rewrite. Keep `next-code-tui-anim` only.
- **Risk:** High  
- **Status:** After PR9 smoke green.

## Workflow map (required)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | — (Face already vendored) | Copy old TUI screens “just in case” |
| **Wire** | Any CLI that assumed TUI → Face or clear error | Leave silent dual UI |
| **Delete** | Legacy env path, root re-export, then `next-code-tui` dep/crates when safe | Delete `next-code-tui-anim` |

## Research first (LOOK)
1. `rg "next_code_tui|crate::tui|tui_launch|LEGACY_TUI"` outside `next-code-tui/**`.
2. What `pub use next_code_tui::*` still provides to root.
3. Confirm logo only needs `next-code-tui-anim`.

## Evidence (fill before BUILD)

| Claim | Citation | Status |
|-------|----------|--------|
| Legacy hatch in dispatch | `src/cli/dispatch.rs` / `pager_launch.rs` | unverified — needs line |
| Root re-exports TUI | `src/lib.rs` | verified (pre-audit) |
| Pager depends on anim only | `xai-grok-pager/Cargo.toml` | verified (pre-audit) |

## Copy / wire / delete
| Action | What |
|--------|------|
| **Delete** | `NEXT_CODE_LEGACY_TUI` (or feature default off) |
| **Delete** | `pub use next_code_tui::*` / root dep |
| **Keep** | `next-code-tui-anim` |
| **Wire** | Leftover commands → Face |

## Implementation steps
1. [ ] Remove/gate legacy branch.  
2. [ ] Break re-export; relocate survivors.  
3. [ ] Drop crate dep; optional full crate delete (or PR11b).  
4. [ ] Docs AGENTS/README.  
5. [ ] CI without required old TUI e2e.

## Manual verify
Face-only entry; logo OK; `cargo check -p next-code`.

## Open questions
1. One-release feature flag vs hard delete?  
2. Which non-UI modules still need TUI crates (video_export)?  

## Out of scope
Slash brand (PR10), stub→real (PR12).

## Done when
Old interactive TUI is not a supported product path.
