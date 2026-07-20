# Plan Report

## Summary (read this first)
- **You asked:** After merging PR1 into `dev`, branch PR2 and continue Grok UI migration.
- **What is going on:** PR1 (ratatui leaf crates) is on `dev`. PR2 vendors `xai-grok-pager-render` plus the minimum shim/leaf crates so the Face presentation layer compiles in next-code.
- **We recommend / did:** Keep Cargo package names `xai-*` (fewer import rewrites); shim brain deps; adapt render for crates.io **ratatui 0.28** (Grok uses 0.29).
- **Risk:** Medium (shims are intentional stubs; Windows host fails some Unix-path unit tests)
- **Status:** Implemented — `cargo check -p xai-grok-pager-render` green. Reply if you want commit/push to PR #36.

## Feature planning
- **Recommended approach:** Face render substrate only — not full Grok pager, not entrypoint swap.
- **Prior art:** [xai-org/grok-build](https://github.com/xai-org/grok-build) `SOURCE_REV` `ba69d70`
- **Integration points:** PR1 `next-code-ratatui-inline` / `textarea` via `package =` rename; new `crates/xai-*`
- **Sub-agents used:** partial (usage-limited); research continued in-parent against grok-build + crates.io ratatui

## Evidence (verified during implement)
1. Grok pager-render needs `unstable-backend-writer` on ratatui **0.29** — crates.io **0.28** has no such feature → `SharedTermWriter` + `draw_frame(..., writer, ...)` discard path
2. `tui-scrollbar` / `ratatui-core` conflict with workspace `unicode-width = 0.2.2` → proportional scrollbar rewrite
3. Upstream `user_grok_home` = `Some(grok_home())` when home or `GROK_HOME` resolves — shim corrected to match
4. `Line::from(Cow)` missing on ratatui 0.28 → explicit Borrowed/Owned match

## Steps
1. [x] Workspace members + leaf copies + shims
2. [x] Vendor pager-render; ratatui 0.28 adaptations
3. [x] `cargo check -p xai-grok-pager-render` green
4. [x] LICENSE/NOTICE on new crates
5. [ ] Commit + push PR #36 (await owner)
6. [x] Do **not** wire binary or delete old TUI

## Verification (this machine)
- `cargo check -p xai-grok-pager-render` — **0 errors**
- `cargo test -p xai-grok-pager-render --lib` — **949 passed**, **31 failed** (Windows host: Unix absolute paths in osc8/tool_paths/prompt_images; 2 glyph tests assert non-legacy console; expected gap vs Linux CI)

## Files / crates
| Crate | Role |
|-------|------|
| `xai-tty-utils`, `xai-grok-paths`, `xai-grok-markdown(-core)` | Vendored leaf |
| `xai-grok-config`, `telemetry`, `workspace`, `tools`, `shared` | Shim / slim copy |
| `xai-grok-pager-render` | Face render |

## Open for later PRs
- Pager (~374k) + ACP host + entry
- Revisit ratatui 0.29+ when unicode-width pin allows
- Unix CI for full unit-test parity
