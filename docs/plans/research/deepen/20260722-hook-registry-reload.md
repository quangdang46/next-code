# Deepen — Hook registry reload (package compile-in) (2026-07-22)

**ID:** D7 · **Priority:** P1 · Phase 1–2  
**Status:** Design contract (docs only — not implement approval)  
**Parent:** Master plan — package `[[hooks]]` compile into existing registry  
**Related:** Face `/hooks` add/remove already mutates TOML layers  
**Siblings:** D2 enable-state, D4 bundle merge, D8 manifest, D1 trust

---

## Summary (read first)

When plugins enable/disable, trust changes, or package manifests change, handlers must appear/disappear in the **single** `next-code-hooks` registry — without a second dispatcher and without stale RCE surfaces.

---

## Problem

| Trigger today | Gap |
|---------------|-----|
| Edit `hooks.toml` via Face | Partially wired; confirm daemon/session consistency |
| Disable plugin (D2) | Skills gated in Phase 1; compiled hooks must drop too once D4 lands |
| Trust revoked (D1) | Project executables must unload immediately or before next event |
| Package install (Phase 2) | `[[hooks]]` compile-in needs explicit reload |

Stale handlers after disable = disabled UI + still-running code.

---

## Semantics (target)

| Trigger | Expected |
|---------|----------|
| Edit user/project `hooks.toml` | Reload handlers (Face already merges; confirm daemon/session) |
| Enable/disable plugin with merged hooks | Drop or add provenance `plugin:<id>` handlers |
| Trust revoked | Unload project executable handlers immediately (or next safe point + no further spawn) |
| Package install / link (Phase 2) | Compile `[[hooks]]` → registry; reload |
| Pack profile bare (D0) | Product-default hooks from pack not loaded |

---

## Rules (frozen)

1. **Single registry** — `next-code-hooks` remains the only gate runtime.
2. **Provenance** — every handler tagged (`user.toml` / `project.toml` / `plugin:<id>` / `env`).
3. **Atomic swap** preferred over half-applied merges mid-tool-call — queue reload at a **safe point** (between tool calls / idle).
4. **In-flight hooks** — finish or timeout; **next** event sees the new set.
5. **Idempotent enable toggles** — repeated enable/disable must not duplicate handlers.
6. **Fail soft** — reload errors log + keep last-good set (or empty project set on trust revoke — prefer safety: drop project executables on revoke even if reload errors).

---

## Safe-point sketch

```text
request_reload(reason)
  if tool_call_in_flight:
    mark pending_reload = true
  else:
    swap_registry(build_from_layers())
  on tool_call_end:
    if pending_reload: swap_registry(...)
```

Do not hot-mutate the handler list while a PreToolUse chain is running.

---

## Open Q

| Q | Docs stance for v1 |
|---|--------------------|
| Does Face session need ACP notify “hooks reloaded”? | **Silent OK for v1**; optional toast later |
| Must reload be synchronous on disable? | **Before next PreToolUse** is the bar; sync preferred |

---

## Interaction matrix

| Source layer | On disable / revoke |
|--------------|---------------------|
| User TOML | Remains |
| Project TOML executables | Drop if untrusted |
| `plugin:<id>` | Drop if disabled |
| Env `NEXT_CODE_HOOKS_CONFIG` | Remains until env changes |

---

## Acceptance tests (design)

| ID | Scenario | Pass |
|----|----------|------|
| RL-01 | Disable plugin with compiled hooks | Gone before next PreToolUse |
| RL-02 | Enable → disable → enable | No duplicate handlers |
| RL-03 | Deny still works after reload | Cookbook deny path |
| RL-04 | Trust revoke | Project command handlers not spawned |
| RL-05 | Mid-tool-call reload request | Applied after call ends |

---

## Exit criteria

- [ ] Disable plugin → its compiled hooks gone before next PreToolUse.
- [ ] No duplicate handlers after repeated enable toggles.
- [ ] Test covering reload + deny still works.
- [ ] Provenance retained on remaining handlers.
- [ ] Documented whether ACP notify is required (v1: no).

---

## Non-goals

- Second HTTP-only hook runtime.
- Live-editing handler code without reload.
- Guaranteeing zero in-flight after kill (timeouts already exist).

---

## Status

**Design contract.** Phase 1 needs reload for enable/trust once D4 merges; Phase 2 needs it for manifest compile-in. Waiting **go ahead**.
