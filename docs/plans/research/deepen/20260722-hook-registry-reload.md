# Deepen — Hook registry reload (package compile-in) (2026-07-22)

**Priority:** P1 · Phase 1–2  
**Parent:** Master plan — package `[[hooks]]` compile into existing registry  
**Related:** Face `/hooks` add/remove already mutates TOML layers

---

## Problem

When plugins enable/disable or package manifests change, handlers must appear/disappear without inventing a second dispatcher — and without stale RCE surfaces.

## Semantics (target)

| Trigger | Expected |
|---------|----------|
| Edit user/project `hooks.toml` | Reload handlers (Face already merges; confirm daemon/session) |
| Enable/disable plugin with merged hooks | Drop or add provenance `plugin:<id>` handlers |
| Trust revoked | Unload project executable handlers immediately |
| Package install (Phase 2) | Compile `[[hooks]]` → registry; reload |

## Rules

1. **Single registry** — next-code-hooks remains the only gate runtime.
2. **Provenance** — every handler tagged (user.toml / project.toml / plugin:id / env).
3. **Atomic swap** preferred over half-applied merges mid-tool-call (queue reload at safe point).
4. **In-flight hooks** — finish or timeout; next event sees new set.

## Open Q

Does Face session need ACP notify “hooks reloaded,” or silent is OK for v1?

## Exit criteria

- [ ] Disable plugin → its compiled hooks gone before next PreToolUse.
- [ ] No duplicate handlers after repeated enable toggles.
- [ ] Test covering reload + deny still works.
