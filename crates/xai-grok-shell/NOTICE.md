# NOTICE — xai-grok-shell

Compile-stub façade of `xai-org/grok-build` `xai-grok-shell` (Apache-2.0) for next-code
Grok UI migration (PR5).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-grok-shell (~434 files, ~14MB)

## Role in next-code

Upstream is the full Grok shell/runtime crate (terminal, PTY, session engine, MCP dispatch,
auth, billing, memory, etc.). Vendoring it wholesale was explicitly out of scope for PR5
(Option D, deferred) — instead this is a frequency-ordered façade covering only the
highest-frequency import prefixes the future pager crate (`xai-grok-pager`, not yet
vendored — PR7) uses: `util::config` (full `RemoteSettings` DTO), `agent::config`,
`auth::{AuthMeta, GateInfo, AuthManager}`, `sampling::{types, error}`,
`extensions::{notification, session_search, mcp, task, billing}`, `config` (load/plugin
toggle helpers), `util::{grok_home, clipboard, changelog, tips}`, `session` (persistence /
worktree / storage / merge / restore / repo_changes / prompt_queue / info /
ContextInfo / PromptOrigin), `models::default_model`, `tier::is_restricted_tier_name`,
`active_sessions`.

Function bodies are empty/no-op placeholders and DTOs are `Default`-derived, not real
logic — same stub convention as PR3/PR4. `util::clipboard` and `util::with_locked_stderr`
re-export the real implementations already vendored into `xai-grok-shared` (PR2) rather
than duplicating them. `util::grok_home::grok_home` re-exports `xai-grok-config` (PR3).

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
