# Plan Report — PR10 Face config / settings / slash brand

## Summary (read this first)
- **You asked:** Settings persist in next-code; slash/brand still Grok — fix via wire + clean.
- **What is going on:** Face settings call shell `set_*` no-ops. Slash catalog is stock Grok (`/usage`→grok.com, `/gboom`, `/imagine`, Grok login flows). Quit hint already says `nextcode` (PR8); slash does not.
- **We recommend:** Keep **Face** settings/slash **UI**. **Wire** config to next-code. **Delete/hide** xAI-only slash and grok.com entry points; remap shared commands (`/model`, `/theme`) to next-code providers. Prefer Face UX over rebuilding old TUI settings.
- **Risk:** Medium  
- **Status:** After PR9 for model-affecting paths; slash hide can start earlier.

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

## Evidence (fill before BUILD)

| Claim | Citation | Status |
|-------|----------|--------|
| `set_*` mostly `Ok(())` | `xai-grok-shell/src/util/config.rs` | verified (pre-audit) |
| `/usage manage` opens grok.com | `slash/commands/usage.rs` + tests in `slash/commands/mod.rs` | verified (pre-audit) |
| Face still registers Grok-oriented slash set | `slash/commands/*.rs` | verified (pre-audit) |
| next-code equivalent keys for theme/model | config types | unverified — needs matrix |

## Slash brand matrix (complete during LOOK)

| Slash | Keep for nextcode? | Action |
|-------|-------------------|--------|
| `/model`, `/theme`, `/help`, `/new`, `/resume`… | Yes (generic) | Wire to next-code |
| `/usage` `/cost` | Maybe | Wire to next-code usage **or** hide if none |
| `/login` `/logout` | Yes but next-code | Wire daemon login; **no** Grok OAuth |
| `/gboom`, `/imagine`, `/imagine_video`, announcements→xAI | No | Hide/restrict in embed |
| `/usage manage` → grok.com | No | Delete URL / replace docs link |

## Copy / wire / delete
| Action | What |
|--------|------|
| **Wire** | Config R/W + model catalog |
| **Wire** | Generic slash → next-code behavior |
| **Delete** | xAI-only slash from embed registry (restrict list / feature) |
| **Delete** | grok.com billing/docs links in embed |

## Implementation steps
1. [ ] Call-site matrix Face symbol → next-code key (Evidence).
2. [ ] Read path then write path + reload hooks.
3. [ ] Slash brand matrix → restrict/hide list for embed (`nextcode` argv0 or env).
4. [ ] Tests: 3–5 config keys + restricted slash not in menu.
5. [ ] Manual: theme/model persist; `/` menu has no grok.com / gboom.

## Files
- `xai-grok-shell` config stubs  
- Face slash registry restrict (prefer embed flag over editing every command)  
- Composition root if callbacks needed  

## Manual verify
1. Theme/model survive restart.  
2. Slash menu: no grok.com / imagine / gboom (or documented exceptions).  
3. `/login` does not open Grok OAuth.

## Open questions
1. Keep `/usage` pointing at next-code billing, or hide until product has one?  
2. Restrict slash via registry API already used for tier deny — reuse?  
3. ACP skills slash collision with builtins?

## Out of scope
TUI crate delete (PR11), stub git/trust (PR12).

## Done when
Daily settings persist; embed slash/brand is nextcode-safe; Face UI kept.
