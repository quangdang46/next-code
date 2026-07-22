# Deepen — Merge bundle hooks/MCP into registries (2026-07-22)

**Priority:** P1 · Phase 1 (or honest demote — see counts deepen)  
**Parent:** Master plan Phase 1 §3  
**Evidence:** [inventory](../20260722-nextcode-extension-inventory.md) GAP rows for `hooks/hooks.json` / `.mcp.json`

---

## Goal

When a plugin is **enabled** (+ trust if project-exec), its declared hooks/MCP become **real** runtime entries — or we refuse to pretend (counts honesty).

## Hooks path

1. Parse bundle `hooks/hooks.json` (Grok-shaped) → map event names via existing OpenCode alias table (`docs/HOOKS.md`).
2. Compile into `next-code-hooks` registry as handlers with provenance `plugin:<id>`.
3. Respect enable-state + trust gate before spawn.
4. Reload semantics: see [`20260722-hook-registry-reload.md`](./20260722-hook-registry-reload.md).

**Do not** invent a third parallel runtime — OpenCode research: compile into existing dispatcher.

## MCP path

1. Read bundle `.mcp.json` (or package mcp resources).
2. Merge into MCP client config layers with provenance.
3. Gate by enable-state; optional MCP trust flag.

## Failure modes

| Issue | Behavior |
|-------|----------|
| Invalid JSON / unknown event | Skip + log; do not crash session |
| Duplicate server name | Deterministic precedence (user > project > plugin) documented |
| Disabled plugin | Strip all contributed handlers/servers |

## Exit criteria

- [ ] Enabled plugin with hooks.json → PreToolUse fires via next-code-hooks.
- [ ] Enabled plugin with .mcp.json → tools visible to model (or explicit merge error).
- [ ] Disabled → neither present.
- [ ] Face counts match (with honesty deepen).
