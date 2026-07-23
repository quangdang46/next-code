# BUG/LOOK — `/connect` “không thấy model UI như OpenCode”

## Verdict

**Install is not old.** Local `nextcode --version` = `v0.32.0-dev (c8ffc3342)`, which is **after** #61 (`168cd5993`) and includes `4e16be97d` (OpenCode-style `/connect` picker). `origin/dev` tip `de04a0a4d` is only #71 (model catalog cold-start) on top.

**Expectation mismatch, not missing binary:** #61 shipped **provider + auth-method** Face ArgPicker — **not** a model catalog inside `/connect`. Model list is **`/model`** (Ctrl+M). OpenCode’s post-connect jump to `DialogModel` was **deferred** (UX-3 / plan Q3).

## What they should see

| Command | UI |
|---------|-----|
| `/connect` (bare, Enter) | ArgPicker title **“Connect a provider”** — sections **Popular** + **Providers** |
| Pick multi-method family (e.g. Claude) | Second step **“Select auth method”** |
| After pick | Face paste/API-key or OAuth/browser chrome (`face_auth`) |
| After success | Select-model ArgPicker (OpenCode `DialogModel` parity) + toast “Connected — pick a model” |
| `/model` | Searchable **“Select model”** palette (also Ctrl+M; #71 fixed empty cold-start) |

**How to open:** Face session → type `/connect` alone → Enter → provider → auth → model list opens.

## Verified

| Claim | Evidence |
|-------|----------|
| #61 merge on `origin/dev` | `168cd5993` ancestor of `de04a0a4d` |
| Connect UX commit | `4e16be97d` — `OpenConnectPicker`, Popular/Providers, method step |
| Bare `/connect` opens picker | `connect.rs` `connect_run` → `Action::OpenConnectPicker`; `dispatch_open_connect_picker` |
| Binary includes feature | `nextcode --version` → `c8ffc3342`; ancestor of `4e16be97d` |
| OpenCode post-success → model | Plan cites OpenCode `dialog.replace(DialogModel)` |
| Face gap (pre-fix) | Same plan: “Post-connect → Jump to model select” **not** shipped; UX-3 deferred |

## Gap vs OpenCode (fixed)

```text
OpenCode: /connect → provider → method → key/OAuth → DialogModel
Face:     /connect → provider → method → key/OAuth → OpenModelPicker
```

## Implementation

1. `Action::NextCodeConnect` sets `open_model_picker_after_auth`.
2. `handle_auth_complete` (after restore / startup drain) calls `maybe_open_model_picker_after_connect` → toast + `Action::OpenModelPicker`.
3. Flag cleared on AuthFailed / CancelLogin; plain `/login` does not set it.
4. Syncs non-empty `app.models` into agent/dashboard before opening picker.

**Status:** Implemented.
