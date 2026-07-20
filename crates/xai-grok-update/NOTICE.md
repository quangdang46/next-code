# NOTICE — xai-grok-update

Facade of `xai-org/grok-build` `xai-grok-update` (Apache-2.0) for the next-code
Grok Face migration (PR7).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-grok-update

## Role in next-code

Upstream checks channel pointers and downloads CLI updates. This stub only
covers pager import sites: `channel_label()` (always `""`) and
`auto_update::UpdateAvailable { latest_version }`.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
