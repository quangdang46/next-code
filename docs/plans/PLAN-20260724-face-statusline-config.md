# PLAN — Face configurable statusline (v1)

**Date:** 2026-07-24  
**Branch / worktree:** `pr-face-statusline-config` @ `next-code-worktrees/face-statusline-config`  
**Gap:** LOOK survey #6 — Claude `BuiltinStatusLine` / `/statusline` vs Face prompt chrome.

---

## Research summary

| Source | Finding |
|--------|---------|
| Claude `BuiltinStatusLine.tsx` | Idle chrome: **model · Context N% · (tokens) · Session/Weekly · cost**; separator ` │ `. |
| Claude `StatusLine.tsx` | Optional **custom shell** via `settings.statusLine.command` + JSON stdin. |
| Claude `/statusline` | Prompt command → agent-assisted setup (edits `~/.claude/settings.json`), not a native segment picker. |
| Face today | Prompt bottom border via `PromptInfo` / `render_info_line`: **model · mode flags** (+ usage warning). No remappable segments. |
| Older drafts | `status-bar-codex-claude.md` targeted **legacy TUI** — out of scope. |

**v1 product choice:** extend Face prompt info line (not legacy TUI). Claude density (mode · model · context%) without cloning brand or shell-command layer.

---

## Design

### Config (`~/.next-code` → `[ui.status_line]`)

```toml
[ui.status_line]
enabled = true
mode = true
model = true
context = true
cwd = false
git = false
order = "mode,model,context"   # optional reorder; unknown ids ignored
```

- Per-segment `Option<bool>`: unset → inherit default (mode/model/context on; cwd/git off).
- `order`: comma-separated segment ids; unset → default order filtered by visibility.
- Shell command / cost / rate-limit pills → **v2**.

### Chrome

Agent-view prompt info line builds left spans from selected segments:

| Id | Render |
|----|--------|
| `mode` | Existing plan / always-approve / auto flags |
| `model` | Model (+ effort) label |
| `context` | `N%` when context usage known |
| `cwd` | Basename of session cwd |
| `git` | Branch when available (best-effort; omit if unknown) |

Usage warnings remain independent (right/left warning path unchanged).

### UX

- **Settings → Appearance → Status Line** group: enable + per-segment toggles + order string.
- **`/statusline`**: open Settings; `on`/`off`; `reset`; `order mode,model,context`; `toggle <segment>`.

### Persist

Same Shared path as `info_floats`: mutate `app.current_ui.status_line`, `Effect::PersistSetting` → `set_ui_nested("status_line", …)`.

---

## Non-goals (v1)

- Legacy TUI `draw_status` rewrite.
- Custom shell statusline command.
- Parallel PR surfaces (keybindings, context viz pane, plan gate, agent team, background tasks).

---

## Tests

- Parse / defaults / order filtering (unknown ids, empty → defaults).
- Render selection: which labels appear for a given config + snapshot.
- Slash `/statusline` arg routing.

---

## Smoke

1. Rebuild + install from worktree.
2. `/statusline` → Appearance → Status Line.
3. Toggle **Context %** / reorder → prompt chrome updates live.
