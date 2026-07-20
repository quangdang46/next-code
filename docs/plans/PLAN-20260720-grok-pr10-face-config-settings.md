# Plan Report — PR10 Face config / settings / slash

## Summary (read this first)
- **You asked:** Wire Face settings UI to next-code real config (not shell no-ops).
- **What is going on:** Face settings/slash call `xai_grok_shell::…::set_*` which are mostly `Ok(())` stubs (`crates/xai-grok-shell/src/util/config.rs`). User toggles appear to work then vanish; model/theme may not match daemon.
- **We recommend:** Shim Face config writes/reads → next-code `config.toml` / `next-code-config-types` (and daemon reload if needed). Keep Face UI; replace stub bodies. Prefer Grok Face settings UX over old next-code-tui settings screens.
- **Risk:** Medium (key name mismatches, hot-reload)
- **Status:** Ready to implement after PR9 (can start research in parallel).

## Goal for this PR
Changing theme / model / common toggles in Face persists under `~/.next-code` (or `NEXT_CODE_HOME`) and affects the next turn.

## Research first (LOOK)
1. Face settings views: `crates/xai-grok-pager/src/settings/`, slash commands that call shell config.
2. List which `set_*` / `load_*` Face actually calls (rg from pager).
3. next-code config schema: `next-code-config-types`, `~/.next-code/config.toml` docs.
4. grok-build: how stock persists `[ui]` vs our `[display]` (PR3 already mapped home).

## Copy / wire / delete
| Action | What |
|--------|------|
| **Wire** | Stub `set_*`/`get_*` → next-code config store |
| **Wire** | Model catalog source → next-code provider catalog |
| **Delete** | Do not delete Face settings UI |

## Implementation steps
1. [ ] Build a call-site matrix: Face symbol → next-code config key.
2. [ ] Implement read path first (`load_effective_config` / remote settings DTO already partially stubbed).
3. [ ] Implement write path + optional broadcast `models-updated` / config reload (see `src/cli/startup.rs` patterns).
4. [ ] Slash: keep generic; no-op or hide xAI-only commands (document list).
5. [ ] Tests for mapping round-trip of 3–5 keys (theme, model, yolo/permission default).
6. [ ] Manual: toggle theme, restart Face, theme stuck; switch model, next reply uses it.

## Files (expected)
- `crates/xai-grok-shell/src/util/config.rs` (and related loaders)
- Possibly thin adapter in `src/cli/` if shell must not depend on app-core (respect crate layering — use traits/callbacks registered from composition root like existing `on_config_reloaded`)
- `docs/grok-migration-SUMMARY.md` note which keys are live

## Manual verify
1. Face → settings → change theme → quit → reopen → theme kept.
2. Change default model → new session uses it.
3. YOLO / default permission if exposed → survives restart.

## Out of scope
- Full marketplace / plugin install (stub OK)
- Voice settings

## Done when
Top Face settings that users hit daily persist in next-code config; stubs remain only for unused Face surfaces.
