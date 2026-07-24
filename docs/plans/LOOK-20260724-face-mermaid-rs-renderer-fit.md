# LOOK — Can `mermaid-rs-renderer` replace Face’s Mermaid stack?

Date: 2026-07-24  
Status: **Implemented** (Face default engine = mmdr; P2 dialect/fixture parity deferred)  
Repo under eval: `quangdang46/mermaid-rs-renderer` (merge `baad3f06`, package `0.3.2`)  
Face stack: `xai-grok-mermaid` → **`MmdrEngine`** (`render_png_bytes`) + pager `mermaid_worker`; legacy `PureRustEngine` / `third_party/mermaid-to-svg` gated behind feature `legacy-mermaid-to-svg`

---

## Verdict

**Implemented for Face Open/Copy.** mmdr’s Face embed API landed (PR [quangdang46/mermaid-rs-renderer#13](https://github.com/quangdang46/mermaid-rs-renderer/pull/13)): `render_png_bytes`, secure raster, `PngRenderParams`, `RenderError`, `Theme::face_light` / `face_dark`. Face now defaults to `MmdrEngine`; keep pager glue (`mermaid_worker`, `__mermaid-render`, timeout, cache, Open/Copy).

**P2 still deferred:** visual/fixture parity for exotic dialects vs `mermaid-to-svg`; delete `third_party/mermaid-to-svg` (+ dagre stack) once parity is acceptable. Optional A/B via `--features legacy-mermaid-to-svg`. Diagrams float image paint remains a separate Face UI follow-on.

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
| Lazy Open / Copy path / Copy source affordances | Unchanged pager UI |

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
| P2 fixture/dialect parity | **Deferred** |
| Delete `third_party/mermaid-to-svg` | **Follow-up** (feature-gated for now) |

---

## Integration (landed)

1. Pin `quangdang46/mermaid-rs-renderer` git rev `baad3f0695ca2a3a5cf613ff723576ea55fd8ec7` (`default-features = false`, `features = ["png"]`).
2. `MmdrEngine: MermaidEngine` in `xai-grok-mermaid`; `default_engine()` → `MmdrEngine`.
3. Keep `mermaid_worker` / `__mermaid-render` / timeout / cache / Open+Copy.
4. Legacy `PureRustEngine` behind `legacy-mermaid-to-svg`.

**Update (2026-07-24):** Face migration PR wires mmdr as default; LOOK closed for P0/P1.
