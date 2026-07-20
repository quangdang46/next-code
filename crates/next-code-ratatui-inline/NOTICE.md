# NOTICE — next-code-ratatui-inline

Adapted from `xai-org/grok-build` crate `xai-ratatui-inline` (Apache-2.0).

## Upstream provenance

`xai-ratatui-inline` includes code derived from the Ratatui terminal UI library.

Ratatui is dual-licensed under the MIT License and the Apache License, Version 2.0.
This crate's forked Terminal / viewport implementation is based on Ratatui source
(see README "Why Fork ratatui's Terminal?" and comments in `src/terminal.rs`).

Upstream Ratatui: https://github.com/ratatui/ratatui
Copyright (c) 2023-2024 The Ratatui Developers
Copyright (c) Florian Dehau (original tui-rs lineage, where applicable)

The remainder of the Grok crate is Copyright 2023-2026 xAI and licensed under the
Apache License, Version 2.0.

Grok source: https://github.com/xai-org/grok-build
Path: `crates/codegen/xai-ratatui-inline/`

## Modifications in next-code

- Package rename: `xai-ratatui-inline` → `next-code-ratatui-inline`
- Edition set to 2024 for workspace compatibility
- Dependencies pinned to crates.io versions (crossterm 0.28, ratatui 0.28)
- `scrolling-regions` declared as a local empty feature (crates.io ratatui 0.28
  has no `scrolling-regions`; default path uses the non-fork backend)
- Unit-test imports keep `crate::` (not the package name)

## Deferred to a later PR (not required for pager API surface)

- `benches/bench.rs`, `examples/inline.rs`
- Integration test `tests/segment_differential.rs` (needs termwiz / ansi-width)

Full license text: `LICENSE` / `LICENSE-Apache-2.0`.
