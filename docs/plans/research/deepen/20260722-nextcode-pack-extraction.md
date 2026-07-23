# Deepen — nextcode pack extraction (product profile) (2026-07-22)

**Priority:** P2 (sketch starts Phase 1 §5)  
**Parent:** Master plan “Parallel: nextcode pack extraction”  
**Freeze companion:** [`20260722-bare-host-no-prompt-inject.md`](./20260722-bare-host-no-prompt-inject.md)

---

## Goal

Move opinions a white-label fork would change behind a **disableable product profile** named `nextcode`, without forking Face or the agent loop.

## Product profile table (Phase 1 sketch → Phase 2 extract)

| Piece | Host | nextcode pack | Notes |
|-------|------|---------------|-------|
| Face render / ACP / permissions / hook engine | ✓ | | |
| Built-in `system_prompt.md` | | ✓ | Reclassified from CORE |
| Welcome / status / quit chrome copy | | ✓ | |
| Brand-hidden slash set | | ✓ | |
| Visible `/plugins` `/hooks` | | ✓ (labels) | Engine host |
| Settings catalog groups / defaults | | ✓ | Persistence host |
| Starter skills / MCP / hook recipes | | ✓ | |
| Default tool enablement prefs | | ✓ | Tool binaries host |
| Auth connect UX copy | | ✓ | OAuth host |

## Disable / bare

`profile = "bare"` or `packs.nextcode = false` → no pack inject (Pi-style empty shell). User/project overlays still load.

## Exit criteria

- [ ] Table frozen in repo (this file + inventory update).
- [ ] At least system prompt + brand-hide gated by profile flag when implement starts.
- [ ] Alternate pack can supply its own prompt without core PR (Phase 2+).
