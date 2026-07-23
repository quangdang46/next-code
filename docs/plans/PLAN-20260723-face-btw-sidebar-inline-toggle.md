# Plan Report — Face `/btw` output mode toggle (sidebar vs inline)

## Summary

Add a `/settings` config toggle so Face `/btw` can deliver answers either as:

| Mode | UX | Source of truth today |
|------|----|------------------------|
| **`inline`** | Stock Face / Grok overlay above the prompt (`btw_overlay`); Esc persists a collapsed `BtwBlock` into scrollback | Current Face + stock grok-build pager |
| **`sidebar`** | Legacy next-code TUI right-hand side panel (`title=/btw`, Alt+M hide, status copy “Running /btw - answer will appear in the side panel”) | `next-code-tui` only — **not** stock Grok Face |

**Terminology correction (verified):** the screenshot strings (`side /btw`, Alt+M, “Running /btw…”) come from **legacy next-code-tui**, not upstream Grok Face. Stock Grok Face already uses the compact **inline panel** (`btw_overlay`). Calling the TUI right pane “upstream Grok” is a mislabel; the intentional product delta is **opt-in TUI side-panel parity inside Face**.

**Default: `inline`** — matches stock Grok Face and preserves current Face behavior. `sidebar` is an explicit next-code product delta (port/adapt TUI side-panel chrome into Face).

- **Status:** Implemented

---

## Evidence

### 1) Stock Grok Face `/btw` (overlay, not right pane)

| Claim | Status | Citation |
|-------|--------|----------|
| Slash `/btw` → `Action::SendBtw` | verified | `crates/xai-grok-pager/src/slash/commands/btw.rs` (`BtwCommand::run`) |
| Dispatch sets `btw_state = Loading`, fires `Effect::SendBtw` | verified | `crates/xai-grok-pager/src/app/dispatch/notes.rs` (`dispatch_send_btw`) |
| ACP ext `x.ai/btw` with `{sessionId, question}` | verified | `crates/xai-grok-pager/src/app/effects/mod.rs` (`Effect::SendBtw`) |
| Response → `BtwOverlayState::Done` / `Error` | verified | `notes.rs` (`handle_btw_response`) |
| Render: compact bordered panel **above prompt** | verified | `crates/xai-grok-pager/src/views/btw_overlay.rs` module docs + `render_btw_panel` |
| Esc dismiss → scrollback `BtwBlock` | verified | `crates/xai-grok-pager/src/app/agent_view/viewer.rs` (`dismiss_btw_panel`) |
| Minimal mode hosts same overlay in live region | verified | changelog / `minimal/api.rs` (`start_minimal_btw` / `finish_minimal_btw`) |
| String “Running /btw - answer will appear in the side panel” in Face/Grok | **not found** | Exa + local Face tree |
| Alt+M hide for `/btw` in Face/Grok | **not found** | Face actions use Ctrl+T/B for todos/tasks panes; Alt+M is TUI |

DeepWiki (`xai-org/grok-build`): same overlay model; no separate SidePanel component for `/btw`; no Alt+M; no output-mode config toggle.

### 2) Current next-code Face wire (inline / overlay path)

| Claim | Status | Citation |
|-------|--------|----------|
| Daemon brain: `face_ext::btw_payload` tool-free `complete` over transcript | verified | `src/cli/face_ext.rs` (`btw_payload`) |
| Routed via `pager_agent` → `handle_ext_method("x.ai/btw")` | verified | `src/cli/pager_agent.rs` + `face_ext::handle_ext_method` |
| Face UI already paints overlay (PR65 / `PLAN-20260722-face-recap-btw-wire`) | verified | same pager paths as stock |

This **is** the stock Grok delivery surface. There is no separate “inline chat message as normal assistant turn” path for Face `/btw` today (answer is overlay → optional `BtwBlock` on dismiss).

### 3) Screenshot UX = legacy next-code-tui side panel

| Claim | Status | Citation |
|-------|--------|----------|
| `/btw` writes managed side-panel page `id=btw`, `title=/btw` | verified | `crates/next-code-tui/src/tui/app/commands.rs` (`handle_btw_command`, `BTW_PAGE_ID`) |
| Status: `Running /btw - answer will appear in the side panel.` | verified | same file L1392–1396 |
| Mid-turn: `/btw noted - answer will appear in the side panel.` | verified | same file L1387–1390 |
| Alt+M toggles side panel visibility | verified | `crates/next-code-tui/src/tui/keybind.rs` + tests `scroll_copy_01` |
| Help: “Ask a side question in the side panel” | verified | `state_ui_input_helpers.rs` / `input_help.rs` |
| Mechanism: hidden system reminder + `side_panel` tool write (not `x.ai/btw`) | verified | `build_btw_system_reminder` + `side_panel::write_markdown_page` |

### 4) Where Face `/settings` stores toggles

| Layer | Role | Citation |
|-------|------|----------|
| Catalog | `SettingMeta` in `settings/defs.rs` (`default_settings()`) | key, category, owner, `SettingKind::{Bool,Enum}`, defaults |
| Schema | `[ui].*` fields on `UiConfig` | `crates/xai-grok-shared/src/ui_config.rs` |
| Apply | `dispatch/settings/setters.rs` + `ui.rs` | live cache + `Effect::PersistSetting` → `config.toml` |
| Modal | `dispatch_open_settings` / `refresh_open_settings_modals` | `dispatch/settings/ui.rs` |
| Ownership | `SettingOwner::{Shared,Shell,Pager}` | Shared/Shell persist to `[ui]`; Pager often ephemeral |

Pattern copied: `screen_mode` (`fullscreen` \| `minimal`) — `SettingKind::Enum` + `UiConfig` field + setter + registry get/set arms.

---

## Config (shipped)

```toml
[ui]
# Face /btw answer surface.
# inline  = stock Face/Grok overlay above the prompt (default)
# sidebar = legacy next-code TUI right-hand side panel (product delta)
btw_output_mode = "inline"   # "inline" | "sidebar"
```

| Item | Value |
|------|--------|
| **Key** | `btw_output_mode` under `[ui]` |
| **Settings label** | `/btw` output |
| **Choices** | `inline` (“Overlay (Face / Grok)”), `sidebar` (“Side panel (legacy TUI)”) |
| **Category** | Appearance |
| **Owner** | `SettingOwner::Shared` |
| **Default** | **`inline`** |
| **Restart** | Live-apply (next `/btw`) |

---

## Implemented

| Kind | What |
|------|------|
| **Keep** | Face `BtwCommand` → `x.ai/btw` → `face_ext::btw_payload` brain (both modes share daemon) |
| **Keep** | `btw_overlay` + `BtwBlock` path as `inline` |
| **Wire** | Settings catalog + `UiConfig.btw_output_mode` + setter/persist |
| **Wire** | `dispatch_send_btw` branches on mode; sidebar toast + `btw_sidebar` stamp |
| **Wire** | Right-hand carve of scrollback (`AgentViewLayout::apply_btw_sidebar`); Alt+M hide; Esc dismiss |
| **Do not** | Re-implement Face `/btw` via hidden main-turn + `side_panel` tool |

## How to toggle / smoke

1. `/settings` → Appearance → **`/btw` output`** → Overlay vs Side panel
2. Or set `[ui] btw_output_mode = "sidebar"` in config.toml
3. Smoke: `/btw what is the cwd?` — overlay by default; with sidebar, toast + right panel; Alt+M hides; Esc dismisses to `BtwBlock`

## Worktree

- Path: `C:\Users\ADMIN\Documents\Projects\next-code-worktrees\pr-face-btw-output-mode`
- Branch: `pr-face-btw-output-mode`
