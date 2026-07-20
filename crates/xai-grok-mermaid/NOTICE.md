# NOTICE — xai-grok-mermaid

Facade of `xai-org/grok-build` `xai-grok-mermaid` (Apache-2.0) for the next-code
Grok Face migration (PR7).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-grok-mermaid

## Role in next-code

Upstream renders Mermaid → SVG → PNG via a pure-Rust layout engine and optional
`mmdc` subprocess. This stub covers pager import sites only:
`MermaidTheme`, `RenderLimits`, `RenderParams`, `RenderedDiagram`,
`MermaidError`, `SubprocessError`, `default_engine`, `render_checked` (always
`Err`), and `run_with_timeout` (always `Err`). No layout engine, no fonts, no
rasterizer.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
