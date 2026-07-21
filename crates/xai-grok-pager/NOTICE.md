# NOTICE — xai-grok-pager

Vendored from `xai-org/grok-build` `xai-grok-pager` (Apache-2.0) for the next-code
Grok Face migration (PR7).

Upstream: https://github.com/xai-org/grok-build
Upstream path: crates/codegen/xai-grok-pager (~433 `.rs` / ~374k LOC)

## Adaptations for next-code

- Cargo deps use path crates + `package =` renames for `next-code-ratatui-*`.
- `xai-grok-voice` without `audio` feature (PR6 `AUDIO_SUPPORTED = false`).
- `build.rs` defines `VERSION_WITH_COMMIT` from `CARGO_PKG_VERSION`.
- Benches / PTY e2e test harnesses / playground bins dropped from this
  `Cargo.toml` (sources may still exist under `src/bin/` unused).
- Does **not** replace `next-code-tui` or change the `next-code` binary (PR8).

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
