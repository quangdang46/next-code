# NOTICE — xai-crash-handler

Facade of `xai-org/grok-build` `xai-crash-handler` (Apache-2.0) for the next-code
Grok Face migration (PR7).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-crash-handler

## Role in next-code

Upstream is a full SIGSEGV/SIGBUS crash handler with symbolication. This stub only
covers the pager import sites: `enable_terminal_escape_restore` /
`disable_terminal_escape_restore` (no-ops) and `terminal::{MOUSE_PASTE_RESET,
MOUSE_TRACKING_RESET, RESTORE_SEQ}` (exact upstream CSI byte constants).

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
