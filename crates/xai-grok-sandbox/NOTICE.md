# NOTICE — xai-grok-sandbox

Facade of `xai-org/grok-build` `xai-grok-sandbox` (Apache-2.0) for the next-code
Grok Face migration (PR7).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-grok-sandbox

## Role in next-code

Upstream applies Landlock/Seatbelt/bwrap sandboxing. This stub only covers pager
import sites: `profile_name` (always `None`), `ProfileName` (parse/Display
matching upstream), and `sandbox_profile_conflicts` (always empty).

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
