# Deepen — Project trust gate for executable resources (2026-07-22)

**Priority:** P0 · Phase 1  
**Parent:** Master plan Phase 1 §1  
**Steal from:** Pi `trust.json` + herdr install preview ([pi surfaces §2.8](../20260722-pi-extension-surfaces.md), [herdr ABI](../20260722-herdr-multilang-abi.md))

---

## Problem

Executable project hooks/plugins = RCE. Marketing “any language extensions” without a trust gate is unsafe.

## Frozen intent

Before loading **executable** project-scoped resources, host must have an explicit trust decision for that project tree (or ancestor), analogous to Pi’s `TRUST_REQUIRING_PROJECT_CONFIG_RESOURCES`.

### Trust-requiring (v1)

| Resource | Why |
|----------|-----|
| Project `.next-code/hooks.toml` handlers with `type = "command"` / argv | Spawn |
| Project `.next-code/plugins/**` with executable runners / future package `[[hooks]]` argv | Spawn |
| Future package install with `[[build]]` / argv tools | Spawn |

### Not trust-requiring alone

- Declarative skills markdown, prompt templates, theme tokens (data-only).
- User-global `~/.next-code/hooks.toml` (user already chose home install).
- MCP stdio servers: optional separate `--require-mcp-trust` (inventory already notes).

## Resolution order (sketch)

1. CLI override: `--approve-project-trust` / `--deny-project-trust` (non-interactive).
2. Nearest saved trust store entry (path TBD: `~/.next-code/trust.json` or project `.next-code/trust.json` — **open: prefer user-home store like Pi**).
3. Interactive Face/CLI prompt once per project (session may require restart to apply — match Pi lesson).
4. Default: **ask** (never silent-allow executables from project).

## UX

- Face: one clear prompt + remember choice.
- `/trust` or settings entry to revisit.
- Logs: structured “skipped project hook — untrusted.”

## Exit criteria

- [ ] Untrusted project: command hooks in project layer **do not run**.
- [ ] Trusted project: same hooks run as today.
- [ ] Non-interactive CI path documented with explicit flags.

## Out of scope

Sandboxing tools after trust; WASM; replacing DCG permissions.
