# Deepen designs — platform Phase 1–4 tickets (2026-07-22)

Frozen / near-frozen design notes that turn the master plan into implementable contracts.

**Master:** [`../../PLAN-20260722-pi-full-custom-platform.md`](../../PLAN-20260722-pi-full-custom-platform.md)  
**Readiness gate:** [`../../PLAN-20260722-platform-implement-readiness.md`](../../PLAN-20260722-platform-implement-readiness.md)  
**Research evidence:** [`../README.md`](../README.md)  
**Audit:** [`20260722-docs-completeness-audit.md`](./20260722-docs-completeness-audit.md)

## Index (D1–D13 + audit)

| ID | Priority | File | Topic |
|----|----------|------|-------|
| D0 | P0 | [`20260722-bare-host-no-prompt-inject.md`](./20260722-bare-host-no-prompt-inject.md) | Bare/platform no opinionated prompt inject; `system_prompt.md` → pack |
| D1 | P0 | [`20260722-trust-gate-design.md`](./20260722-trust-gate-design.md) | Project trust before exe hooks/plugins |
| D2 | P0 | [`20260722-plugins-state-skill-gate.md`](./20260722-plugins-state-skill-gate.md) | Enable-state gates skill ingest |
| D3 | P1 | [`20260722-bundle-counts-honesty.md`](./20260722-bundle-counts-honesty.md) | Stop UI lies on Active/Blocked |
| D4 | P1 | [`20260722-bundle-hooks-mcp-merge.md`](./20260722-bundle-hooks-mcp-merge.md) | Wire bundle hooks/MCP into registries |
| D5 | P1 | [`20260722-hooks-cookbook-layout.md`](./20260722-hooks-cookbook-layout.md) | Bash/Python/Node PreToolUse cookbook |
| D6 | P1 | [`20260722-host-callback-bin-path.md`](./20260722-host-callback-bin-path.md) | `NEXT_CODE_BIN_PATH` |
| D7 | P1 | [`20260722-hook-registry-reload.md`](./20260722-hook-registry-reload.md) | Reload after package compile-in |
| D8 | P2 | [`20260722-plugin-manifest-abi-v1.md`](./20260722-plugin-manifest-abi-v1.md) | Manifest ABI v1 |
| D9 | P2 | [`20260722-package-slash-acp.md`](./20260722-package-slash-acp.md) | Package slash via ACP |
| D10 | P2 | [`20260722-nextcode-pack-extraction.md`](./20260722-nextcode-pack-extraction.md) | Product profile / pack extraction |
| D11 | P2 | [`20260722-tools-abi-v1.md`](./20260722-tools-abi-v1.md) | `[[tools]]` argv / MCP |
| D12 | P2 | [`20260722-face-ui-hints-external-pane.md`](./20260722-face-ui-hints-external-pane.md) | Phase 4 UI ceiling |
| D13 | P2 | [`20260722-argv-plugin-security.md`](./20260722-argv-plugin-security.md) | Security beyond trust |

## Docs only

These files are **not** an implement approval. Reply **go ahead** on the master / readiness plan before production Rust.
