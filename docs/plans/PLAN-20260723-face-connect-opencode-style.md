# PLAN: Port opencode connect provider UI to Face

## Summary

Our Face `/connect` modal differs significantly from opencode's. OpenCode uses a **dialog stack** (centered modal with backdrop) that seamlessly transitions through provider picker → auth method → model selection. Our Face switches from a small ArgPicker → full-screen Welcome screen → separate ArgPicker. This plan brings the Face /connect flow closer to opencode's UX while staying within Face's existing `ArgPicker` + `ModalWindow` infrastructure.

**Risk:** Low (no ACP/daemon changes; only Face presentation)
**Status:** Waiting for user approval

## What opencode does (researched)

| Aspect | OpenCode | Our Face |
|--------|----------|----------|
| Modal container | Centered dialog with backdrop (z-stack), 60/88/116 col widths | ArgPicker with `ModalWindow` frame (bordered popup, centered) |
| Provider list | `DialogProvider` — Popular + Providers sections, fuzzy search | `ArgPicker` — same sections, fuzzy search (already close!) |
| Auth flow | Inline in dialog: `dialog.replace(<ApiMethod>)` | Full-screen `Welcome` view (view switch, not in modal) |
| Model picker | `DialogModel` — favorites + recent + provider groups | `ArgPicker` for models — flat list with provider headers |
| Transition | `dialog.replace(<NextStep>)` — no flicker | View switch to Welcome → auth → back to ArgPicker for models |
| Status indicators | Checkmark `✓` on already-connected providers | No connected-state indicators in connect picker |
| Favorites/recents | Persisted model favorites, recent models | No favorites, no recents |
| Footer actions | Tab/Shift+Tab for contextual actions (Connect provider, Favorite) | No footer actions |
| Search | `fuzzysort` with weighted scoring (title 2x category) | Basic substring/fuzzy (existing picker infrastructure) |

## What to implement (3 phases)

### Phase A: Inline auth in modal (P0)

Instead of switching to the full-screen Welcome view for auth, render the auth UI **inside** the existing picker modal.

**Current flow:**
```
ArgPicker (connect) → view switch → Welcome (full-screen) → auth OK → view switch → ArgPicker (model)
```

**Target flow:**
```
ArgPicker (connect) → Enter → replace modal content with auth prompt (inline)
                    → auth OK → replace modal content with ArgPicker (model)
```

**What moves:**
| File | Change |
|------|--------|
| `crates/xai-grok-pager/src/views/welcome/mod.rs` | Extract auth-step rendering (loopback paste, device code, OAuth waiting) into a reusable render block. Keep Welcome screen only for `--resume` / first-launch experience. |
| `crates/xai-grok-pager/src/app/dispatch/settings/ui.rs` | `dispatch_open_connect_picker` → when provider selected and auth starts, keep modal open but swap content to auth-inline view |
| `crates/xai-grok-pager/src/views/modal.rs` | Add `ActiveModal::AuthInline { provider, auth_state, method_id }` variant |
| `crates/xai-grok-pager/src/app/agent_view/` | Route key events to auth-inline modal when active |
| `crates/xai-grok-pager/src/views/picker.rs` | Add `render_auth_inline_content()` that renders paste-box / OAuth-waiting / device-code UI |

**OpenCode analogue:** `DialogPrompt` for API key / auth code entry, `AutoMethod` for OAuth-with-progress.

### Phase B: Connected status indicators (P1)

Show which providers already have credentials in the connect picker.

| File | Change |
|------|--------|
| `crates/xai-grok-pager/src/slash/commands/connect.rs` | `build_connect_family_items()` → accept `connected_ids: &[&str]` param; mark `ArgItem::is_current` for already-authed providers |
| `crates/xai-grok-pager/src/app/dispatch/settings/ui.rs` | `dispatch_open_connect_picker` → pass current auth state to `build_connect_family_items()` |
| `crates/xai-grok-pager/src/views/picker.rs` | Render `✓` checkmark for `is_current` items in the picker entries |

**OpenCode analogue:** Checkmark `✓` gutter in `DialogProvider`.

### Phase C: Seamless auth → model picker transition (P1)

After auth succeeds, transition the modal content to the model picker **within the same modal** instead of closing and reopening.

| File | Change |
|------|--------|
| `crates/xai-grok-pager/src/app/dispatch/auth.rs` | `maybe_open_model_picker_after_connect` → swap `ActiveModal::AuthInline` with `ActiveModal::ArgPicker { command: "model" }` instead of view-switch |
| `crates/xai-grok-pager/src/slash/commands/model.rs` | Ensure `build_model_items()` can be rendered directly without `/model` slash command round-trip |

### Phase D: Footer actions (P2)

Add contextual footer bar to the ArgPicker for connect/model modals.

| File | Change |
|------|--------|
| `crates/xai-grok-pager/src/views/modal_window.rs` | `ModalWindowConfig.shortcuts` already exists — wire it for picker modals |
| `crates/xai-grok-pager/src/views/modal.rs` | Connect picker footer: `Tab Search · Esc Close` |
| `crates/xai-grok-pager/src/views/modal.rs` | Model picker footer: `Tab Connect provider · Ctrl+a Add favorites` |

**OpenCode analogue:** Tab/Shift+Tab footer actions in `DialogSelect`.

## Not doing (deliberate scope cut)

| Feature | Rationale |
|---------|-----------|
| Model favorites/recents persistence | Requires cross-session state. Worth its own plan. |
| PromptsMethod (conditional auth metadata fields) | No provider in our 46-element catalog requires this yet. Add when needed. |
| Selection-aware backdrop dismiss | Mouse events are currently alpha; revisit when mouse support stabilizes. |
| Toast notifications for auth errors | We already have `show_toast`; auth errors already use it. Keep. |

## Files to touch

### Face layer (crates/xai-grok-pager)
| File | Phase | What |
|------|-------|------|
| `src/views/modal.rs` | A | Add `ActiveModal::AuthInline` variant |
| `src/views/modal_window.rs` | D | Ensure shortcuts render for picker modals |
| `src/views/picker.rs` | A,B | `render_auth_inline_content()`, `is_current` rendering |
| `src/app/dispatch/settings/ui.rs` | A,B,C | Wire auth-inline to dispatch; pass connected state; model picker transition |
| `src/app/dispatch/auth.rs` | C | `maybe_open_model_picker_after_connect` → modal swap |
| `src/app/dispatch/router.rs` | A | Route `AuthComplete` while in inline modal |
| `src/app/actions.rs` | A | Add `SetAuthModalContent` or equivalent action |
| `src/app/agent_view/mod.rs` | A | Handle key input for `AuthInline` modal |
| `src/slash/commands/connect.rs` | B | Accept `connected_ids` param in `build_connect_family_items()` |
| `src/slash/commands/model.rs` | C | Export `build_model_items()` for direct reuse |

### Views/welcome
| File | Phase | What |
|------|-------|------|
| `src/views/welcome/mod.rs` | A | Extract auth-step render functions for reuse in inline modal |

### No changes needed
- `crates/next-code-provider-metadata/` (catalog stays as-is)
- `crates/next-code-app-core/` (ACP wire unchanged)
- `src/cli/` (daemon/launcher unchanged)
- Any `next-code-tui` legacy code

## How to verify

1. `/connect` → centered modal with provider list → select provider
2. Auth UI renders **inside the modal** (not full-screen Welcome)
3. Paste API key or complete OAuth → modal shows model picker
4. Already-connected providers show `✓` in the picker
5. Footer shows contextual shortcuts
6. `NEXT_CODE_LEGACY_TUI=1` (if wired) still shows legacy Welcome for `--resume`

## Open questions

1. How much of the Welcome screen rendering is shared vs duplicated? Should we keep `ActiveView::Welcome` for first-launch but inline for `/connect`?
2. The `AuthInline` modal needs to handle polling for OAuth URL and auth-code paste — does the existing `dispatch_login` polling fit without view-switch?
3. For `is_current` indicators: do we query auth state synchronously from the agent's `auth_methods` or via ACP?
