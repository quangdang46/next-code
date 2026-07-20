# NOTICE — xai-fast-worktree

Facade of `xai-org/grok-build` `xai-fast-worktree` (Apache-2.0) for the next-code
Grok Face migration (PR7).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-fast-worktree

## Role in next-code

Upstream creates CoW git worktrees and tracks them in SQLite. This stub covers
pager import sites only: `ENOSPC_OS_MESSAGE` / `OUT_OF_DISK_CONTEXT`, and
(under the empty `metadata` feature) `db::WorktreeDb::open_default` (always
`Err`), `WorktreeRecord`, `WorktreeKind`, `WorktreeStatus`.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
