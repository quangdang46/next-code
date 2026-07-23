# Freeze — Bare / platform must NOT inject opinionated prompts (2026-07-22)

**Status:** **FROZEN product contract** (docs). Phase 1+ implement must obey.  
**Parent:** [`PLAN-20260722-pi-full-custom-platform.md`](../../PLAN-20260722-pi-full-custom-platform.md)  
**Inventory cite:** [`../20260722-nextcode-extension-inventory.md`](../20260722-nextcode-extension-inventory.md) §8 + row “System prompt”

---

## Frozen rule (one sentence)

**Bare host / platform shell = Pi-style empty identity:** no brand chrome, no nextcode system/append prompt, no starter skills/MCP/slash palette, unless an **explicit product pack** (default: `nextcode`) is enabled.

---

## Why this exists

Pi does not inject opinionated system/brand prompts into a bare shell. next-code today ships a built-in `system_prompt.md` tagged in inventory as **CORE default identity** — that misclassifies product opinion as host.

If Phase 1+ treats inventory “CORE” literally, bare mode still looks like nextcode.

---

## Inventory correction (reclassify)

| Surface | Inventory said (2026-07-22) | **Correct class under this freeze** |
|---------|-----------------------------|-------------------------------------|
| Built-in `system_prompt.md` | **CORE** default identity | **PROD / nextcode pack** — disable with pack |
| `SYSTEM.md` / `APPEND_SYSTEM.md` / `AGENTS.md` / overlays | PLUG | **PLUG** (user/project) — always allowed when present |
| Welcome / quit / Extensions chrome copy | PROD | **PROD / nextcode pack** |
| Brand-hidden slash set | PROD | **PROD / nextcode pack** |
| Starter skills / example MCP / hook recipes | PROD (optional) | **PROD / nextcode pack** |
| Host-minimal identity | *(unspecified)* | **HOST** — empty or user-only; no baked brand voice |

**Host may still supply:** mechanical system scaffolding required for tool use / safety (schema instructions, tool XML format) that is **not** brand or product persona. Anything a fork would rewrite for white-label → pack.

---

## Profiles

| Profile | System / brand inject | Slash / chrome | Skills / MCP starters |
|---------|----------------------|----------------|------------------------|
| **bare** | None from nextcode pack | Platform-minimal (or empty pack palette) | None unless user/project installs |
| **nextcode** (default distro) | Today’s `system_prompt.md` + welcome/brand | Brand-hide + `/plugins` `/hooks` visible | Optional starters |
| **alternate pack** | That pack’s prompts | That pack’s palette | That pack’s content |

Bare ≈ “disable nextcode pack,” not “fork Face.”

---

## Implement checklist (when coding — not this doc)

1. [ ] Product profile flag / pack id: `bare` | `nextcode` | custom.
2. [ ] Load built-in `system_prompt.md` **only** when nextcode pack enabled.
3. [ ] Face welcome/status strings gated by pack.
4. [ ] Update inventory row: System prompt → PROD (pack), not CORE.
5. [ ] Smoke: bare session has no nextcode persona string in assembled system prompt (user overlays still apply).

---

## Non-goals

- Removing user `SYSTEM.md` / project `AGENTS.md` in bare mode.
- Making host unable to attach tool-protocol instructions.
- Shipping bare as the default install for end users (default remains **nextcode** pack).

---

## Open Q answered by this freeze

Audit: *Is today’s built-in `system_prompt.md` host or nextcode pack?*  
**Answer: nextcode pack (PROD).** Host is empty-shell capable.
