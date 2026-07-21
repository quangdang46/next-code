# NOTICE — xai-grok-mermaid

Vendored from `xai-org/grok-build` `xai-grok-mermaid` (Apache-2.0) for the
next-code Grok Face migration (PR15 un-stub).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: a881e6703f46b01d8c7d4a5437683546df30449d
Upstream path: crates/codegen/xai-grok-mermaid

## Role in next-code

Renders Mermaid → SVG → PNG via the offline [`PureRustEngine`] (vendored
`third_party/mermaid-to-svg` + bundled Roboto + resvg rasterize). Optional
[`MmdcEngine`] is present but not selected by [`default_engine`].

Companion vendored layout stack (see `third_party/NOTICE`):
`mermaid-to-svg`, `dagre_rust`, `graphlib_rust`, `ordered_hashmap`.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI
where modified (Cargo path pins, workspace membership, next-code `__mermaid-render` wire).
