# NOTICE — xai-grok-agent

Compile stub of `xai-org/grok-build` `xai-grok-agent` (Apache-2.0) for next-code Grok UI
migration (PR5).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-grok-agent (~30 files)

## Role in next-code

Upstream is a full agent builder/discovery/plugin-marketplace crate. This stub only covers
the handful of types/functions the future pager imports (`views/agents_modal.rs`,
`plugin_cmd.rs`): `config::{BuiltinAgentName, AgentDefinition, AgentScope, PromptMode}`,
`discovery::discover`, and `plugins::{install_registry, manifest, git_install}` DTOs /
parsing helpers. Function bodies are empty/no-op placeholders (no real filesystem
discovery, no real git clone) — same stub convention as PR3/PR4
(see `xai-grok-workspace/src/file_system/fuzzy.rs`). Full agent-builder logic
(`builder.rs`, `compaction.rs`, `system_reminder.rs`, marketplace/trust/hooks_adapter) is
NOT vendored.

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
