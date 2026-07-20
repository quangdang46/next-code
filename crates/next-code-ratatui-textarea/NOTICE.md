# NOTICE — next-code-ratatui-textarea

Adapted from `xai-org/grok-build` crate `xai-ratatui-textarea` (Apache-2.0).

## Upstream provenance

`xai-ratatui-textarea` is first-party code maintained by xAI for the Grok TUI.

Copyright 2023-2026 xAI
Licensed under the Apache License, Version 2.0.

Grok source: https://github.com/xai-org/grok-build
Path: `crates/codegen/xai-ratatui-textarea/`

This crate is not a wholesale redistribution of third-party source. Dependencies
such as ratatui are declared in `Cargo.toml`.

## Modifications in next-code

- Package rename: `xai-ratatui-textarea` → `next-code-ratatui-textarea`
- Edition set to 2024 for workspace compatibility
- Dependencies pinned to crates.io versions (crossterm 0.28, ratatui 0.28)
- Removed `ratatui-core` and `tui-scrollbar` (replaced with inline scrollbar
  render against `ratatui::buffer::Buffer`)
- Retained full editor API surface: `editor.rs`, `editor_keys.rs`, `editor_tests/`
- Feature `debug-logs` retained (`dep:tracing`)
- Crossterm `bracketed-paste` feature enabled to match upstream
- Dev-dependencies for unit tests: `rand` 0.9, `chrono`, `itertools`,
  `pretty_assertions`

## Deferred to a later PR (not required for pager API surface)

- `examples/textarea_demo.rs` (needs extra playground deps)
- Upstream `benches/` / integration `tests/segment_differential.rs` on the
  inline crate (criterion / termwiz)

Full license text: `LICENSE` / `LICENSE-Apache-2.0`.
