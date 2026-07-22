# Deepen — nextcode pack extraction (product profile) (2026-07-22)

**ID:** D10 · **Priority:** P2 (sketch starts Phase 1 §5)  
**Status:** Design sketch (docs only — not implement approval)  
**Parent:** Master plan “Parallel: nextcode pack extraction”  
**Freeze companion:** [`20260722-bare-host-no-prompt-inject.md`](./20260722-bare-host-no-prompt-inject.md) (D0)  
**Inventory:** [`../20260722-nextcode-extension-inventory.md`](../20260722-nextcode-extension-inventory.md)

---

## Summary (read first)

Move opinions a white-label fork would change behind a **disableable product profile** named `nextcode`, without forking Face or the agent loop. **D0 freezes the bare rule** (no opinionated prompt inject). This file is the **extraction table** and phased move plan, with **verified load sites** for prompts today.

---

## Goal

| Keep as host | Extract as nextcode pack |
|--------------|--------------------------|
| Face render, ACP, permissions, hook engine | Brand voice, welcome/chrome copy |
| Tool binaries / dispatch | Built-in `system_prompt.md` |
| Mechanical tool-protocol scaffolding | Brand-hidden slash set, starter content |

Bare / alternate packs opt out without forking the host.

---

## Verified prompt load sites (must gate for D0)

| Piece | Path / symbol | Today |
|-------|---------------|-------|
| Embedded default | `crates/next-code-base/src/prompt.rs` → `DEFAULT_SYSTEM_PROMPT = include_str!("prompt/system_prompt.md")` | Always in `base_system_prompt_parts` |
| Edit-mode modules | same file → `EDIT_*_PROMPT` includes | Mechanical (host OK) |
| Mermaid module | `MERMAID_PROMPT` / `PromptCapabilities` | Feature flag — host capability, not brand |
| Swarm routing | `DEFAULT_SWARM_PROMPT` + `load_swarm_prompt` | Product-ish; treat as pack-owned when extracting |
| Tests asserting identity | `crates/next-code-base/src/prompt_tests.rs` | Asserts not Claude; will need bare vs nextcode cases |
| Overlay PLUG | `SYSTEM.md` / `APPEND_SYSTEM.md` / `AGENTS.md` | User/project — **always allowed** (D0) |

**Phase 1 code touch rule:** any PR that edits `prompt.rs` / `system_prompt.md` must gate pack content behind profile (or leave a `TODO(D10)` that fails bare smoke — prefer real gate).

Inventory still labels System prompt as CORE — **D0 wins** until inventory row is patched.

---

## Product profile table (Phase 1 sketch → Phase 2 extract)

| Piece | Host | nextcode pack | Notes |
|-------|------|---------------|-------|
| Face render / ACP / permissions / hook engine | ✓ | | |
| Built-in `system_prompt.md` | | ✓ | Reclassified from CORE (D0) |
| Welcome / status / quit chrome copy | | ✓ | Face strings in grok-pager |
| Brand-hidden slash set | | ✓ | |
| Visible `/plugins` `/hooks` | | ✓ (labels) | Engine host |
| Settings catalog groups / defaults | | ✓ | Persistence host |
| Starter skills / MCP / hook recipes | | ✓ | |
| Default tool enablement prefs | | ✓ | Tool binaries host |
| Auth connect UX copy | | ✓ | OAuth host |
| Notepad feature | ✓ | Default-on prefs | Feature host; prefs pack |
| Edit-tool protocol prompts | ✓ | | Not brand voice |
| Tool XML / schema reminders | ✓ | | Mechanical host |

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

### Assembled prompt layers (must match D0)

1. HOST mechanical (edit mode, tool schema) — always  
2. Pack (`system_prompt.md`, swarm defaults, brand) — **iff pack enabled**  
3. PLUG overlays — always when present  
4. Session/runtime (notepad priority, etc.) — existing host rules  

Bare smoke = layers 1+3+4 only.

---

## Phased extraction

| Phase | Move |
|-------|------|
| Phase 1 §5 | **Sketch this table**; gate `system_prompt.md` load behind profile flag when touching prompts |
| Phase 2 | Brand-hide list + welcome strings behind pack module/crate feature |
| Phase 2–3 | Starter skills/MCP as optional pack content, not CORE |
| Later | Alternate pack ships own prompt without core PR |

Phase 1 first build does **not** block on full extraction — but must **not violate D0** when prompt paths are edited.

### Minimal Phase 1 implement sketch

```text
fn base_system_prompt_parts(...) -> Vec<String> {
  let mut parts = vec![/* edit_mode + optional mermaid */];
  if product_pack_enabled("nextcode") {
    parts.insert(0, DEFAULT_SYSTEM_PROMPT.to_string());
  }
  parts
}
```

Exact API name deferred; behavior frozen by D0 T1–T3.

---

## Inventory follow-up

| Row | Today | After |
|-----|-------|-------|
| System prompt | CORE + overlays | **PROD pack** + PLUG overlays |
| Brand-hidden slash | PROD | PROD pack (unchanged class; explicit pack ownership) |
| Swarm prompt default | *(unspecified / CORE-ish)* | PROD pack (overlays remain PLUG) |

Update inventory when pack flag lands or as docs follow-up — D0 wins until then.

---

## Acceptance tests (design)

| ID | Scenario | Pass |
|----|----------|------|
| PK-01 | `profile=bare` | No baked nextcode persona (D0 T1) |
| PK-02 | `profile=nextcode` | Parity with today’s prompt/chrome |
| PK-03 | Table in repo | This file + D0 linked from readiness |
| PK-04 | Inventory row | Updated when flag ships |
| PK-05 | `prompt_tests` | Bare profile does not require DEFAULT_SYSTEM_PROMPT identity strings |

---

## Exit criteria

- [ ] Table frozen in repo (this file + inventory update).
- [ ] At least system prompt + brand-hide gated by profile flag when implement starts.
- [ ] Alternate pack can supply its own prompt without core PR (Phase 2+).
- [ ] D0 smoke tests still green.
- [ ] `include_str!("prompt/system_prompt.md")` remains pack-owned content, not “always host.”

---

## Non-goals

- Forking Face for white-label.
- Making bare the default end-user install.
- Resolving D8 filename / D11 tools timing (orthogonal).
- Moving tool binaries into the pack.

---

## Risks

| Risk | Mitigation |
|------|------------|
| Pack never extracted → forever fork | Phase 1 sketch + D0 gate on prompt PRs |
| Accidental CORE creep | Review: “would a fork rewrite this?” → pack |
| Bare strips mechanical edit prompts | Keep `EDIT_*` host; only gate brand file |

---

## Status

**Sketch ready; D0 frozen.** Waiting **go ahead** for code; docs may update inventory anytime.
