# NOTICE — xai-grok-mermaid

Originally vendored from `xai-org/grok-build` `xai-grok-mermaid` (Apache-2.0) for
the next-code Grok Face migration (PR15 un-stub).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: a881e6703f46b01d8c7d4a5437683546df30449d
Upstream path: crates/codegen/xai-grok-mermaid

## Role in next-code

Default (and only) offline engine is [`MmdrEngine`]: Mermaid → PNG via
[`quangdang46/mermaid-rs-renderer`](https://github.com/quangdang46/mermaid-rs-renderer)
(`render_png_bytes`, Face light/dark themes, secure bundled-font raster). Pin:
git rev `baad3f0695ca2a3a5cf613ff723576ea55fd8ec7` (package 0.3.2+, Face embed API).

Optional [`MmdcEngine`] (external `mmdc` CLI) remains available but is not selected
by [`default_engine`]. The former dagre/`mermaid-to-svg` PureRust path was removed.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI
where modified (Cargo path pins, workspace membership, next-code `__mermaid-render` wire).
