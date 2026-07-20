# NOTICE — xai-acp-lib

Vendored (almost wholesale) from `xai-org/grok-build` (Apache-2.0) for next-code Grok UI
migration (PR5).

Upstream: https://github.com/xai-org/grok-build
SOURCE_REV: ba69d70
Upstream path: crates/codegen/xai-acp-lib

## Role in next-code

This is the ACP (Agent Client Protocol) channel/message crate the future pager depends on
heavily: `AcpAgentTx`/`AcpAgentRx`, `AcpClientTx`/`AcpClientRx`, the `acp_send` round-trip
helper, and the `AcpAgentMessage`/`AcpClientMessage` enums. Adapted only where dependency
versions/paths needed to fit this workspace (same approach as PR2's ratatui 0.29 -> 0.28
adaptation) — the module logic itself is unchanged from upstream.

This PR (PR5) does **not** wire `AcpAgentTx`/channels into `next-code-agent-runtime` or
`next-code-app-core`'s Registry — that remains PR8 (`GrokHost`).

Copyright 2023-2026 xAI (upstream). next-code adaptations copyright SpaceXAI where modified.
