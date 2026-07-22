# Deepen — Bundle Active/Blocked counts honesty (2026-07-22)

**Priority:** P1 · Phase 1  
**Parent:** Master plan Phase 1 §3  
**Paired with:** [`20260722-bundle-hooks-mcp-merge.md`](./20260722-bundle-hooks-mcp-merge.md)

---

## Problem

Face Plugins UI counts bundle `hooks/hooks.json` and `.mcp.json` as if live. Runtime does **not** merge them into `next-code-hooks` / MCP client → **UI lie**.

## Decision rule (pick one per surface — do not mix silently)

| Option | Meaning |
|--------|---------|
| **A — Wire** | Merge into real registries (see merge deepen). Counts = runtime. |
| **B — Demote** | Stop showing Active/Blocked as live; label “declared / not loaded” or hide counts until wired. |

**Phase 1 default recommendation:** **B for hooks+MCP counts immediately** if merge slips; land **A** in same phase if cheap.

## Exit criteria

- [ ] No Face string implies runtime load for unmerged bundle hooks/MCP.
- [ ] If A: counts match registry after enable+trust gates.
- [ ] Docs (`plugins.md` / Face copy) match chosen option.
