# BUG — Face embed still brands Grok + stacks redundant auth chrome

Branch: `pr-face-embed-branding`  
Related: `BUG-20260723-face-model-persist-mismatch.md` (same `default_provider` display-name root for model line)  
Status: **Implemented**

## Summary (read this first)

- **Symptom A (branding):** Windows Terminal tab titled `grok`; welcome shows `Grok Build Beta 0.1.0` and `Thanks for trying Grok Build…` even when launching `nextcode` / `next-code`.
- **Symptom B (auth chrome):** Already authenticated via API keys, but welcome stacks three overlapping signals — green provider dots (`● openrouter`), model line (`api-key: opencode go · … · /model to switch`), and Face footer (`Logged in with API key | Beta`).
- **Product rule (skill):** Face chrome must brand **nextcode / next-code** when embedded; stock Grok strings only for stock `grok` bin.
- **Verified roots:** (1) Face `TitleManager` hardcodes `"grok"` OSC title; (2) welcome badge / hero subtitle hardcode `"Grok Build"` and are **not** gated by `is_nextcode_embed()`; (3) version comes from `xai-grok-version` crate `0.1.0`, not next-code; (4) auth chrome is three intentional but uncoordinated layers; (5) model-line provider text reuses bad display name `"opencode go"` from the persist bug.

## Ranked hypotheses

| # | Hypothesis | Verdict |
|---|------------|---------|
| 1 | WT tab says `grok` because Face `TitleManager` / `TitleItem::Grok` writes OSC title `"grok"` and overwrites next-code `nc:client` | **Verified** |
| 2 | Welcome header / subtitle still say Grok because `render_version_badge` + `HERO_SUBTITLE` are hardcoded and ignore embed | **Verified** |
| 3 | Embed launches wrong binary / `grok` alias on PATH | **Ruled out as primary** — `dispatch` → `pager_launch::run_face_pager` in-process; fake clap argv `["grok"]` is only for `PagerArgs` parse, not process image |
| 4 | `Logged in with API key` means xAI subscription / Grok account | **Ruled out** — set because NextCode advertises ACP method id `xai.api_key` (`XAI_API_KEY_METHOD_ID`) |
| 5 | Green `openrouter` + `api-key: opencode go` means two active routes | **Partially** — dots = credential inventory; model line = config default route; both can be true when OpenRouter key exists and `default_provider` is OpenCode Go |
| 6 | Same root as model-persist bug for `opencode go` on the model line | **Verified** (display name in config → painted as-is) |

## Evidence

### 1) Terminal tab title = Face OSC `"grok"`

next-code sets a sane process title first:

- `src/cli/dispatch.rs` / `startup.rs` → `proctitle::set_initial_title`
- Interactive client default: `"nc:client"` — `src/cli/proctitle.rs` (~62–74)

Face then owns the tab title via notifications:

- Default items include `TitleItem::Grok` — `crates/xai-grok-pager/src/notifications/config.rs` (~74–84)
- `TitleItem::Grok` pushes literal `"grok"` — `notifications/title.rs` (~135–138)
- Empty compose fallback + `reset()` also write `"grok"` — same file (~90–93, ~115–118)

DeepWiki (stock grok-build): title items include `grok`; no embed rebrand hook. Matches our vendored Face.

**A/B note:** Official `grok` bin correctly wants tab `grok`. nextcode overwrite is the bug.

### 2) Welcome product name / thanks line still Grok

| Surface | Hardcoded string | Gate on embed? | Path |
|---------|------------------|----------------|------|
| Hero inline badge | `"Grok Build Beta  "` + `xai_grok_version::VERSION` | **No** | `views/welcome/mod.rs` `render_version_badge` HeroInline (~590–600) |
| Full badge | `"Grok Build  "` + version + `" Beta"` | **No** (only appends embed `build_age`) | same (~552–577) |
| Hero subtitle | `"Thanks for trying Grok Build, give feedback with /feedback!"` | **No** | `views/welcome/hero_box.rs` `HERO_SUBTITLE` (~31, used ~337) |
| Logo | next-code donut (already rebranded) | N/A | `views/welcome/logo.rs` |

Embed detection exists and is used elsewhere (slash hide list, changelog, etc.):

- `product_welcome::is_nextcode_embed()` ⇔ `install_product_welcome_status` was called — `product_welcome.rs` (~101–115)
- Installed from `pager_launch` → `face_welcome_status::install_face_welcome_status` — `src/cli/pager_launch.rs` (~31–34)

**Welcome badge / subtitle never call `is_nextcode_embed()`.**

### 3) Version `0.1.0` is Face crate, not next-code

- Badge paints `xai_grok_version::VERSION` — defaults to `CARGO_PKG_VERSION` = **`0.1.0`** (`crates/xai-grok-version/Cargo.toml` + `src/lib.rs`)
- next-code already has the real product version in chrome animal lines via `next_code_build_meta::VERSION` — `face_welcome_status.rs` (~257–275)
- So header can show **Grok Build Beta 0.1.0** while client animal line shows the real next-code version — inconsistent.

### 4) Fake clap argv `["grok"]` (related smell, not WT title)

```37:38:src/cli/pager_launch.rs
    let mut pager_args = xai_grok_pager::app::PagerArgs::try_parse_from(["grok"])
        .map_err(|e| anyhow::anyhow!("failed to build Face pager args: {e}"))?;
```

- Purpose: synthesize `PagerArgs` without re-parsing the real CLI.
- Does **not** change `std::env::args_os()`; `resume_cli_name()` still prefers real stem / `XAI_PAGER_RESUME_CLI` — `app/mod.rs` (~784–802).
- Still a branding smell: any future code that trusts `PagerArgs` binary name will say `grok`. Prefer stem of real argv0 (`nextcode` / `next-code`) or set `XAI_PAGER_RESUME_CLI` at launch.

### 5) Triple auth chrome — what each piece means

Operator sees (example matching screenshots + persist bug config):

1. **Green dots** `● openrouter` (+ others)  
   - Source: `face_welcome_status::build_auth_dot_entries` — inventory of **configured** credentials (`AuthStatus::check_fast`), not the active turn route.  
   - OpenRouter appears whenever an OpenRouter key is configured, even if active model is OpenCode Go.

2. **Model line** `api-key: opencode go · deepseek-v4-flash · /model to switch`  
   - Source: `Config.provider.default_provider` + `default_model` → `header_provider_label` → `format_model_switch_parts`.  
   - `header_provider_auth_tag("opencode go")` returns `"api-key"` via openai-compatible display-name helper (`face_welcome_status.rs` ~172–179).  
   - Label keeps lowercase **display** string `"opencode go"` (space), same bad pin as `BUG-20260723-face-model-persist-mismatch.md`.

3. **Footer** `Logged in with API key | Beta`  
   - Source: Face `render_version_badge` when `is_api_key_auth` (`welcome/mod.rs` ~542–547).  
   - Set at startup because NextCodeFaceAgent advertises ACP method `xai.api_key` (`pager_agent.rs` ~1476–1478) and Face treats that id as API-key auth when no AuthMeta (`event_loop.rs` ~960–967; `XAI_API_KEY_METHOD_ID` in `xai-grok-shell`).  
   - **Meaning:** “Face session authenticated via API-key-shaped ACP method,” **not** “OpenRouter is the active provider” and **not** “Grok subscription.”  
   - `Beta` is `xai_grok_update::channel_label()` / HeroFooter channel display — Grok release channel, not next-code channel.

So the chrome is **technically consistent with three different concepts** (inventory / active config route / Face auth mode) but **product-confusing** when stacked.

### 6) Relation to provider persist bug

Same config field:

```toml
[provider]
default_provider = "opencode go"   # display name, not catalog id opencode-go
default_model = "deepseek-v4-flash"
```

- Persist bug: daemon routing fails / chrome lies about model.  
- This bug: welcome **paints** that display name into `api-key: opencode go`, amplifying the mismatch next to green `openrouter` inventory.  
- Fixing persist (catalog id) cleans the model-line label; auth-chrome cleanup is still needed so inventory ≠ footer ≠ route do not all shout “API key.”

## What was ruled out

| Claim | Why ruled out |
|-------|----------------|
| User launched `grok.exe` by mistake | In-process Face via `run_face_pager`; logo already next-code |
| Need dual Leave / quit shim for title | Title is OSC write every tick, unrelated to restore |
| “Logged in with API key” requires OpenRouter | Driven by ACP method id `xai.api_key`, independent of which provider key is active |
| product_welcome missing entirely | Installed; used for animal/mcp/skills/updates — **not** for product name / tab title |

## Copy / wire / delete map (for BUILD after OK)

Prefer **wire at embed seam** + **minimal Face brand hooks** (skill: do not rewrite Face brain).

| Kind | Change | Where |
|------|--------|--------|
| **Wire** | Pass real CLI stem into Face: `try_parse_from([resume_cli_name()])` or set `XAI_PAGER_RESUME_CLI` before `app::run` | `src/cli/pager_launch.rs` |
| **Wire** | Install embed brand snapshot: product display name (`nextcode` / `Next Code`), version (`next_code_build_meta::VERSION`), optional subtitle | Extend `ProductWelcomeStatus` + `install_face_welcome_status` |
| **Wire** | When `is_nextcode_embed()`, `render_version_badge` / `HERO_SUBTITLE` use product fields (fallback stock Grok) | `views/welcome/mod.rs`, `hero_box.rs` |
| **Wire** | When embed: `TitleItem::Grok` / fallback / `reset()` paint `"nextcode"` (or argv0 stem), not `"grok"` | `notifications/title.rs` (+ optional rename later) |
| **Wire** | Auth chrome cleanup (pick one product policy — recommend **A**): | see below |
| **Delete / avoid** | Do not keep dual strings “for compatibility” in embed path; no second welcome stack | — |
| **Do not copy** | Stock grok-build has no embed brand API (DeepWiki) — we add thin hooks only | — |

### Auth chrome cleanup recommendation (policy A — preferred)

**One primary auth story on welcome:**

1. Keep **model line** as the active route (`api-key:opencode-go · model · /model`) — after persist fix uses catalog id.  
2. Keep **provider dots** as secondary inventory **or** demote: only show when >1 configured provider / compact mode; consider marking active provider.  
3. In embed: **suppress** Face footer `"Logged in with API key"` (and Grok `Beta` channel) when `product_welcome` is installed — replace with next-code product badge (`nextcode` + real version) only.  
4. Do **not** invent a fourth auth line.

Alternate **B** (heavier): teach Face AuthMeta from NextCodeFaceAgent with `auth_mode` reflecting active provider; still suppress Grok Beta channel in embed.

## Files to touch (expected)

- `src/cli/pager_launch.rs` — argv0 / resume CLI env  
- `src/cli/face_welcome_status.rs` — brand + version fields; maybe auth inventory policy  
- `crates/xai-grok-pager/src/product_welcome.rs` — brand fields + helpers  
- `crates/xai-grok-pager/src/views/welcome/mod.rs` — badge strings  
- `crates/xai-grok-pager/src/views/welcome/hero_box.rs` — subtitle  
- `crates/xai-grok-pager/src/notifications/title.rs` (+ tests) — tab title  
- Tests: welcome badge embed; title embed fallback; face_welcome_status provider label after persist fix  
- Coordinate with `BUG-20260723-face-model-persist-mismatch.md` (catalog id) so model line stops saying `opencode go`

## Open questions (resolved in BUILD)

1. **Product string:** hero badge + tab title use short `nextcode` (not `Next Code` / `next-code`).
2. **Auth inventory:** keep green dots always (policy A secondary inventory).
3. **Ship with persist fix or separate?** Separate branch `pr-face-embed-branding` from `origin/dev`. Model-line chrome normalizes display name → catalog id here (`opencode go` → `opencode-go`) so branding PR is useful alone; persist fix still owns daemon routing.

## Risk

- Medium for title/badge (user-visible, low logic risk).  
- Auth-chrome policy change can surprise users who relied on inventory dots — document in release notes.  
- TitleManager tests hardcode `"grok"` — need embed-aware cases, not blanket rename that breaks stock grok-bin expectations inside the same crate.

## Status

**Implemented** on `pr-face-embed-branding`.

### BUILD notes

| Surface | Stock (`grok`) | Embed (`nextcode` / `next-code`) |
|---------|----------------|----------------------------------|
| WT/OSC title | `grok` | `nextcode` |
| Hero inline / full badge | `Grok Build Beta` + Face `0.1.0` | `nextcode` + `next_code_build_meta::VERSION` |
| Hero subtitle | Thanks for trying Grok Build… | Thanks for trying nextcode… |
| Footer `Logged in with API key` / Grok `Beta` | unchanged | suppressed |
| Model-line provider | (unchanged stock path) | catalog id preferred (`api-key:opencode-go`) |
| `PagerArgs` / resume CLI | N/A | real argv0 stem + `XAI_PAGER_RESUME_CLI` |

Tests: `cargo test -p xai-grok-pager --lib title::tests brand_` · `cargo test --lib face_welcome_status::`
