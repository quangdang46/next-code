# NOTICE — xai-grok-plugin-marketplace

Facade of `xai-org/grok-build` `xai-grok-plugin-marketplace` (Apache-2.0) for the
next-code Grok Face migration (PR7).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-grok-plugin-marketplace

## Role in next-code

Upstream clones marketplace git caches, scans catalogs, and installs plugins via
`xai-grok-agent`'s install registry. This stub covers pager import sites only:
`OFFICIAL_SOURCE_*`, `is_official_source_url`, `MarketplaceSource` / `SourceKind`,
`load_sources` / `load_extra_sources_from_settings` / `scan_marketplace` (empty),
plus stub modules `git`, `install_resolve`, `installer`, `matcher` that return
`Err` / `None` / empty.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
