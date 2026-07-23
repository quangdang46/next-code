# Plan Report

## Summary (read this first)
- **You asked:** Face `/model` palette (PR #57 @ `35b3eae7b`) seems not persistent — quit and reopen still shows the old model (e.g. `deepseek-v4-flash`).
- **What is going on:** Selecting a model in an **active agent** is meant to (1) switch the live ACP session and (2) write a default for next cold start. Live switch goes through ACP `SetSessionModel` → daemon `Request::SetModel` only. Persist is split across Face `PersistSetting` → `[provider].default_model` and (on successful SetModel) daemon `Config::set_default_model(model, provider_key)`. Several seams make “UI says A, next launch is still B” likely: Face persist does **not** write `default_provider`, dashboard `/model` **deliberately does not persist**, and Grok’s `persist_models_default` path is still a **no-op stub**.
- **We recommend:** Treat this as a **wire/persist bug** (not a palette UX bug). Align Face default-model persist with legacy TUI / daemon: write **both** `default_model` and `default_provider` through one authoritative writer; fix or delete the no-op `persist_models_default`; decide whether dashboard staging should also update the global default.
- **Risk:** Medium (config races / wrong provider pin can strand users on wrong route)
- **Status:** **Implemented** — Root cause amplified: Face stub `process_chat_mode_enabled()` was always `true`, so `SetDefaultModel` **never** emitted persist (`if !chat_mode`). Fixed stub to `false` (Build-equivalent). Also: Face `/model` persists model+provider atomically; dashboard `/model` persists global default; `persist_models_default` stub wired. Residual: daemon `Config::save` full-rewrite can still race with Face toml_edit on quit-before-flush (unverified); cross-provider pin from Overview float uses display-name → lowercase key.

## Bug investigation
- **Verified root cause:** **Partial / multi-path** — not a single missing call. Persist is attempted, but the Face-only writer is incomplete vs what cold start needs, and one Grok persist effect is a stub. Exact user trigger (agent vs dashboard, SetModel success vs fail, quit-before-async) **unverified — needs operator repro + config diff**.
- **Hypotheses ranked:**
  1. **Face `PersistSetting(default_model)` writes only `[provider].default_model`, never `default_provider`.** Cold start applies defaults via `set_config_default_model(model, default_provider_key)`. Cross-provider picks need the provider pin; Face path alone is insufficient. Daemon `apply_set_model` does write both — but only if ACP SetModel **succeeds**. — **verified** (writer + cold-start consumer)
  2. **Dashboard `/model` stages next spawn only — “deliberately do NOT persist a global default”.** Quit from dashboard after picking A → reopen still B. — **verified** (explicit comment + empty effects)
  3. **`Effect::PersistPreferredModel` → `persist_models_default` is a no-op stub** (`Ok(())`). Stock Grok uses this for `[models].default`; SwitchModelComplete still emits it after a successful switch. Dead secondary persist. — **verified**
  4. **Dual writers race:** Face `set_toml_key` (toml_edit surgical) vs daemon `Config::save` (full `toml::to_string_pretty` rewrite). Late Face write can clobber daemon’s `default_provider` update. — **verified** as race shape; **unverified** that it hit this user
  5. **Quit before async effects complete** (optimistic `set_default_model_inner` updates badge; PersistSetting/SwitchModel are spawned tasks). — **plausible; unverified — needs repro**
  6. **Env override `NEXT_CODE_MODEL` / `MODEL`** forces model on every `Config::load`. — **verified** code exists; **ruled out for this machine** (env unset in check)
- **Ruled out (for this LOOK):**
  - “ACP `set_session_model` is the only path and never touches config” — **false**: daemon `apply_set_model` persists on success (`provider_control.rs`).
  - “PR10 never wired default model” — **false**: PR10 wired `set_default_model` → `[provider].default_model`; gap is **provider pair + stub + dashboard**.
  - “Cold-start badge only from session history, ignoring config” — **partially false**: welcome status reads `Config.provider.default_model`; in-session Overview badge comes from ACP History `provider_model` after daemon apply.
- **Sub-agents used:** skipped — narrow Face/daemon seam; DeepWiki/Exa MCP unavailable (used WebSearch/WebFetch + local tree).
- **Citations checked:** listed under Evidence.

## Evidence

### 1) Select path (ArgPicker → effects)
| Claim | Status |
|-------|--------|
| Bare `/model` opens ArgPicker (`OpenModelPicker`) | **verified** `slash/commands/model.rs:86-88`, `dispatch/settings/ui.rs:145-188` |
| ArgPicker confirm builds `/model <insert_text>` → `SendSlashCommandPreservingDraft` | **verified** `app/modals.rs:762-764` |
| Non-effort pick → `Action::SetDefaultModel` | **verified** `model.rs:94-96` |
| Effort pick → `Action::SwitchModel` only (session effort; not SetDefaultModel) | **verified** `model.rs:98-117`, `router.rs:845-863` |
| `SetDefaultModel` emits `PersistSetting(default_model)` then `SwitchModel` | **verified** `setters.rs:1720-1749`; unit test `dispatch/tests/settings.rs:266-294` |

### 2) Persist writers
| Claim | Status |
|-------|--------|
| `PersistSetting` → `set_default_model` → `[provider].default_model` via `xai_grok_config::set_toml_key` | **verified** `effects/helpers.rs:896-902`, `xai-grok-shell/.../config.rs:1014-1021` |
| Face writer does **not** set `default_provider` | **verified** (only `default_model` key) |
| ThemeKind (PR10) uses same `set_*` / `set_toml_key` family under `[ui].theme` | **verified** `config.rs:1086-1088`; PR10 plan `docs/plans/PLAN-20260720-grok-pr10-face-config-settings.md` |
| Stock Grok persists `[models].default` via `set_default_model` → `persist_models_default` | **verified** grok-build `settings_writes.rs` + user-guide `[models] default` |
| next-code `persist_models_default` is stub no-op | **verified** `xai-grok-shell/.../config.rs:1394-1398` |
| `SwitchModelComplete` still calls `PersistPreferredModel` → stub | **verified** `lifecycle.rs:1083-1086`, `effects/mod.rs:1837-1853` |
| ACP `set_session_model` → daemon `Request::SetModel` only at Face agent | **verified** `pager_agent.rs:1406-1426` |
| Daemon `apply_set_model` on success calls `Config::set_default_model(active, provider_key)` | **verified** `provider_control.rs:537-556` |
| Legacy TUI `/model` also persists model **and** `provider_key` | **verified** `model_context.rs:1337-1345` |

### 3) Dashboard carve-out
| Claim | Status |
|-------|--------|
| Dashboard `/model` / SetDefaultModel **stages only**, no global persist | **verified** `dispatch/dashboard.rs:1457-1467` |

### 4) Cold start / badge source
| Claim | Status |
|-------|--------|
| New Face session model state from daemon History `provider_model` + `available_models` | **verified** `pager_agent.rs:116-178`, `233-248` |
| Provider startup applies `config.default_model` + `default_provider` via `set_config_default_model` | **verified** `provider/startup.rs:302-312`, `provider/mod.rs:1369-1470` |
| Welcome / product status model line reads `Config::load().provider.default_model` | **verified** `face_welcome_status.rs:306-313` |
| Operator disk currently: `default_model = "deepseek-v4-flash"`, `default_provider = "opencode-go"` | **verified** (read `~/.next-code/config.toml` this LOOK) |
| Env `MODEL` / `PROVIDER` override on load | **verified** `env_overrides.rs:698-707`; unset on this host |

### 5) Repro path
| Claim | Status |
|-------|--------|
| Select A → quit → `nextcode` → still B | **unverified — needs operator repro** + before/after `config.toml` + whether pick was on dashboard vs agent + SetModel success toast |

## Feature planning / copy-wire-delete map

| Kind | Do | Do not |
|------|----|--------|
| **Wire** | Make Face default-model persist write the same pair as daemon/TUI (`default_model` + `default_provider`), ideally one writer (`Config::set_default_model` or equivalent) | Invent a second Face-only config schema |
| **Wire / fix stub** | Implement `persist_models_default` as alias to next-code provider default persist, **or** stop emitting `PersistPreferredModel` in embed | Leave silent `Ok(())` stub that looks like success |
| **Product decision** | Dashboard: keep stage-only **or** also persist global default (document either way) | Surprise users who pick on dashboard then quit |
| **Copy** | Keep Face ArgPicker / `/model` UX from PR #57 | Port legacy TUI model picker UI |
| **Delete** | Dead dual-write once one authoritative path exists | Stack more compensating sleeps / “remind to /model again” |

## Steps (simple checklist)
1. [x] Operator repro deferred — implemented from verified seams.
2. [x] Dashboard: persist global default (product choice for next-code).
3. [x] Route Face persist through atomic `[provider].default_model` + `default_provider` writer (`set_provider_defaults`).
4. [x] Fix `persist_models_default` stub → `persist_models_default_with_provider`.
5. [x] Regression: unit tests for pair write + SetDefaultModel PersistPreferredModel.
6. [ ] Rebuild/install both aliases; smoke select → quit → reopen.

## Implementation notes (2026-07-22)

| Change | Where |
|--------|--------|
| **Gate fix** | `process_chat_mode_enabled()` stub → `false` so SetDefaultModel persist runs |
| Atomic pair writer | `xai_grok_config::set_provider_defaults` / `_at` |
| Shell setters | `set_default_model` → pair API; `persist_models_default_with_provider` |
| Agent `/model` | `SetDefaultModel` emits `PersistPreferredModel` (+ provider from Overview float) instead of model-only `PersistSetting` |
| Switch complete | `PersistPreferredModel` carries `provider_key` |
| Dashboard `/model` | stages **and** emits `PersistPreferredModel` (model only; preserves existing provider pin) |

## Files touched
- `crates/xai-grok-config/src/lib.rs`
- `crates/xai-grok-shell/src/util/config.rs`
- `crates/xai-grok-pager/src/app/actions.rs`
- `crates/xai-grok-pager/src/app/effects/mod.rs`
- `crates/xai-grok-pager/src/app/dispatch/settings/setters.rs`
- `crates/xai-grok-pager/src/app/dispatch/session/lifecycle.rs`
- `crates/xai-grok-pager/src/app/dispatch/dashboard.rs`
- `crates/xai-grok-pager/src/app/dispatch/tests/settings.rs`
- `crates/xai-grok-pager/src/app/dispatch/tests/task_result.rs`

## Open questions (resolved for implement)
1. Dashboard `/model` → **persist global default** (next-code product).
2. Authoritative Face writer → `PersistPreferredModel` + `set_provider_defaults` (toml_edit; preserves `[ui]`).

## If you want more detail
Branch LOOK: `pr-face-model-select` @ `35b3eae7b`. PR #57 is palette UX; persist wiring is PR10 + daemon `apply_set_model`. Stock Grok persists `[models].default` in `~/.grok/config.toml`; next-code remapped Face settings to `~/.next-code` `[provider].default_model` but left provider pairing and `persist_models_default` incomplete.
