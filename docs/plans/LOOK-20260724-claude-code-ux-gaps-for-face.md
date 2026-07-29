# LOOK — Claude Code UX gaps Face can still learn from

**Date:** 2026-07-24  
**Scope:** Research / docs only — no product implementation.  
**Sources:** local `.tmp-research-plugins/claude-code/` · Exa (Claude interactive mode / hooks / docs) · next-code Face plans under `docs/plans/` · Face slash inventory `PLAN-20260721-slash-commands-grok-vs-nextcode.md`

---

## Already tracked (do not re-plan as primary)

| Item | Tracker |
|------|---------|
| AskUserQuestion multi-question / checkbox / chips | [PR #83](https://github.com/quangdang46/next-code/pull/83) — implement in progress |
| Agent team / subagent panel Claude UX | [PR #85](https://github.com/quangdang46/next-code/pull/85) — full implement in progress |
| Background tasks pill + `/tasks` hub | [PR #84](https://github.com/quangdang46/next-code/pull/84) — full implement in progress |
| Cursor multitask MVP | [PR #82](https://github.com/quangdang46/next-code/pull/82) — Cursor, not Claude |
| Connect paste dialog (OpenCode) | [PR #86](https://github.com/quangdang46/next-code/pull/86) |

**Related but not this survey’s primary list:** Face permission-confirm ACP wire (`PLAN-20260722-face-permission-confirm-wire.md`); hooks Face wire / OpenCode compare (`PLAN-20260721-hooks-follow-opencode.md`); legacy-TUI statusline drafts (`status-bar-codex-claude.md`, `status-bar-improvement.md`); older DCG permission roadmap (`permission-improvement.md`).

---

## High-value gaps (Claude has · Face weak / missing)

Top picks beyond #82–#86. “Why” is one line each.

1. **Permission-mode cycle as first-class chrome (Shift+Tab / Alt+M)** — Claude cycles `default` ↔ `acceptEdits` ↔ `plan` ↔ `bypassPermissions` with visible mode in the prompt chrome (`src/types/permissions.ts`, `PromptInput` → `cyclePermissionMode`). Face has ask / always-approve / auto / `/plan`, but not Claude’s one-key cycle + `acceptEdits` “edits free, shell still asks” muscle memory.

2. **Plan → execute gate (ExitPlanMode + plan artifact)** — **Shipped** in `PLAN-20260724-face-plan-execute-gate.md` / this PR: ExitPlanMode bridge + PlanApprovalView gate, plan.md write exception under DCG Plan, prePlanMode stash/restore, `/plan open` editor. Session **`/diff`** review remains a follow-on.

3. **Tool-specific permission cards** — Claude ships per-tool UIs (bash cwd/sandbox, file-edit diffs, etc. under `src/components/permissions/`). Face `permission_view` is generic Approve/Always/Reject — fine after ACP wire lands, still far from Claude’s contextual cards (`permission-improvement.md` Gap 1).

4. **Session `/diff` review dialog** — Claude `src/commands/diff` → `DiffDialog` / `DiffFileList` over message-derived edits. Face has no `/diff` slash module; next-code TUI `/diff` never got Face chrome.

5. **`/context` as API-true visualization** — Claude `ContextVisualization` analyzes the same view the model sees (compact boundary + microcompact) (`src/commands/context/context.tsx`). Face `/context` exists but is a lighter usage pane, not Claude’s colored grid / collapse-aware breakdown.

6. **Configurable statusline (segments + optional shell)** — Claude `BuiltinStatusLine` / `StatusLine` + `/statusline` agent-assisted setup (`src/commands/statusline.tsx`, `src/components/StatusLine.tsx`). Face welcome/status chrome improved; idle footer still not a Codex/Claude-style remappable statusline (drafts target legacy TUI).

7. **Remappable keybindings product** — Claude full schema + `/keybindings` (`src/keybindings/*`, `src/commands/keybindings`). Face has action registry + settings “keybindings” hints; not a user-owned binding file with validation/templates.

8. **`!` bash mode (immediate shell into transcript)** — Type `!cmd` to run locally and inject output without a model turn ([interactive mode ref](https://claudefa.st/blog/guide/mechanics/interactive-mode)). Face/next-code rely on model bash / tools — no first-class bang prompt mode.

9. **Sticky user-prompt header while scrolled** — Claude `StickyPrompt` / `FullscreenLayout` keeps the last user prompt pinned when reading history (`src/components/FullscreenLayout.tsx`). Face scrollback is strong; sticky prompt chrome is a clear Claude comfort gap.

10. **Typed memory browser (`/memory` + memdir)** — Claude memdir types (`user` / `feedback` / `project` / `reference`) + LocalMemory UI (`src/memdir/`, `src/commands/local-memory/`). Face has memory activity floats (`PLAN-20260721-face-info-widget-floats.md`); no Claude-like memory editor / taxonomy UX.

11. **Hooks as permission/policy UX (PermissionRequest + PostCompact)** — Claude hooks can answer permission dialogs with JSON `behavior: allow` / `setMode`, and reinject context after compact ([hooks guide](https://code.claude.com/docs/en/hooks-guide); events in tree). next-code `hooks.toml` is real but Face `/hooks` + PermissionRequest-class decisions + prompt/agent hooks are not at Claude depth (partially deferred in hooks plan).

12. **One-key thinking / effort muscle memory** — Claude documents Alt+T extended thinking + ultrathink culture in interactive mode. Face has `/effort` dropdowns; missing a single toggle that power users expect from Claude muscle memory.

---

## Medium / nice-to-have

| Gap | Claude cue | Face note |
|-----|------------|-----------|
| Prompt vim (Normal/Insert in input) | `/vim`, `src/vim/*` | Face `/vim-mode` is mostly scrollback nav, not Claude prompt editing |
| Ctrl+G external editor for long prompts | interactive mode | Face multiline; no standard “open $EDITOR for this prompt” |
| `/ide` + edit plan/file in IDE | `src/commands/ide`, `IdeStatusIndicator` | ACP/IDE bridge not a Face product surface |
| Output styles | `src/commands/output-style`, `outputStyles/` | Personas / response style partial; not Claude styles dir |
| Sandbox toggle | `/sandbox`, sandbox permission card | Sandbox crate stubbed for Face vendor; not a user toggle |
| Remote Control / Bridge / Desktop / Mobile | `src/bridge`, `/remote-control`, `/desktop`, `/mobile` | Out of Face core; distinct product |
| Autonomy / schedule / proactive ticks | `src/commands/autonomy*`, `proactive/`, `jobs/` | next-code overnight/swarm adjacent; Face chrome absent |
| Ultraplan / thinkback | `ultraplan.tsx`, `thinkback*` | Niche; skip until plan-mode gate is solid |
| Skill store / skill-learning | `skill-store`, `skill-learning` | Face `$skill` + `/skills` already on roadmap via plugins |
| Auto mode classifier opt-in dialog | `AutoModeOptInDialog.tsx` | Face `auto` gate exists; Claude’s classifier story differs |
| Tips / release-notes feed polish | LogoV2 tips, `/release-notes` | Welcome tips exist; not a Claude-style tip carousel priority |

---

## Explicit non-goals / already parity-ish

- **Already in flight:** AskUserQuestion chips (#83), agent teams (#85), background `/tasks` (#84), multitask (#82), connect paste (#86).
- **Face already has (wire/polish ≠ invent):** `/btw`, `/fork`, `/rewind`, `/compact`, `/resume`, `/plan` enter, `/model`+effort pickers, settings/theme modals, vim-ish scrollback, plugins/hooks Extensions chrome, MCP surfaces, tip/welcome status fields.
- **Do not chase as Face differentiators:** Claude buddy/companion sprites, stickers, poor-mode budget flags, mobile QR, Slack/GitHub app install commands, Claude Desktop glue.
- **Hooks rewrite to OpenCode TS-in-process:** already rejected; keep next-code hooks runtime (`PLAN-20260721-hooks-follow-opencode.md`).
- **Custom plugin UI inside Face:** frozen non-goal (`20260722-face-ui-hints-external-pane.md`).

---

## Suggested next 3 picks (after #82–#86 land)

1. **Permission-mode cycle + `acceptEdits` chrome** — Shift+Tab-class cycle, status glyph, and map Face modes cleanly onto next-code DCG (`default` / `acceptEdits` / `plan` / `bypass`). Unlocks muscle memory; pairs with permission-confirm wire already planned.
2. **Plan exit / review gate** — **Shipped** via `PLAN-20260724-face-plan-execute-gate.md` (ExitPlanMode + approve/revise/abandon + `/plan open` + prePlanMode). Session **`/diff`** review still follow-on.
3. **Statusline v1 for Face** — persistent idle segments (mode · model · context%) + `/statusline` setup; borrow Claude/Codex drafts but implement on Face footer, not legacy TUI.

**Runners-up if those three slip:** `/context` API-true viz · `!` bash mode · sticky prompt header · remappable keybindings file.

---

## Research footnotes (paths / URLs)

| Topic | Where |
|-------|--------|
| Permission modes | `.tmp-research-plugins/claude-code/src/types/permissions.ts` |
| Plan slash + file | `…/src/commands/plan/plan.tsx`, `utils/plans.js` (via imports) |
| Diff review | `…/src/commands/diff/`, `…/src/components/diff/` |
| Context viz | `…/src/commands/context/context.tsx`, `ContextVisualization.tsx` |
| Statusline | `…/src/commands/statusline.tsx`, `…/src/components/StatusLine.tsx` |
| Sticky prompt | `…/src/components/FullscreenLayout.tsx` |
| Memory | `…/src/memdir/`, `…/src/commands/local-memory/` |
| Keybindings | `…/src/keybindings/`, `…/src/commands/keybindings/` |
| Interactive shortcuts (Shift+Tab, `!`, Alt+T, Ctrl+B/T) | https://claudefa.st/blog/guide/mechanics/interactive-mode (2026-07) |
| Hooks PermissionRequest / PostCompact | https://code.claude.com/docs/en/hooks-guide |
| Face slash inventory | `docs/plans/PLAN-20260721-slash-commands-grok-vs-nextcode.md` |
| Permission dialog gaps (older) | `docs/plans/permission-improvement.md` |
