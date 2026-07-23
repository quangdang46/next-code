# Deepen — Hook registry reload (package compile-in) (2026-07-22)

**ID:** D7 · **Priority:** P1 · Phase 1–2  
**Status:** Design contract (docs only — not implement approval)  
**Parent:** Master plan — package `[[hooks]]` compile into existing registry  
**Related:** Face `/hooks` add/remove already mutates TOML layers  
**Siblings:** D2 enable-state, D4 bundle merge, D8 manifest, D1 trust

---

## Summary (read first)

When plugins enable/disable, trust changes, or package manifests change, handlers must appear/disappear in the **single** `next-code-hooks` registry — without a second dispatcher and without stale RCE surfaces.

**Verified today:** Face “Reload hooks” refreshes the **list UI** from disk. It does **not** rebuild the live `Arc<RwLock<HookRegistry>>` inside the agent tool registry. That gap is the Phase 1 reload ticket.

---

## Problem

| Trigger today | Gap |
|---------------|-----|
| Edit `hooks.toml` via Face | TOML rewritten; list re-reads; **session spawn path may keep old handlers** |
| Disable plugin (D2) | Skills gated in Phase 1; compiled hooks must drop too once D4 lands |
| Trust revoked (D1) | Project executables must unload immediately or before next event |
| Package install (Phase 2) | `[[hooks]]` compile-in needs explicit reload |

Stale handlers after disable = disabled UI + still-running code.

---

## Verified code map (2026-07-22)

### Build (session start)

| Step | Symbol | Behavior |
|------|--------|----------|
| Load layers | `next_code_hooks::load_hooks_config` | `crates/next-code-hooks/src/config.rs` — user → project → env merge |
| Construct | `HookRegistry::from_config` | `crates/next-code-hooks/src/registry.rs` — event → `Vec<HookHandlerConfig>` |
| Hold live | `ToolRegistry::new` | `crates/next-code-app-core/src/tool/mod.rs` — `Arc::new(RwLock::new(HookRegistry::from_config(...)))` + legacy v1 merge via `legacy_v1_to_v2_handlers` |
| Dispatch | PreToolUse path | Same crate `tool/mod.rs` reads `self.hook_registry` under lock |

There is **no** public `HookRegistry::reload` / `swap` API today — only `new` / `from_config` / `get_matching`.

### Face ACP “reload” (UI only)

| Step | Symbol | Behavior |
|------|--------|----------|
| Keybind | Extensions Hooks tab `'r'` | `crates/xai-grok-pager/src/views/extensions_modal.rs` → `HooksAction::Reload` |
| ACP | `x.ai/hooks/action` | `src/cli/pager_agent.rs` → `face_plugins::hooks_action_payload` |
| Handler | `HooksAction::Reload` arm | `src/cli/face_plugins.rs` — returns success **string only**; does **not** call `load_hooks_config` into the agent |
| Face comment | `modals.rs` | “Hooks reload re-reads hooks.toml only — do not wipe plugins…” — intentional **list** refresh, not session registry |

`hooks_list_payload` (`face_plugins.rs`) always re-reads TOML layers for the Extensions tab. That is **catalog honesty**, not live dispatcher honesty.

### Mutations that rewrite disk

| Action | Symbol | Disk effect | Live registry |
|--------|--------|-------------|---------------|
| Enable/Disable | `set_hook_enabled_by_face_name` | Rewrites user/project TOML | **Not** swapped |
| Add | `merge_hooks_toml_into_user` | Merges into `~/.next-code/hooks.toml` | **Not** swapped |
| Remove | face hooks remove path | Deletes handler from TOML | **Not** swapped |
| Trust/Untrust ACP | `HooksAction::Trust` | Returns `Unsupported` — “always loaded” | N/A (D1 will change product) |

### Alternate load sites (do not invent a second bus)

| Site | Path | Note |
|------|------|------|
| Write tool local | `crates/next-code-app-core/src/tool/write.rs` | Builds a **local** `HookRegistry::from_config(load_hooks_config())` — not the session Arc |
| DCG bridge tests/helpers | `crates/next-code-app-core/src/dcg_bridge.rs` | Fresh `from_config` per call |
| Integration tests | `tests/hooks_integration.rs` | Construct once per test |

**Implement rule:** one session-owned `Arc<RwLock<HookRegistry>>` remains canonical; reload = `*lock = HookRegistry::from_config(build_layers())` (plus provenance layers when D4 lands).

---

## Semantics (target)

| Trigger | Expected |
|---------|----------|
| Edit user/project `hooks.toml` | Reload handlers (Face already merges disk; **must** swap session registry) |
| Enable/disable plugin with merged hooks | Drop or add provenance `plugin:<id>` handlers |
| Trust revoked | Unload project executable handlers immediately (or next safe point + no further spawn) |
| Package install / link (Phase 2) | Compile `[[hooks]]` → registry; reload |
| Pack profile bare (D0) | Product-default hooks from pack not loaded |

---

## Rules (frozen)

1. **Single registry** — `next-code-hooks` / session `hook_registry` remains the only gate runtime.
2. **Provenance** — every handler tagged (`user.toml` / `project.toml` / `plugin:<id>` / `env` / `legacy_v1`).
3. **Atomic swap** preferred over half-applied merges mid-tool-call — queue reload at a **safe point** (between tool calls / idle).
4. **In-flight hooks** — finish or timeout; **next** event sees the new set.
5. **Idempotent enable toggles** — repeated enable/disable must not duplicate handlers.
6. **Fail soft** — reload errors log + keep last-good set (or empty project set on trust revoke — prefer safety: drop project executables on revoke even if reload errors).
7. **Face Reload must become real** — ACP success only after session swap **or** documented “UI-only” until agent notify lands; do not leave the current success string implying live reload if it does not.

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

Do not hot-mutate the handler list while a PreToolUse chain is running (`get_matching` + `execute_hook` in flight).

### Suggested API (non-normative)

```rust
// crates/next-code-hooks — keep from_config pure
impl HookRegistry {
    pub fn from_config(config: HooksConfig) -> Self { /* exists */ }
}

// crates/next-code-app-core ToolRegistry
async fn reload_hooks(&self) {
    let mut cfg = next_code_hooks::load_hooks_config();
    // re-apply legacy v1 merge (same as ToolRegistry::new)
    // when D4 lands: extend with enabled+trusted plugin layers
    let built = HookRegistry::from_config(cfg);
    *self.hook_registry.write().await = built;
}
```

Wire Face `HooksAction::Reload` → ACP → daemon/session `reload_hooks` (not message-only). Enable/disable/add/remove should either call the same path or set `pending_reload` after TOML write.

---

## Open Q

| Q | Docs stance for v1 |
|---|--------------------|
| Does Face session need ACP notify “hooks reloaded”? | **Silent OK for v1**; optional toast later |
| Must reload be synchronous on disable? | **Before next PreToolUse** is the bar; sync preferred |
| Does write.rs local registry stay? | Prefer migrate to session registry; do not add a third cache |

---

## Interaction matrix

| Source layer | On disable / revoke |
|--------------|---------------------|
| User TOML | Remains |
| Project TOML executables | Drop if untrusted (D1) |
| `plugin:<id>` | Drop if disabled (D2/D4) |
| Env `NEXT_CODE_HOOKS_CONFIG` | Remains until env changes |
| Legacy v1 `config.toml [hooks]` | Remains until removed from config (still merged at build) |

---

## Acceptance tests (design)

| ID | Scenario | Pass |
|----|----------|------|
| RL-01 | Disable plugin with compiled hooks | Gone before next PreToolUse |
| RL-02 | Enable → disable → enable | No duplicate handlers |
| RL-03 | Deny still works after reload | Cookbook deny path |
| RL-04 | Trust revoke | Project command handlers not spawned |
| RL-05 | Mid-tool-call reload request | Applied after call ends |
| RL-06 | Face Hooks Reload | Session `hook_registry` matches disk (not UI-only) |
| RL-07 | Face enable/disable handler | Next PreToolUse honors new `enabled` flag without restart |

Named existing tests to extend (not replace):

- `src/cli/face_plugins.rs` — `hooks_action_*` (disk mutations)
- `crates/xai-grok-pager/.../task_result.rs` — `hooks_disable_success_refreshes_lists_without_plugins_reload_loop`
- `tests/hooks_integration.rs` — execute allow/deny

Add: unit test that swaps `Arc<RwLock<HookRegistry>>` and asserts `get_matching` changes.

---

## Exit criteria

- [ ] Disable plugin → its compiled hooks gone before next PreToolUse.
- [ ] No duplicate handlers after repeated enable toggles.
- [ ] Test covering reload + deny still works.
- [ ] Provenance retained on remaining handlers.
- [ ] Documented whether ACP notify is required (v1: no).
- [ ] Face `HooksAction::Reload` either swaps live registry or stops claiming “reloaded” until it does.
- [ ] `ToolRegistry` documents the single reload entrypoint (shared by Face + D4 compile-in + D1 revoke).

---

## Non-goals

- Second HTTP-only hook runtime.
- Live-editing handler code without reload.
- Guaranteeing zero in-flight after kill (timeouts already exist in `execute.rs`).
- Chaining hooks reload into `PluginsAction::Reload` (Face deliberately avoids that loop).

---

## Phased delivery

| Step | Deliverable | Depends |
|------|-------------|---------|
| 7a | Session `reload_hooks` + Face Reload wires it | ToolRegistry Arc |
| 7b | Enable/disable/add/remove set pending or call 7a | 7a |
| 7c | D4 plugin layers included in `build_from_layers` | D4 |
| 7d | Trust revoke drops project executables via same path | D1 |

---

## Status

**Design contract.** Phase 1 needs reload for enable/trust once D4 merges; Phase 2 needs it for manifest compile-in. **Today’s Face Reload is UI-catalog only — implement must close that lie.** Waiting **go ahead**.
