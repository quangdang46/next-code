# Deepen — nextcode pack extraction (product profile) (2026-07-22)

**ID:** D10 · **Priority:** P2 (sketch starts Phase 1 §5)  
**Status:** Design sketch (docs only — not implement approval)  
**Parent:** Master plan “Parallel: nextcode pack extraction”  
**Freeze companion:** [`20260722-bare-host-no-prompt-inject.md`](./20260722-bare-host-no-prompt-inject.md) (D0)  
**Inventory:** [`../20260722-nextcode-extension-inventory.md`](../20260722-nextcode-extension-inventory.md)

---

## Summary (read first)

Move opinions a white-label fork would change behind a **disableable product profile** named `nextcode`, without forking Face or the agent loop. **D0 freezes the bare rule** (no opinionated prompt inject). This file is the **extraction table** and phased move plan.

---

## Goal

| Keep as host | Extract as nextcode pack |
|--------------|--------------------------|
| Face render, ACP, permissions, hook engine | Brand voice, welcome/chrome copy |
| Tool binaries / dispatch | Built-in `system_prompt.md` |
| Mechanical tool-protocol scaffolding | Brand-hidden slash set, starter content |

Bare / alternate packs opt out without forking the host.

---

## Product profile table (Phase 1 sketch → Phase 2 extract)

| Piece | Host | nextcode pack | Notes |
|-------|------|---------------|-------|
| Face render / ACP / permissions / hook engine | ✓ | | |
| Built-in `system_prompt.md` | | ✓ | Reclassified from CORE (D0) |
| Welcome / status / quit chrome copy | | ✓ | |
| Brand-hidden slash set | | ✓ | |
| Visible `/plugins` `/hooks` | | ✓ (labels) | Engine host |
| Settings catalog groups / defaults | | ✓ | Persistence host |
| Starter skills / MCP / hook recipes | | ✓ | |
| Default tool enablement prefs | | ✓ | Tool binaries host |
| Auth connect UX copy | | ✓ | OAuth host |
| Notepad feature | ✓ | Default-on prefs | Feature host; prefs pack |

---

## Disable / bare

`profile = "bare"` or `packs.nextcode = false` → no pack inject (Pi-style empty shell). User/project overlays still load (D0).

```toml
# Non-normative sketch — final keys with implement
[product]
profile = "nextcode"   # bare | nextcode | <custom>
```

| Profile | Pack prompts | Pack chrome | User overlays |
|---------|--------------|-------------|---------------|
| `bare` | Off | Platform-minimal | On |
| `nextcode` | On | On | On |
| custom | That pack | That pack | On |

---

## Phased extraction

| Phase | Move |
|-------|------|
| Phase 1 §5 | **Sketch this table**; gate `system_prompt.md` load behind profile flag when touching prompts |
| Phase 2 | Brand-hide list + welcome strings behind pack module/crate feature |
| Phase 2–3 | Starter skills/MCP as optional pack content, not CORE |
| Later | Alternate pack ships own prompt without core PR |

Phase 1 first build does **not** block on full extraction — but must **not violate D0** when prompt paths are edited.

---

## Inventory follow-up

| Row | Today | After |
|-----|-------|-------|
| System prompt | CORE + overlays | **PROD pack** + PLUG overlays |
| Brand-hidden slash | PROD | PROD pack (unchanged class; explicit pack ownership) |

Update inventory when pack flag lands or as docs follow-up — D0 wins until then.

---

## Acceptance tests (design)

| ID | Scenario | Pass |
|----|----------|------|
| PK-01 | `profile=bare` | No baked nextcode persona (D0 T1) |
| PK-02 | `profile=nextcode` | Parity with today’s prompt/chrome |
| PK-03 | Table in repo | This file + D0 linked from readiness |
| PK-04 | Inventory row | Updated when flag ships |

---

## Exit criteria

- [ ] Table frozen in repo (this file + inventory update).
- [ ] At least system prompt + brand-hide gated by profile flag when implement starts.
- [ ] Alternate pack can supply its own prompt without core PR (Phase 2+).
- [ ] D0 smoke tests still green.

---

## Non-goals

- Forking Face for white-label.
- Making bare the default end-user install.
- Resolving D8 filename / D11 tools timing (orthogonal).

---

## Risks

| Risk | Mitigation |
|------|------------|
| Pack never extracted → forever fork | Phase 1 sketch + D0 gate on prompt PRs |
| Accidental CORE creep | Review: “would a fork rewrite this?” → pack |

---

## Status

**Sketch ready; D0 frozen.** Waiting **go ahead** for code; docs may update inventory anytime.
