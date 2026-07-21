# Plan Report — PR15 Face mermaid / image engine (un-stub)

## Summary (read this first)
- **You asked:** Dedicated PR so Face Mermaid Open / Copy / expand work like stock Grok (not folded into PR12 stub→shell, PR14 cleanup, or floats #45).
- **What is going on:** Stock Face already has Kitty/iTerm media + mermaid affordance UI (`[Open Image]` / `[Copy Image Path]` / `[Copy Source]`) and `mermaid_worker` (lazy render → PNG cache → open/copy). Our `xai-grok-mermaid` is still the **PR7 StubEngine** — `default_engine()` / `render_checked` always `Unsupported`. Diagrams scroll-float stays text-only until a real PNG pipeline exists.
- **We recommend:** **Copy** the real `xai-grok-mermaid` engine (+ `third_party/mermaid-to-svg` if required) from grok-build; **Wire** `default_engine()` → `PureRustEngine` and ensure the next-code binary intercepts `__mermaid-render` like stock pager `main`. Do **not** rewrite Face UI. Diagrams float image paint = **follow-on** after this engine lands.
- **Risk:** Medium–High (large vendor surface: layout + fonts + resvg; Windows subprocess/re-exec; binary must honor hidden subcommand)
- **Status:** Implemented (copy → wire) — reply for smoke / merge when ready

## Feature planning
- **Recommended approach:** Replace the PR7 façade body with upstream mermaid → SVG → PNG (`PureRustEngine` + `rasterize`; optional `MmdcEngine` only if we copy it and keep it non-default). Keep the public types the pager already imports. Wire crash-isolated child render: call `xai_grok_pager::app::mermaid_worker::maybe_run_render_subprocess()` at the top of `next-code` `main` (or `pager_launch` before `app::run`) so re-exec of `next-code __mermaid-render …` works when Face is embedded.
- **Prior art (GitHub / DeepWiki):** [xai-org/grok-build](https://github.com/xai-org/grok-build) — `crates/codegen/xai-grok-mermaid` (`pure.rs`, `raster.rs`, `mmdc.rs`, `engine.rs`, `subprocess.rs`); pager `mermaid_worker` + `third_party/mermaid-to-svg`. DeepWiki: `default_engine()` → `PureRustEngine` (verified via wiki answer 2026-07-21).
- **Integration points:** `crates/xai-grok-mermaid/**` → real engine; workspace `Cargo.toml` deps; optional `third_party/mermaid-to-svg`; `src/main.rs` / `pager_launch.rs` → early `__mermaid-render` gate; Face UI already in `media.rs` / `mermaid_content.rs` (no rewrite).
- **Sub-agents used:** skipped — LOOK verified locally + DeepWiki; scope is vendor/wire, not multi-surface product design.
- **Option B:** Thin wrapper crate that path-deps a git subtree of upstream mermaid only (heavier sync; prefer full replace of our façade tree to match NOTICE SOURCE_REV discipline).
- **Open questions:**
  1. Pin upstream SOURCE_REV to current NOTICE (`ba69d70`) or newer grok-build tip?
  2. Ship optional `mmdc` engine in-tree, or PureRust-only for v1?
  3. Accept new heavy deps (resvg/usvg/tiny-skia + font bundle) in workspace without feature-gate, matching stock?

## Workflow map (required)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | grok-build `xai-grok-mermaid` sources + `third_party/mermaid-to-svg` (and NOTICE/LICENSE) | Reimplement layout in next-code; invent PNG pipeline |
| **Wire** | `default_engine()` → real engine; `render_checked` / `run_with_timeout` real bodies; early `__mermaid-render` in next-code entry | Change affordance/Kitty UI; fold into PR12 shell stubs |
| **Delete** | Stub-only comments/NOTICE “always Err” once replaced | Delete Face mermaid UI or floats PR work |

## Evidence

| Claim | Status | Citation |
|-------|--------|----------|
| Stub always fails render | **verified** | `crates/xai-grok-mermaid/src/lib.rs` — `StubEngine::render` / `default_engine` / `render_checked` → `MermaidError::Unsupported` |
| PR7 façade role | **verified** | `crates/xai-grok-mermaid/NOTICE.md` — stub, no layout/fonts/rasterizer; SOURCE_REV `ba69d70` |
| Face affordance + click Open/Copy | **verified** | `crates/xai-grok-pager/src/app/agent_view/media.rs` — `paint_diagram_affordances`, `on_mermaid_affordance_click`; `mermaid_content.rs` — `AffordanceKind::{Open,CopyPath,CopySource}` |
| Kitty/iTerm + image viewer present | **verified** | `media.rs` — Kitty/iTerm escape builders, `image_viewer` key handling; `agent_view/mod.rs` — `image_viewer: Option<ImageViewerState>` |
| Worker calls stub engine | **verified** | `mermaid_worker.rs` — `render_source_to_png` → `render_checked(default_engine()…)` |
| Out-of-process child API exists, embed gate missing | **verified** | `maybe_run_render_subprocess` in `mermaid_worker.rs`; **no** callers under `src/` (`pager_launch` → `app::run` only) |
| Stock engine shape | **verified** (DeepWiki) | grok-build `PureRustEngine` / `pure.rs` + `raster.rs` + vendored `mermaid-to-svg`; optional `mmdc.rs` |
| No `third_party/` in next-code yet | **verified** | workspace root — no `third_party` dir |
| Diagrams float deferred on engine | **verified** | `PLAN-20260721-face-info-widget-floats.md` — Diagrams text interim until mermaid image pipeline |
| Not PR12 / PR14 / #45 | **verified** | PR12 = shell/workspace stubs; PR14 = parity cleanup; #45 = info floats |

## Steps (simple checklist)
1. [x] Confirm upstream tree at chosen SOURCE_REV (DeepWiki + raw paths / local grok-build mirror if available).
2. [x] **Copy** `xai-grok-mermaid` real modules + `third_party/mermaid-to-svg` (NOTICE update).
3. [x] Workspace deps: match upstream Cargo.toml (resvg stack, fonts, etc.); `cargo check -p xai-grok-mermaid` + `-p xai-grok-pager`.
4. [x] **Wire** next-code entry: early `maybe_run_render_subprocess()` so `next-code __mermaid-render` child works (stock parity).
5. [x] Unit: engine render of a tiny flowchart → PNG bytes; worker tests still pass.
6. [ ] Manual smoke (Kitty or iTerm if available; else Open → OS viewer + Copy path): mermaid in transcript → Open / Copy path / Copy source; stub error gone.
7. [x] Document Diagrams float follow-on (image paint) — do **not** block this PR on floats.
8. [x] Rebuild/install both `next-code` / `nextcode` aliases after BUILD approval.

## Files to touch (BUILD — after go ahead)
- `crates/xai-grok-mermaid/**` — replace StubEngine with real engine
- `third_party/mermaid-to-svg/**` (+ `dagre_rust` / `graphlib_rust` / `ordered_hashmap`) — vendored
- Root / crate `Cargo.toml` — deps + workspace members
- `src/main.rs` — `__mermaid-render` intercept via `maybe_run_render_subprocess`
- `docs/plans/PLAN-20260720-grok-post-pr8-roadmap.md` / SUMMARY — already note PR15 (this plan)

## Out of scope
- PR12 shell/git/trust stubs  
- PR14 dead-stub deletion sweep  
- Floats #45 Diagrams image float registration (follow-on)  
- `GrokHost` rewrite, voice, grok.com sync  

## Done when
- `default_engine().render(…)` produces PNG for a simple diagram  
- Face Open / Copy path succeed without “stub: rendering not available”  
- Child `__mermaid-render` path works from the installed next-code binary  
- Kitty/iTerm / image-viewer paths unchanged aside from receiving real PNGs  

## Status
**Implemented** (copy → wire). SOURCE_REV `a881e6703f46b01d8c7d4a5437683546df30449d` (grok-build `main` tip; NOTICE pin `ba69d70` was stale/missing). Diagrams float image paint remains follow-on after #45. Rebuild/install done (`52692e609`). Child smoke: installed `next-code __mermaid-render` → valid PNG. Manual Face Open/Copy UI smoke still operator-side.
