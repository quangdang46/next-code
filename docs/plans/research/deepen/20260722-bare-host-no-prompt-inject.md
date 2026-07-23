# Freeze — Bare / platform must NOT inject opinionated prompts (2026-07-22)

**Status:** **FROZEN product contract** (docs). Phase 1+ implement must obey.  
**Priority:** P0 · D0  
**Parent:** [`PLAN-20260722-pi-full-custom-platform.md`](../../PLAN-20260722-pi-full-custom-platform.md)  
**Companion:** [`20260722-nextcode-pack-extraction.md`](./20260722-nextcode-pack-extraction.md) (D10)  
**Inventory cite:** [`../20260722-nextcode-extension-inventory.md`](../20260722-nextcode-extension-inventory.md) §8 + row “System prompt”  
**Readiness:** [`../../PLAN-20260722-platform-implement-readiness.md`](../../PLAN-20260722-platform-implement-readiness.md) criterion “Bare no opinionated prompts”

---

## Frozen rule (one sentence)

**Bare host / platform shell = Pi-style empty identity:** no brand chrome, no nextcode system/append prompt, no starter skills/MCP/slash palette, unless an **explicit product pack** (default: `nextcode`) is enabled.

---

## Why this exists

Pi does not inject opinionated system/brand prompts into a bare shell. next-code today ships a built-in `system_prompt.md` tagged in inventory as **CORE default identity** — that misclassifies product opinion as host.

If Phase 1+ treats inventory “CORE” literally, bare mode still looks like nextcode. White-label / alternate-pack products would have to fork or patch core to remove persona strings.

This freeze is the **product-law** companion to D10 (pack extraction). D0 answers *what must never happen on bare*; D10 answers *how pieces move behind a pack flag*.

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

**Docs follow-up (not blocked on code):** when pack extraction lands (or sooner as a research patch), update inventory row “System prompt” from CORE → PROD / pack. Until then, **this file wins** over the inventory table for implement decisions.

---

## What “bare” means (and does not)

| Bare **is** | Bare **is not** |
|-------------|-----------------|
| Pack id off / `profile = "bare"` | A Face fork or alternate TUI |
| No nextcode persona in assembled system prompt | Stripping user `SYSTEM.md` / project `AGENTS.md` |
| No brand welcome / quit / Extensions product copy | Disabling mechanical tool-protocol scaffolding |
| No starter skills/MCP/slash from the nextcode pack | Forbidding PLUG overlays the user installed |
| Default for white-label / platform-only smoke | Default install for end users (that stays **nextcode**) |

Bare ≈ “disable nextcode pack,” not “fork Face.”

---

## Profiles

| Profile | System / brand inject | Slash / chrome | Skills / MCP starters |
|---------|----------------------|----------------|------------------------|
| **bare** | None from nextcode pack | Platform-minimal (or empty pack palette) | None unless user/project installs |
| **nextcode** (default distro) | Today’s `system_prompt.md` + welcome/brand | Brand-hide + `/plugins` `/hooks` visible | Optional starters |
| **alternate pack** | That pack’s prompts | That pack’s palette | That pack’s content |

Config sketch (non-normative until Phase 1 code):

```toml
# Hypothetical — final key names land with D10
[product]
profile = "bare"          # or "nextcode" | custom pack id
# packs.nextcode = false  # equivalent disable
```

---

## Assembled system prompt — load order

When coding the load path, treat layers as:

1. **HOST mechanical** (allowed always): tool XML / schema instructions, safety rails required for correct tool use — **not** brand voice or product persona.
2. **Pack prompts** (nextcode or alternate): built-in `system_prompt.md` and pack-owned append fragments — **only if pack enabled**.
3. **PLUG user/project**: `SYSTEM.md`, `APPEND_SYSTEM.md`, `AGENTS.md`, prompt overlays — **always when files exist**, independent of pack.
4. **Session / runtime**: turn-local injections (notepad priority tier, etc.) per existing host rules.

**Bare session smoke:** layers 1 + 3 + 4 only. Layer 2 must be absent (no nextcode persona substring from the baked file).

---

## Host MAY vs MUST NOT inject

| Host **MAY** (mechanical) | Host **MUST NOT** on bare |
|---------------------------|---------------------------|
| Tool-call format / schema reminders | Product name voice (“You are next-code…”) |
| Permission / DCG operational notes | Welcome / quit brand chrome as system text |
| Empty or null product identity | Shipping starter skill/MCP text as if CORE |
| User-authored overlay content | Treating inventory CORE label as license to inject |

Anything a fork would rewrite for white-label → **pack**, not host.

---

## Cross-links

| Doc | Relationship |
|-----|----------------|
| D10 pack extraction | Implementation table for which chrome/prompt/slash pieces move behind the flag |
| Inventory “System prompt” row | Must be reclassified PROD; D0 is authoritative until then |
| Master plan Option B′ | Platform vs nextcode pack; bare/alternate pack |
| Face limits research | Face stays sealed; bare is pack-off, not UI guest host |
| Readiness gate | Criterion “Bare no opinionated prompts” = **Frozen** via this file |

---

## Implement checklist (when coding — not this doc)

1. [ ] Product profile flag / pack id: `bare` | `nextcode` | custom.
2. [ ] Load built-in `system_prompt.md` **only** when nextcode pack enabled.
3. [ ] Face welcome/status/quit strings gated by pack.
4. [ ] Brand-hide slash set gated by pack (bare = platform-minimal palette).
5. [ ] Update inventory row: System prompt → PROD (pack), not CORE.
6. [ ] Smoke: bare session has no nextcode persona string in assembled system prompt (user overlays still apply).
7. [ ] Smoke: `profile = nextcode` restores today’s prompt + chrome behavior.
8. [ ] Unit/integration: pack-disable path does not regress tool-protocol scaffolding.

---

## Non-goals

- Removing user `SYSTEM.md` / project `AGENTS.md` in bare mode.
- Making host unable to attach tool-protocol instructions.
- Shipping bare as the default install for end users (default remains **nextcode** pack).
- Replacing Face or moving render into a guest package.
- Resolving Phase 2+ ABI open Qs (manifest filename, `[[tools]]` timing, external pane).

---

## Acceptance tests (docs → code later)

| # | Setup | Expect |
|---|--------|--------|
| T1 | `profile=bare`, no user SYSTEM.md | Assembled system prompt has **no** baked nextcode persona from `system_prompt.md` |
| T2 | `profile=bare` + user `SYSTEM.md` | User text present; pack file still absent |
| T3 | `profile=nextcode` | Today’s `system_prompt.md` content present (parity) |
| T4 | bare + Extensions UI | No product welcome copy that asserts nextcode identity as default voice |
| T5 | Inventory docs | Row “System prompt” marked PROD/pack after follow-up edit |

---

## Open Q answered by this freeze

Audit: *Is today’s built-in `system_prompt.md` host or nextcode pack?*  
**Answer: nextcode pack (PROD).** Host is empty-shell capable.

**Status:** Frozen. Expand agents / implement must not weaken this contract. Line-count target for this file: **≥80** (met); other deepen designs target **≥120** after expand wave.
