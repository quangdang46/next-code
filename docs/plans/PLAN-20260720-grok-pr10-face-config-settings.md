# Plan Report — PR10 Face config / settings / slash brand

## Summary (read this first)
- **You asked:** Settings persist in next-code; slash/brand still Grok — fix via wire + clean.
- **What is going on:** Face settings call shell `set_*` no-ops. Slash catalog is stock Grok (`/usage`→grok.com, `/gboom`, `/imagine`, Grok login flows). Quit hint already says `nextcode` (PR8); slash does not.
- **We recommend:** Keep **Face** settings/slash **UI**. **Wire** config to next-code. **Delete/hide** xAI-only slash and grok.com entry points; remap shared commands (`/model`, `/theme`) to next-code providers. Prefer Face UX over rebuilding old TUI settings.
- **Risk:** Medium  
- **Status:** **Implemented (PR10 slice)** — see § Implementation notes below.

## Workflow map (required)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | Keep Face settings screens + slash palette UX | Port old `next-code-tui` settings UI back |
| **Wire** | `set_*`/`load_*` → `~/.next-code` config + provider catalog | Leave stubs that pretend to save |
| **Delete / clean** | Hide or no-op xAI slash; replace grok.com URLs; drop Grok OAuth `/login` path in embed | Delete Face `/model` command itself |

## Research first (LOOK)
1. Face: `settings/`, `slash/commands/*`, registry.
2. rg pager → `xai_grok_shell::` set_/load_.
3. next-code config + provider catalog.
4. grok-build: which slash are xAI-only vs generic.

## Evidence (filled)

| Claim | Citation | Status |
|-------|----------|--------|
| `set_*` mostly `Ok(())` | `xai-grok-shell/src/util/config.rs` | was verified; **now wired** to toml_edit |
| `/usage manage` opens grok.com | `slash/commands/usage.rs` | gated off in nextcode embed |
| Face still registers Grok-oriented slash set | `slash/commands/*.rs` | restricted via `EMBED_BRAND_RESTRICTED_COMMANDS` |
| ThemeKind persist in `[ui].theme` | `xai-grok-config` load/set + theme cache | verified + wired |
| Face ThemeKind ≠ origin dark/light | product decision | verified — no remap |

## Slash brand matrix

| Slash | Keep for nextcode? | Action |
|-------|-------------------|--------|
| `/model`, `/theme`, `/help`, `/new`, `/resume`… | Yes | Keep UI; model via ACP History; theme via `[ui]` |
| `/usage` `/cost` | Show only | manage→grok.com stripped in embed |
| `/login` `/connect` | Yes | Face picker + CLI login instructions (partial auth) |
| `/gboom`, `/imagine`, `/imagine-video`, announcements, marketplace, plugins, hooks, privacy | No | `EMBED_BRAND_RESTRICTED_COMMANDS` |
| `/docs web` → docs.x.ai | No in embed | Error + hide suggest |

## Implementation notes (2026-07-21)

### Landed
1. **Brand restrict:** `product_welcome::EMBED_BRAND_RESTRICTED_COMMANDS` merged in `AppView::apply_tier_restrictions` when `is_nextcode_embed()`.
2. **URL gates:** `/usage manage` and `/docs web` refuse xAI URLs in embed.
3. **Config persist:** `load_effective_config_disk_only` + shell `set_*` write `[ui].*` (ThemeKind ids) and `[provider].default_model` under `~/.next-code/config.toml` via toml_edit (preserves siblings). `collapsed_edit_blocks` resolve reads disk then defaults **true**.
4. **`/model`:** catalog already via `pager_agent` ACP History; default model setting persists to `[provider].default_model`.
5. **`/connect` + `/login`:** Face `suggest_args` from `tui_login_providers()`; embed `/login` remapped away from Grok OAuth; credential write still via `next-code login <provider>` (partial).
6. **Skills:** `AvailableCommandsUpdate` with path/scope meta; `$skill` / Face `/skill` expand via `system_reminder` in `pager_agent`.
7. **Alias hazards documented:** `/clear`≡`/new`, `/log`≡transcript (Face meanings).

### Deferred
- Full interactive OAuth/API-key capture **inside** Face (no Grok OAuth; CLI completes write).
- Port swarm/overnight/selfdev slash set.
- Remap Face `/sessions` vs next-code resume alias beyond docs.

## Manual verify
1. Theme survive restart (`[ui].theme` in `~/.next-code/config.toml`).
2. Slash menu: no gboom/imagine/marketplace (restricted); `/usage manage` / `/docs web` no xAI URLs.
3. `/connect` / `/login` show provider dropdown; no Grok OAuth.
4. `/skills` + `$skillname` / `/skillname` activate skill content.
5. `collapsed_edit_blocks` stays denser default unless user flips.

## Done when
Daily settings persist; embed slash/brand is nextcode-safe; Face UI kept. — **met for approved slice**.
