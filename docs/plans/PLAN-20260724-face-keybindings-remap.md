# PLAN — Face keybindings file / remap

**Date:** 2026-07-24  
**Branch / worktree:** `pr-face-keybindings-remap` @ `next-code-worktrees/face-keybindings-remap`  
**Status:** implemented in this PR  
**Risk:** Low–medium — extends existing `ActionRegistry`; startup still works if the user file is missing or invalid (fall back to shipped defaults + warnings).

---

## Summary

Ship a Claude Code–style **user-owned keybindings file** for Face:

- Path: `~/.next-code/keybindings.json` (or `$NEXT_CODE_HOME` / `$GROK_HOME` via `grok_home()`)
- Schema: Claude-shaped `{ "bindings": [ { "context", "bindings": { key → action | null } } ] }`
- Defaults remain in `actions/defaults.rs`; user entries **merge on top** (key last-wins / null unbinds)
- `/keybindings` creates a template (if absent), opens `$EDITOR`, and reloads the registry on return; `/keybindings reload` reloads without editing
- Hardcoded demote / tasks-pane chords in touched surfaces route through the registry

---

## Evidence (verified)

| Claim | Where |
|------|--------|
| Claude file + `/keybindings` + merge (defaults then user; null unbind) | `.tmp-research-plugins/claude-code/src/keybindings/loadUserBindings.ts`, `template.ts`, `schema.ts`, `commands/keybindings/keybindings.ts`; docs https://code.claude.com/docs/en/keybindings |
| Face single source of truth is `ActionRegistry` / `ActionId` / `When` | `crates/xai-grok-pager/src/actions/{mod,defaults}.rs` |
| Registry rebuilt at startup from config-gated defaults | `app/event_loop.rs` → `ActionRegistry::defaults_with_config` |
| Config home | `xai_grok_pager_render::util::grok_home()` → `~/.next-code` |
| `$EDITOR` suspend pattern | `Action::SuspendForEditor` + event_loop suspend |
| Hardcoded chords to fix | `agent_view/panes.rs` (`Ctrl+B` tasks toggle); `views/agent.rs` demote hint `Ctrl+G` |
| Legacy TUI `[keybindings]` in `config.toml` | `next-code-config-types` — **out of scope**; Face does not use that table |

---

## Copy / wire / delete map

| Kind | What |
|------|------|
| **Wire** | User JSON → parse → mutate `ActionDef` keys → `ActionRegistry` |
| **Wire** | `/keybindings` slash + optional Settings keyword |
| **Wire** | Startup + post-editor reload |
| **Fix** | Demote hint + tasks-pane toggle use `registry.matches_id` / `find` |
| **Do not** | Invent a second Face keymap parallel to `ActionRegistry` |
| **Do not** | Touch legacy TUI `KeybindingsConfig` / `config.toml [keybindings]` |
| **Do not** | Merge into context-viz / plan-gate / agent-team / background-tasks / AskUser PRs |

---

## Schema (user file)

```json
{
  "$docs": "Face keybindings — contexts match ActionRegistry When; actions are snake_case ActionId names",
  "bindings": [
    {
      "context": "AgentScreen",
      "bindings": {
        "ctrl+h": "send_to_background",
        "ctrl+g": null,
        "ctrl+b": "toggle_tasks"
      }
    }
  ]
}
```

- **Contexts:** `Always`, `PromptFocused`, `ScrollbackFocused`, `AgentScreen`, `WelcomeScreen`, `DashboardFocused`, `DashboardOverlay`
- **Actions:** snake_case of `ActionId` (e.g. `send_to_background`, `toggle_tasks`, `select_next`)
- **Keystrokes:** `ctrl+g`, `shift+tab`, `alt+v`, `enter`, `esc`, … (same family as legacy `parse_keybinding`)
- **`null`:** unbind that key in that context
- Missing file → defaults only. Invalid JSON → defaults + warning (do not crash Face)

---

## Remappable (v1)

All actions registered in `default_actions()` for the contexts above — including demote (`send_to_background`), tasks pane (`toggle_tasks`), todos, queue, sessions, navigation, quit, etc.

Not remappable via this file (still hardcoded / modal-local): list-pane internal `g`/`b`, prompt readline chords that are not `ActionId`s, xt_filter BEL/`Ctrl+G` terminal quirks.

---

## Files to touch

- `docs/plans/PLAN-20260724-face-keybindings-remap.md` (this file)
- `crates/xai-grok-pager/src/actions/user_bindings.rs` (new)
- `crates/xai-grok-pager/src/actions/mod.rs`
- `crates/xai-grok-pager/src/input/key.rs` (parse shortcut string)
- `crates/xai-grok-pager/src/slash/commands/keybindings.rs` (new)
- `crates/xai-grok-pager/src/slash/commands/mod.rs`
- `crates/xai-grok-pager/src/app/actions.rs` + dispatch/router + event_loop reload
- `crates/xai-grok-pager/src/app/agent_view/panes.rs`, `views/agent.rs`
- `crates/xai-grok-pager/src/settings/defs.rs` (keywords / short help)

---

## Test plan

- Unit: parse valid/invalid JSON; merge override; null unbind; lookup after merge
- Unit: `ActionId` / `When` round-trip names
- Manual smoke: edit binding → `/keybindings` or restart → chord changes

---

## Open questions (non-blocking)

1. Hot-reload via file watcher (Claude) vs explicit reload — v1 uses startup + `/keybindings` / post-editor reload.
2. Whether to publish a JSON Schema URL later (Claude uses schemastore); v1 documents `$docs` only.
