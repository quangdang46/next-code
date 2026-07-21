# Plan Report — Slash commands: Grok Face vs next-code

## Summary (read this first)
- **You asked:** Keep Grok Face slash **UI/UX** (beautiful pickers/dropdowns); inventory shared vs next-code-only vs Grok-only; map how to port next-code semantics (e.g. `/connect` provider dropdown) into Face chrome; note Grok-only candidates to bring back or hide for brand.
- **What is going on:** Face has a rich builtin registry (`builtin_commands()` ≈ 60 entries) with arg-suggest dropdowns (`suggest_args`), settings modal, model/theme pickers. next-code TUI has a larger product command set (`REGISTERED_COMMANDS` ≈ 90+ public) including multi-provider `/connect`/`/account`, swarm/overnight/selfdev, etc. PR10 already decided: **keep Face UI**, **wire** config/`set_*`, **hide/remap** xAI-only slash and grok.com links.
- **We recommend:** Treat Face slash palette + arg dropdowns as the presentation layer. For PR10: (1) restrict/hide xAI-only commands via existing `CommandRegistry::set_restricted_commands`, (2) wire shared commands (`/model`, `/theme`, `/settings`, `/login`) to next-code config/auth, (3) add a Face-chrome `/connect` (or remap `/login` args) that reuses Face picker patterns with next-code provider catalog — do **not** resurrect old TUI chrome. Defer bulk next-code-only ports (swarm, overnight, …) past PR10 unless they already have ACP/daemon hooks.
- **Risk:** Medium (auth remap + alias collisions: Face `/sessions`→dashboard vs next-code `/sessions`→resume; Face `/log`→transcript vs next-code `/log` mark).
- **Status:** **Implemented (PR10 slash restrict + wire slice)** — 2026-07-21. Full next-code-only port list still deferred.

## Implementation notes (2026-07-21)

| Item | Result |
|------|--------|
| Restrict list | `EMBED_BRAND_RESTRICTED_COMMANDS` + merge in `apply_tier_restrictions` |
| `/usage manage` / `/docs web` | Gated in embed |
| `/connect` + `/login` | Face `suggest_args`; CLI login for credential write (partial) |
| Alias hazards | Documented on `/new` (`/clear`) and `/transcript` (`/log`); Face meanings kept |
| `$skill` | `pager_agent::expand_skill_invocation` + ACP skill advertise |

Open questions from research — resolved for PR10:
1. **`/usage`:** keep show; strip manage URL in embed.
2. **`/connect`:** explicit Face command + `/login` alias in embed.
3. **Aliases:** accept Face meanings; document (no silent restore of TUI `/clear` history).

## Feature planning
- **Recommended approach:** Copy Face UX; wire next-code brain; delete/hide xAI brand surfaces. Prefer registry deny-list over editing every command file.
- **Prior art (GitHub):** Local tree is source of truth (vendored Face). Upstream grok-build slash set matches `builtin_commands()`; tier restrict already exists for `usage`/`imagine`/`imagine-video`/`voice`.
- **Integration points:**
  - Face registry: `crates/xai-grok-pager/src/slash/commands/mod.rs` → `builtin_commands()`
  - Face restrict API: `crates/xai-grok-pager/src/slash/registry.rs` → `set_restricted_commands`
  - Face tier list: `crates/xai-grok-pager/src/app/app_view.rs` → `TIER_RESTRICTED_COMMANDS`
  - Shell stubs: `crates/xai-grok-shell/src/util/config.rs` → `set_*` no-ops
  - next-code catalog: `crates/next-code-tui/src/tui/app/state_ui_input_helpers.rs` → `REGISTERED_COMMANDS`
  - next-code `/connect`: `crates/next-code-tui/src/tui/app/auth_account_commands.rs` → `show_interactive_login()`
- **Sub-agents used:** skipped — inventory is local-file verification (tiny research deliverable).
- **Option B:** Rebuild next-code TUI slash chrome inside Face — **rejected** by PR10 / grok-migration skill (copy/wire/delete).
- **Open questions:** see bottom (≤3).

## Evidence

| Claim | Citation | Status |
|-------|----------|--------|
| Face builtin list is `builtin_commands()` | `xai-grok-pager/src/slash/commands/mod.rs` L75–145 | verified |
| Face `/model` has chained model→effort `suggest_args` dropdown | `slash/commands/model.rs` L16–65 | verified |
| Face `/settings` opens modal; aliases `config`/`preferences`/`prefs` | `slash/commands/settings_cmd.rs` | verified |
| Face `/usage manage` opens `https://grok.com/?_s=usage` | `slash/commands/usage.rs` L62–63 | verified |
| Face `/docs web` opens `https://docs.x.ai/build/overview` | `slash/commands/docs.rs` L12 | verified |
| Face tier restrict: `usage`, `imagine`, `imagine-video`, `voice` | `app/app_view.rs` L552–553 | verified |
| Registry deny-list reusable for embed brand hide | `slash/registry.rs` L113–124, `set_restricted_commands` | verified |
| Shell `set_*` are no-ops (PR10 wire target) | `xai-grok-shell/src/util/config.rs` L722, L894+ | verified |
| next-code public slash catalog | `next-code-tui/.../state_ui_input_helpers.rs` `REGISTERED_COMMANDS` L39–188 | verified |
| next-code `/connect` opens interactive provider login picker | `auth_account_commands.rs` L4–6; help in `input_help.rs` L166–167 | verified |
| next-code `/login` is alias for `/connect` (not in `REGISTERED_COMMANDS` top-level) | completions treat `/login ` like `/connect ` (`state_ui_input_helpers.rs` L965–985); help text | verified |
| PR10 brand matrix (hide gboom/imagine; wire model/theme/login) | `docs/plans/PLAN-20260720-grok-pr10-face-config-settings.md` | verified |

### How next-code `/connect` dropdown worked (TUI)
1. Bare `/connect` → `App::show_interactive_login()` (provider picker overlay).
2. `/connect <provider>` → resolve via `provider_catalog::tui_login_providers()` and `start_login_provider`.
3. Autocomplete under `/connect `|`/login `|`/auth ` lists provider ids from the catalog (`state_ui_input_helpers.rs` L965–985).
4. `/login` is documented as backwards-compat alias for `/connect`; `/account` is the richer multi-account center.

### Face UI surfaces to reuse (not rebuild)
| Surface | Face path / mechanism | next-code semantics to plug in |
|---------|----------------------|--------------------------------|
| Slash palette + fuzzy match | `slash/mod.rs` `SlashController`, `matcher.rs` | Keep |
| Arg dropdown (`ArgItem`) | `SlashCommand::suggest_args` (see `/model`, `/theme`, `/usage`) | Port `/connect` provider list here |
| Model picker | `/model` + ACP `ModelState` | Wire to next-code provider/model catalog |
| Settings modal | `/settings` → `Action::OpenSettings` | Wire `set_*`/`load_*` → `~/.next-code` |
| Theme picker | `/theme` + preview | Wire theme persistence |
| Login flow | `/login` → `Action::Login` (stock Grok OAuth) | Remap to next-code multi-provider auth |

---

## Inventory 1 — Shared (both have — wire next-code semantics into Grok UI)

Canonical name = Face name when both exist. Wire status from PR10 plan.

| Command | Purpose | Face UI chrome | next-code TUI chrome | PR10 wire |
|---------|---------|----------------|----------------------|-----------|
| `/help` | List/browse commands | Face help browser | Text/help + `/help <cmd>` | Keep UI; content nextcode |
| `/quit` (`/exit`) | Exit app | Face quit | `/exit`/`/quit` | Keep (brand hints already nextcode in PR8) |
| `/new` (Face alias `/clear`) | Fresh session | Face `/new` | Separate `/clear` | Wire session reset; **alias caution**: Face `/clear`≡`/new`, next-code `/clear` is clear-history |
| `/model` (`/m`) | Switch model | Beautiful model+effort dropdown | Model picker + `@provider` routes | **Wire** catalog + persist |
| `/effort` | Reasoning effort | Arg dropdown | `/effort` + keybinds | **Wire** to next-code effort |
| `/compact` | Compact history | Face compact | `/compact` + modes | Wire daemon compact |
| `/resume` | Session picker | Face resume | Interactive session picker | Wire next-code sessions |
| `/rename` | Rename session | Face rename | `/rename` | Wire |
| `/fork` | Branch session | Face fork | Arm fork session | Wire if ACP supports |
| `/plan` | Plan mode | Face plan mode | Side-panel plan-only | Wire intent (semantics differ slightly) |
| `/btw` | Side question | ACP `x.ai/btw` | Side panel btw | Wire to next-code side channel; drop x.ai branding |
| `/context` | Context usage | Face pane | Full context snapshot | Wire; next-code richer |
| `/usage` (`/cost`) | Usage/billing | Show + **manage→grok.com** | Connected-provider usage | **Wire show**; **delete manage URL** or hide manage |
| `/feedback` | Send feedback | Face feedback mode | next-code feedback | Wire target (GitHub/issues, not xAI) |
| `/skills` | Skills UI | Face plugins/skills views | `/skills` list + `$` namespace | Wire ACP skills; avoid collision (PR10 Q3) |
| `/rewind` | Rewind turns | Face rewind | `/rewind` N / undo | Wire |
| `/transcript` | View transcript | Opens `$PAGER` (alias `/log`) | Opens transcript file | Wire; **alias collision** with next-code `/log` mark |
| `/export` | Export conversation | Face export | `/export` | Wire |
| `/agents` | Agent config | Face `/config-agents` alias `/agents` | `/agents` role-model picker | Wire next-code agent roles into Face modal if possible |
| `/changelog` | Release notes | Face `/release-notes` alias `/changelog` | `/changelog` | Keep Face UI; content nextcode |
| `/login` | Auth | Face Grok OAuth `Action::Login` | Alias of `/connect` multi-provider | **Remap** — no Grok OAuth |
| `/config` | Settings | Face alias of `/settings` modal | Show/edit `config.toml` | Prefer Face settings modal; wire disk |

---

## Inventory 2 — next-code only (port into Face UI — priority tiers)

**PR10 must-have (auth/config parity):**

| Command | Purpose | Suggested Face chrome | Notes |
|---------|---------|----------------------|-------|
| `/connect` | Interactive provider connect | Face arg-dropdown or settings-style picker listing `tui_login_providers()` | Primary port example; keep `/login` as alias |
| `/account` (`/accounts`) | Multi-account center | Face modal/picker (not old TUI overlay) | Larger than PR10; may follow connect |
| `/auth` | Auth status / doctor | Face status panel or `/connect` subcommand | Exists in help/completions, not `REGISTERED_COMMANDS` top-level |

**Post-PR10 candidates (product features; port when daemon/ACP ready):**

| Command | Purpose |
|---------|---------|
| `/refresh-model-list` | Force refresh model catalogs |
| `/provider-test-coverage` | Live-test evidence |
| `/subagent`, `/subagent-model` | Manual subagent launch/policy |
| `/observe`, `/todos`, `/splitview` | Side-panel modes |
| `/ssh`, `/git`, `/commit`, `/commit-push` | Git/SSH helpers |
| `/autoreview`, `/autojudge`, `/review`, `/judge` | Review/judge flows |
| `/fast`, `/transport`, `/alignment`, `/reasoning` | Session knobs |
| `/poke`, `/improve`, `/refactor`, `/fix` | Agent loops / recovery |
| `/dictate` | External STT (≠ Face `/voice`) |
| `/memory`, `/swarm`, `/overnight`, `/initiatives`/`/goals` | Feature toggles / coordinators |
| `/test`, `/diff`, `/keys`, `/cache` | Dev/ops |
| `/info`, `/version`, `/productivity` | Status / wrap report |
| `/reload`, `/restart`, `/rebuild`, `/selfdev`, `/update` | Binary lifecycle |
| `/active`, `/catchup`, `/back`, `/save`, `/unsave`, `/split`, `/transfer`, `/workspace` | Session workspace |
| `/permissions`, `/experiment` | DCG / flags |
| `/cancel` | Cancel turn |
| Remote: `/client-reload`, `/server-reload`, `/continue` | Remote mode |

**Hidden/secret next-code (do not port to Face menu):** `/z`, `/zz`, `/zzz`, `/zstatus`, deprecated `/models`.

---

## Inventory 3 — Grok only (bring logic to next-code **or** hide/remap for brand)

### Hide / restrict in nextcode embed (PR10 brand)

| Command | Purpose | Action |
|---------|---------|--------|
| `/gboom` | Hidden easter egg (kitty raycaster) | **Hide** (already `visible=false`; also restrict in embed) |
| `/imagine` | Image gen | **Hide/restrict** (also tier-restricted upstream) |
| `/imagine-video` | Video gen | **Hide/restrict** |
| `/announcements` | xAI announcements | **Hide** |
| `/docs` (+ `docs.x.ai`) | How-to + online Build docs | **Remap** docs URL to next-code docs **or** hide `web` |
| `/usage manage` → grok.com | Billing page | **Delete URL** / remove `manage` arg in embed |
| `/privacy` | xAI coding data retention opt-in/out | **Hide** or no-op unless next-code has equivalent |
| `/share` | Share session via URL (xAI hosting?) | Verify; likely **hide** until next-code share exists |
| `/marketplace`, `/plugins`, `/hooks` | Grok plugin ecosystem | **Hide** or gate until next-code plugins exist |
| `/import-claude` | Claude settings import modal | Keep if useful; brand-neutral — candidate **bring** |
| `/personas` | Personas tab in agents modal | Candidate **bring** or hide if unused |
| `/voice` | Built-in dictation | Candidate **bring** (vs next-code `/dictate` external); or hide until wired |
| `/recap` | ACP `x.ai/recap` summarize | Remap ACP method / hide until next-code recap |
| `/home` (`/welcome`) | Welcome screen | Keep Face UX (no next-code equivalent slash) |
| `/dashboard` (`/sessions`, `/agents-dashboard`) | Multi-agent dashboard | Keep Face UX; **alias collision**: next-code `/sessions`≡`/resume` |
| `/cd` | Change cwd for new agents | Candidate **bring** |
| `/theme` (`/t`) | Theme picker | **Wire** (next-code lacks slash; Face wins) |
| `/settings` | Settings modal | **Wire** (next-code `/config` text) |
| `/minimal` / `/fullscreen` | Screen mode relaunch | Keep Face UX |
| `/multiline`, `/compact-mode`, `/vim-mode`, `/timestamps`, `/timeline`, `/toggle-mouse-reporting` | Face-local UI toggles | Keep; wire persistence via `set_*` |
| `/always-approve`, `/auto` | Permission modes | Wire to next-code permissions |
| `/copy`, `/find`, `/history`, `/jump`, `/expand`, `/queue`, `/tasks`, `/loop`, `/remember`, `/session-info`, `/terminal-setup`, `/debug`, `/scroll-debug` | Face session/UI tools | Keep UX; wire where daemon needed |
| `/logout` | Log out → login screen | **Wire** next-code logout (not Grok) |
| `/config-agents` | Agent definitions modal | Overlaps `/agents` — wire carefully |

---

## Brand / remaps already planned (PR10)

From `PLAN-20260720-grok-pr10-face-config-settings.md`:

| Item | Action |
|------|--------|
| grok.com billing/docs links | Delete / replace |
| `/gboom`, `/imagine`, `/imagine_video`, announcements→xAI | Hide/restrict in embed |
| `/login` / `/logout` | Wire daemon login — **no** Grok OAuth |
| `/model`, `/theme`, generic slash | Wire to next-code |
| Quit hint `nextcode` | Already done (PR8) |
| Prefer restrict via registry API | Reuse `set_restricted_commands` (same path as tier deny) |

**Suggested embed deny list (starting point):**  
`gboom`, `imagine`, `imagine-video`, `announcements`, `marketplace`, `plugins`, `hooks`, `privacy` (+ optionally `share`, `voice` until wired).  
Still show `/usage` but strip `manage`→grok.com; still show `/docs` but strip/replace `web`→docs.x.ai.

**Alias collision table (must decide in PR10 or immediately after):**

| Token | Face meaning | next-code meaning | Recommendation |
|-------|--------------|-------------------|----------------|
| `/clear` | Alias of `/new` | Clear conversation | Keep Face alias; document |
| `/sessions` | Alias of `/dashboard` | Alias of `/resume` | Prefer Face dashboard; map next-code users via `/resume` |
| `/log` | Alias of `/transcript` | Log mark | Prefer Face; expose next-code mark as `/log-mark` later or drop |
| `/config` | Alias of `/settings` | Show/edit config.toml | Face settings modal + wire; keep `edit` as settings action |
| `/agents` | Alias of `/config-agents` | Role model picker | Face modal; seed with next-code roles |
| `/login` | Grok OAuth | Alias of `/connect` | Remap Face `/login` → next-code connect flow |

---

## Recommended approach for PR10 (slash slice)

Aligned with PR10 Copy / Wire / Delete:

1. **Copy (keep):** Face slash palette, `/model`/`/theme`/`/settings` dropdowns/modals, arg-suggest UX.
2. **Wire:**
   - `set_*` / `load_*` in `xai-grok-shell` → `~/.next-code` config (theme, model, UI toggles).
   - `/model` catalog ← next-code providers (after PR9 if model-affecting).
   - `/login`/`/logout` ← next-code auth (multi-provider). Prefer implementing `/connect` as Face command with `suggest_args` from `provider_catalog`, keep `/login` as alias.
3. **Delete/hide:**
   - Embed `set_restricted_commands([...])` for xAI-only list above.
   - Remove or gate `OpenUrl("https://grok.com/...")` and `docs.x.ai` in embed builds (`nextcode` argv0 or env).
4. **Tests:** restricted commands absent from menu; `/usage manage` does not open grok.com; `/login` does not start Grok OAuth; theme/model persist round-trip.
5. **Out of scope for PR10:** porting the long next-code-only table (swarm, overnight, selfdev, …); TUI crate delete (PR11).

### Files to touch (when implementing — not now)
- `xai-grok-pager` — embed restrict seed near app bootstrap; `usage.rs` / `docs.rs` URL gates
- `xai-grok-shell/src/util/config.rs` — real `set_*`/`load_*`
- Optional: new Face `slash/commands/connect.rs` wrapping provider catalog
- Composition root / `pager_launch` if embed flag needed
- Tests under `slash/commands/mod.rs` + registry tests

## Open questions (≤3)
1. **`/usage`:** keep wired to next-code provider usage, or hide until product billing exists? (PR10 left open.)
2. **`/connect` vs remap `/login` only:** add explicit `/connect` command in Face (matches next-code muscle memory) or only remap Face `/login` + arg dropdown?
3. **Alias collisions** (`/sessions`, `/log`, `/clear`): accept Face meanings in embed, or add next-code compatibility aliases?

## Status
**Implemented (PR10 slash restrict + wire slice)** — 2026-07-21. Full next-code-only port list still deferred.
