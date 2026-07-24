# LOOK — Can `mermaid-rs-renderer` replace Face’s Mermaid stack?

Date: 2026-07-24  
Status: **Implemented** (Face default engine = mmdr; legacy dagre stack deleted; transcript inline PNG paint landed)  
Repo under eval: `quangdang46/mermaid-rs-renderer` (merge `baad3f06`, package `0.3.2`)  
Face stack: `xai-grok-mermaid` → **`MmdrEngine`** (`render_png_bytes`) + pager `mermaid_worker` + terminal-tier inline Kitty/iTerm paint

---

## Verdict

**Implemented for Face Open/Copy and transcript inline paint.** mmdr’s Face embed API landed (PR [quangdang46/mermaid-rs-renderer#13](https://github.com/quangdang46/mermaid-rs-renderer/pull/13)): `render_png_bytes`, secure raster, `PngRenderParams`, `RenderError`, `Theme::face_light` / `face_dark`. Face defaults to `MmdrEngine` only (no `legacy-mermaid-to-svg` / `third_party/mermaid-to-svg`). Pager glue (`mermaid_worker`, `__mermaid-render`, timeout, cache, Open/Copy) plus terminal-tier PNG → scrollback `InlineMediaPlacement` (Kitty/iTerm) when overlays are active.

**Still deferred:** Diagrams *info float* image paint (`legacy_deferred.rs` TODO) — separate float UI, not transcript.

---

## What Face needs from a Mermaid engine

| Need | Where today |
|------|-------------|
| Mermaid source → PNG bytes + `width_px`/`height_px` | `xai_grok_mermaid::{MermaidEngine, RenderedDiagram}` → mmdr `RenderedPng` |
| Light/dark theme + opaque surface background | `Theme::face_light` / `face_dark` + `PngRenderParams::background` |
| Target-width / min-width / max-height / scale sizing | `RenderParams` → `PngRenderParams` |
| Source-size limit + typed errors | Face `render_checked` + mmdr `RenderError` mapped |
| Crash isolation under `panic = "abort"` | Unchanged: `mermaid_worker` + `run_with_timeout` |
| Hardened SVG→PNG | mmdr secure raster (bundled Roboto, no remote/file fetch, 32 MP / 16k) |
| Lazy Open / Copy / inline transcript paint | Open tier for OS viewer; Terminal tier + `remember_inline_png` for Kitty/iTerm |

---

## Missing checklist (post-merge)

| Item | Status |
|------|--------|
| P0 secure embed raster | **Done** in mmdr |
| P0 in-memory PNG API | **Done** (`render_png_bytes`) |
| P0 resource caps | **Done** |
| P1 sizing knobs | **Done** (`PngRenderParams`) |
| P1 Face light/dark surfaces | **Done** |
| P1 typed `RenderError` | **Done** (mapped in `MmdrEngine`) |
| Transcript inline PNG paint | **Done** (Terminal quality + existing media escapes) |
| Delete `third_party/mermaid-to-svg` (+ dagre stack) | **Done** |
| Diagrams info-float image paint | **Deferred** (float UI follow-on) |

---

## Integration (landed)

1. Pin `quangdang46/mermaid-rs-renderer` git rev `baad3f0695ca2a3a5cf613ff723576ea55fd8ec7` (`default-features = false`, `features = ["png"]`).
2. `MmdrEngine: MermaidEngine` in `xai-grok-mermaid`; `default_engine()` → `MmdrEngine` only.
3. Keep `mermaid_worker` / `__mermaid-render` / timeout / cache / Open+Copy; add Terminal-tier `ensure_mermaid_inline` → cache → `AnchoredMedia` / Kitty paint.
4. Removed `PureRustEngine`, feature `legacy-mermaid-to-svg`, and vendored dagre stack.

**Update (2026-07-24):** Face migration PR wires mmdr as default; inline paint + legacy removal landed in follow-on.
