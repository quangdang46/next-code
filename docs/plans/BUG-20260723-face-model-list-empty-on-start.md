# BUG — Face `/model` shows no model list right after enter

Branch: `pr-face-model-list-empty-on-start`  
Related: `BUG-20260723-face-startup-session-load.md` (MCP spinner — **secondary / wrong primary**), `BUG-20260723-face-model-persist-mismatch.md` / PR #69 (provider pin / kimi)  
Status: **Implemented**

## Summary (read this first)

- **User symptom (corrected):** mới vào Face, gõ `/model` — **không hiện list model**. Modal “Select model” chỉ còn **Popular providers → connect**, footer prompt **`unknown`**. (Screenshots earlier cũng có Starting session + kimi errors — those can stack, but the complaint is the empty picker.)
- **Bug class:** **Face model inventory / ACP catalog seam** — `ModelState.available` empty when the picker opens.
- **MCP is a red herring for this symptom.** Clearing `Starting session…` does **not** populate `/model`. MCP progress only affects a status spinner seed; `/model` reads `agent.session.models`, not MCP init state.
- **Do not ship MCP-clear-only as the fix for this bug.** Optional later: small MCP clear as UX secondary if still wanted after catalog fix.

## Verified root (code)

`/model` empty list = `ModelState::is_empty()` (`available` empty):

| UI | Code |
|----|------|
| Popular providers all `· connect` | `model.rs` `suggest_args` → if `ctx.models.is_empty()` → `build_popular_provider_items(true)` only |
| Footer `unknown` | `render.rs` `current_model_name().unwrap_or_else(\|\| "unknown")` when `current` is `None` |
| Picker open | `dispatch_open_model_picker` builds ArgPicker from `agent.session.models` (or dashboard.models) |

Face does **not** call a separate ACP `models/list` RPC for this picker. Inventory comes from:

1. **`session/new` / `session/load` → `SessionModelState`** on `SessionCreated` / load handlers (`lifecycle.rs` `handle_session_created` sets `app.models` + `agent.session.models` **only if** `new_models: Some(...)`).
2. **Later `x.ai/models/update`** (`acp_handler/settings.rs` `handle_models_update`) — stock Face path for catalog refresh.

next-code bridge builds (1) from daemon **History** via `session_model_state_from_history` in `pager_agent.rs`. It returns **`None`** when both `provider_model` and `available_models` are empty → Face never binds catalog → stays `ModelState::default()` → Popular-connect + `unknown`.

## Why cold start stays empty / sparse (ranked)

| # | Hypothesis | Verdict |
|---|------------|---------|
| 1 | `/model` before `SessionCreated` binds models (agent starts with empty `ModelState`) | **Verified race** — picker uses live `agent.session.models`; create path seeds empty until ACP response applies `SessionModelState` |
| 2 | Face never forwards daemon **`AvailableModelsUpdated`** → `x.ai/models/update` | **Verified gap** — TUI handles it (`server_events.rs`); `pager_agent` event loops ignore it (`_ => {}`). Prefetch after empty History (`spawn_model_prefetch_update` → bus publish) never reaches Face. No idle reader outside `prompt` / `set_session_model` |
| 3 | Mid-session `ModelChanged` emits `x.ai/models/update` with **`available: &[]`** | **Verified shrink** — `emit_models_update(&model, …, &[])` rebuilds catalog as **current-only** (or no-op if model empty). Worsens inventory after switches; alone usually not Popular-only unless prior state already empty |
| 4 | History catalog empty before OpenRouter/Go `/models` prefetch; Face never sees refresh | **Verified design** — `available_models_display` cache-miss schedules prefetch + relies on ModelsUpdated; Face misses that event |
| 5 | MCP `Starting session…` seed never cleared | **Unrelated to list** — may make “chưa load” feel longer / encourage typing `/model` during race (H1). Fixing MCP alone ≠ list |
| 6 | Provider pin / kimi / OPENROUTER (#69) | **Adjacent** — wrong current model / switch errors; does not by itself explain Popular-only empty `available` |

## LOOK answers

### 1) What `/model` needs to show the list

```
History (available_models + provider_model)
  → session_model_state_from_history → SessionModelState
  → NewSessionResponse.models / SessionCreated
  → ModelState { available, current }
  → OpenModelPicker → suggest_args → build_model_items
```

Optional refresh: daemon `AvailableModelsUpdated` → (missing) → `x.ai/models/update` → `handle_models_update`.

### 2) Why empty / only connect / `unknown`

- **Empty `available`:** never applied `SessionModelState`, or wiped/never refreshed after empty bootstrap.
- **`unknown`:** `current` unset (`None`) — same empty/default `ModelState`, or `From<SessionModelState>` dropped current because id not in `available` map.
- **All connect rows:** intentional empty-catalog fallback in Face `model.rs`, not a separate “auth broken” modal.

### 3) Relation to MCP

| Claim | Honest status |
|-------|----------------|
| MCP spinner hang exists as embed gap | Still true as a **separate** UX bug (`BUG-…-startup-session-load`) |
| MCP is why `/model` has no list | **False** — different state (`mcp_init_progress` vs `ModelState`) |
| MCP-clear PR as fix for user’s pushback | **Wrong target** — cancel / retarget |

Implementer `d9b827ea` (MCP clear): **aborted**; working tree has **no** `emit_mcp_session_ready` left. Recommend **do not reopen MCP-only PR** for this symptom.

## Proposed fix (PLAN — not building yet)

| Kind | Change |
|------|--------|
| **Wire** | On `AvailableModelsUpdated` (and preferably idle/background daemon read): `emit_models_update(provider_model, provider_name, &available_models)` → `x.ai/models/update` with **full** list |
| **Wire** | Fix `ModelChanged` success/error paths: pass real `available_models` from event/snapshot, **never** `&[]` when a catalog exists |
| **Wire** | Ensure cold `session/new` always returns a non-`None` `SessionModelState` when daemon has a current model (even 1-entry fallback) |
| **Prove** | Unit: empty History+current → Some(state); ModelChanged does not empty catalog; AvailableModelsUpdated → models/update payload. Smoke: cold `nextcode` → `/model` shows provider models, not only Popular connect |
| **Out of scope** | MCP spinner clear (secondary); full #69 pin work already shipping |

## Overlap

| Layer | This BUG | MCP startup BUG | #69 |
|-------|----------|-----------------|-----|
| `/model` empty / `unknown` | **Primary** | No | No |
| `Starting session…` hang | Secondary correlate | Primary | No |
| kimi / `opencode go` pin | Adjacent noise | Adjacent | Primary |

## Decision gate

**Implemented** — Face ACP idle/prompt/set_model paths forward `AvailableModelsUpdated` via `x.ai/models/update`, keep a catalog cache so `ModelChanged` never emits `&[]`, and unit-test cold History → `SessionModelState` shapes.
