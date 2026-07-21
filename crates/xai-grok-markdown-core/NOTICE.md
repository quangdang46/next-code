# NOTICE — xai-grok-markdown-core

Vendored / shimmed from `xai-org/grok-build` (Apache-2.0) for next-code Grok UI migration (PR2).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70

## Role in next-code

- Leaf copies (tty-utils, paths, markdown*): largely faithful vendored sources
- Shims (config, telemetry, workspace, tools, shared subset): minimal surface for `xai-grok-pager-render`
- `xai-grok-pager-render`: Face presentation layer; ratatui 0.28 adaptations (no tui-scrollbar / no unstable-backend-writer)

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
