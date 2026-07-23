# Deepen — Package slash → ACP advertise (2026-07-22)

**Priority:** P2 · Phase 2  
**Parent:** Master plan Phase 2; [inventory](../20260722-nextcode-extension-inventory.md) — dynamic slash today = skills only; prompts→Face GAP

---

## Goal

Package `[[slash]]` / `prompts/` become Face-visible commands via ACP (`AvailableCommandsUpdate` / Initialize meta), not Face fork / hardcoded builtins.

## Kinds

| kind | Behavior |
|------|----------|
| `prompt` | Inject markdown template (skill-like) |
| `command` | Argv runner (herdr action-like) — may Phase 2.1 |

## Constraints

- Face builtins remain **nextcode pack** (or thin core + pack overlay) — see bare-host freeze.
- Brand-hide list stays pack-owned.
- No in-process TS slash registration.

## Exit criteria

- [ ] Package prompt slash appears in Face after enable.
- [ ] Disable pack/plugin removes command.
- [ ] Docs parity: CONFIG / plugins docs match Face.
