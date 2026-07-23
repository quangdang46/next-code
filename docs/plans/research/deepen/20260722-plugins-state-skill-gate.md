# Deepen — `plugins-state.json` must gate skill ingest (2026-07-22)

**Priority:** P0 · Phase 1  
**Parent:** Master plan Phase 1 §2  
**Evidence:** [inventory](../20260722-nextcode-extension-inventory.md) — enable/disable is Face-list-only; `SkillRegistry::load_global` loads all `plugins/*/skills`.

---

## Problem

UI shows plugins disabled; runtime still injects their skills → lying product + surprise tool/prompt surface.

## Frozen intent

`plugins-state.json` (and equivalent enable flags) is **authoritative for ingest**, not cosmetic.

### Must gate when disabled

1. **Skills** under that plugin tree (Phase 1 required).
2. Later (same mechanism): bundle MCP contributions, package hooks compile-in, package slash.

### Must still work when enabled

Discovery, list, enable toggle, skill → ACP `AvailableCommandsUpdate` as today.

## Design sketch

```text
discover_plugins()
  → filter by plugins-state.enabled == true
  → SkillRegistry.load only from enabled roots
  → Face list may still show disabled entries (greyed) without loading content
```

## Edge cases

| Case | Behavior |
|------|----------|
| Missing state entry | Default **enabled** (today’s UX) — document; optional future “default deny for project plugins” |
| Partial path / renamed plugin | Match by plugin id from `plugin.json`, not fragile folder name alone if id exists |
| Global vs project overlay | Disable applies to that install record |

## Exit criteria

- [ ] Disable plugin → its skills absent from registry + Face slash.
- [ ] Re-enable → skills return without restart if reload path exists; else document restart.
- [ ] Unit/integration test on load filter.

## Related

- [`20260722-bundle-hooks-mcp-merge.md`](./20260722-bundle-hooks-mcp-merge.md) — same state should later gate hooks/MCP merge.
- [`20260722-bare-host-no-prompt-inject.md`](./20260722-bare-host-no-prompt-inject.md) — pack disable is coarser than per-plugin.
