# Plan Report — Grok plugins vs next-code plugins

## Summary (read this first)
- **You asked:** Copy Grok plugins (Face UI + handle logic) into next-code, delete next-code TS plugins, in PR #49.
- **What is going on:** Face Extensions Plugins tab was brand-hidden and ACP-stubbed; next-code had a separate QuickJS/TS `plugin *` CLI.
- **We recommend / did:** **Copy (UI already vendored) → wire ACP to `~/.next-code` → delete TS plugin product.** Unhide `/plugins` + `/hooks`; keep marketplace hidden; keep `/skills` + `$skill`.
- **Risk:** Medium (install/git/path edge cases; hooks tab list-only).
- **Status:** Implemented on `pr-face-config-settings` — user override of prior “No / Partial” research.

## Verdict table (updated)

| Question | Answer |
|----------|--------|
| Dung Grok plugins UI, xoa next-code plugins? | **Yes** (user override) — Face modal + next-code ACP body; delete TS CLI/runtime. |
| Copy Face modal → wire `.next-code`? | **Done** — `src/cli/face_plugins.rs` |
| Hide until rewrite? | **No longer** — `/plugins` + `/hooks` unhidden; marketplace still hidden. |
| Delete next-code plugins now? | **Done** — CLI + `next-code-plugin-core` / `runtime` crates removed. |

## Copy / wire / delete map

| Kind | What |
|------|------|
| **Copy** | Face Extensions Plugins/Hooks modal + effects already in `xai-grok-pager` |
| **Wire** | `x.ai/plugins/list\|action`, `x.ai/hooks/list\|action` in `pager_agent` → `face_plugins` under `~/.next-code` |
| **Delete** | `next-code plugin *` CLI; `next-code-plugin-core` / `runtime` / `ext-hello`; orphaned TUI `plugin_integration.rs`; TS docs redirected |

## Leftover stubs (honest)

| Item | Status |
|------|--------|
| `xai-grok-shell::plugin` | Still compile stub (Face embed uses next-code ACP, not shell) |
| `xai-grok-agent::plugins` install_registry / git_install | Still stub — unused by embed path |
| `xai-grok-plugin-marketplace` | Stub; marketplace slash still brand-hidden |
| Face Hooks **action** | Returns `Unsupported` (list works) |
| Plugin hooks / MCP from bundles | Counted in list UI; execution not fully ported from Grok SessionActor |

## Evidence
1. `src/cli/face_plugins.rs` — discovery + ACP payloads
2. `src/cli/pager_agent.rs` — method dispatch
3. `crates/xai-grok-pager/src/product_welcome.rs` — brand list without plugins/hooks
4. `docs/plugins.md` — product docs

## Manual smoke
1. Restart Face (`nextcode` / `next-code`).
2. `/plugins` → Extensions modal Plugins tab (not “unavailable”).
3. Drop a folder with `plugin.json` + `skills/foo/SKILL.md` under `~/.next-code/plugins/demo/`.
4. Reload list → see demo; Skills tab / `$` may show `foo` after skill reload.
5. Enable/disable from modal; confirm `~/.next-code/plugins-state.json`.
