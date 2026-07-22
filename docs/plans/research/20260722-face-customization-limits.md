# Research — Face / xai-grok-pager embed limits for “custom TUI everything”

**Date:** 2026-07-22  
**Scope:** Deep research only — no production code.  
**Question:** For next-code’s Face embed (`xai-grok-pager`), what can be customized **without forking Face**, what requires a **Face plugin-host**, and what does full custom TUI (Pi-style) actually cost?  
**Related product plan:** [`docs/plans/PLAN-20260722-pi-full-custom-platform.md`](../PLAN-20260722-pi-full-custom-platform.md)

---

## Verdict (read this first)

| Goal | Feasible without Face fork? | Notes |
|------|----------------------------|--------|
| Brand / slash / welcome / theme / settings | **Yes** | Embed seams already exist |
| Data → fixed Face widgets (floats, reconnect, tools via ACP) | **Yes** | Wire ACP / `next-code/*` notifications |
| Bundle plugins (skills, list/enable, hooks.toml) | **Partial** | Face modal + ACP; **no guest UI draw** |
| Pi `ctx.ui.custom()` / arbitrary Components / Doom-in-Face | **No** | Needs Face plugin-host or full fork |
| Full custom TUI “everything” | **Very high cost** | Rebuild presentation layer; abandon migration north star |

**Recommendation for next-code:** **Defer UI plugins / Face plugin-host.** Keep Face as a **fixed Rust presentation shell**. Pursue Pi-breadth of *workflow* customization via language-agnostic package ABI (manifest + argv/HTTP) as in Option B′ of the Pi platform plan — not in-process guest TUI.

---

## 1. Architecture facts (verified)

### 1.1 Face is the product UI; daemon is the brain

Migration north star (skill + roadmap):

- **Copy Face → delete legacy TUI → wire ACP** — do **not** rewrite Face into next-code-tui style.
- **No `GrokHost` rewrite** unless ACP fails — PR8 chose ACP mediator.
- Prefer stock Face UX; diverge only for branding / daemon.

**Citations:**

- `.agents/skills/grok-migration-workflow/SKILL.md` — hard rules 4–5 (“Copy / wire / remove — do not rewrite Face”)
- `docs/plans/PLAN-20260720-grok-post-pr8-roadmap.md` — PR9–14 wire/delete; GrokHost out of scope
- `docs/plans/PLAN-20260720-grok-pr8-entrypoint.md` — ACP host-as-mediator; Option B GrokHost deferred
- `docs/grok-migration-SUMMARY.md` — “Do not implement SUMMARY §3 GrokHost unless ACP bridge fails”

### 1.2 What “nextcode embed” means in code

Embed is **not** a separate crate feature flag for UI plugins. It is:

1. Binary installs a welcome snapshot once: `pager_launch` → `face_welcome_status` → `install_product_welcome_status`.
2. Face treats “welcome chrome installed” as embed mode: `is_nextcode_embed() == product_welcome_status().is_some()`.

**Citations:**

- `crates/xai-grok-pager/src/product_welcome.rs` — module docs L1–7; `is_nextcode_embed` L111–115; `EMBED_BRAND_RESTRICTED_COMMANDS` L117–135
- `src/cli/face_welcome_status.rs` — gathers auth/model/mcp/skills chrome; calls `install_product_welcome_status`

### 1.3 Two different “embed” words (do not confuse)

| Term | Meaning | File |
|------|---------|------|
| **nextcode embed** | Product chrome + brand-hide when welcome status installed | `product_welcome.rs` |
| **modal embedded** | Minimal-mode borderless modals (`set_embedded`) — layout only | `views/modal_window.rs` L33–49 |

Minimal-mode “embedded” modals are **not** a third-party UI host.

### 1.4 Effects / ACP are the control plane — not a guest renderer

Face owns a large closed `Effect` enum (session create/load, plugins/hooks ACP, settings persist, auth, marketplace, …). Runtime work lives in `app/effects/mod.rs` and talks to the agent via ACP (`x.ai/*` ExtRequest + SessionUpdate).

next-code’s bridge (`src/cli/pager_agent.rs`) feeds Face:

- Typed ACP session updates (tools, plan/todos, text, …) — PR9 theme
- Ext notifications Face already knows how to paint, plus a **small** next-code set:

```text
next-code/token_usage
next-code/provider_name
next-code/memory_info
next-code/git_status
next-code/connection_status
```

**Citations:**

- `crates/xai-grok-pager/src/app/actions.rs` — `pub enum Effect` ~L1344+
- `crates/xai-grok-pager/src/app/effects/mod.rs` — `x.ai/plugins/*`, `x.ai/hooks/*`, `x.ai/skills/*`, …
- `crates/xai-grok-pager/src/app/acp_handler/mod.rs` L613–618 — next-code notification match arms
- `docs/plans/PLAN-20260720-grok-pr9-face-brain-harden.md` — Face ACP handler already capable; wire events, don’t rewrite Face
- `docs/plans/PLAN-20260721-face-info-widget-floats.md` — floats = fixed WidgetKinds + ACP data

**Implication:** Adding a new *kind* of on-screen chrome requires a **Face code change** (new Effect / notification handler / view). Third parties cannot register a paint callback today.

---

## 2. Customization **without forking Face** (ceiling today)

These are the seams next-code (and carefully designed packages) can use while Face stays stock-shaped.

### 2.1 Brand / slash surface

| Mechanism | What it does |
|-----------|--------------|
| `EMBED_BRAND_RESTRICTED_COMMANDS` | Hide + block xAI-only slash (`gboom`, `imagine*`, `marketplace`, `privacy`, `share`, `usage`, …) via `menu_hidden` + unavailable — **not** SuperGrok tier upsell |
| `AppView::apply_tier_restrictions` | Merges brand list when `is_nextcode_embed()` |
| Per-command gates | e.g. `/usage manage`, `/docs web` refuse grok.com URLs in embed |

**Citations:**

- `product_welcome.rs` L122–135, tests L166–186 (`/plugins` + `/hooks` intentionally **not** hidden)
- `app/app_view.rs` L1508–1524
- `docs/plans/PLAN-20260720-grok-pr10-face-config-settings.md` — brand matrix + wire map
- `docs/plans/PLAN-20260721-slash-commands-grok-vs-nextcode.md` — keep Face palette; hide/remap; defer swarm ports

**Limit:** New slash *UX* (arg dropdowns, modals) still means **shipping Face command modules** or ACP-advertised commands that reuse existing Face chrome — not arbitrary author Components.

### 2.2 Welcome / status chrome (data in, Face paints)

`ProductWelcomeStatus` is a fixed schema of chrome lines (badge, server/client animals, model, built, auth dots, Updates, mcp, skills, sessions). next-code fills it at launch; Face welcome paint reads it.

**Citations:**

- `product_welcome.rs` — `ProductWelcomeStatus`, `ChromeLine`, `chrome_lines`
- `docs/plans/PLAN-20260721-face-status-chrome.md` — “Face only paints”; formatters from legacy
- Logo swap already productized via anim (PR8), not Grok braille

**Limit:** Authors cannot invent a fifth chrome widget type without editing Face welcome views.

### 2.3 Theme + `[ui].*` settings

Face settings modal + `/theme` persist through shell `set_*` → `~/.next-code/config.toml` (`[ui].theme`, permission mode, density toggles, …). ThemeKind set is **Face-owned** (GrokNight, TokyoNight, …) — product decision not to remap to legacy dark/light.

**Citations:**

- `PLAN-20260720-grok-pr10-face-config-settings.md` § Implementation notes
- `app/actions.rs` — many `PersistSetting` / `[ui].*` Action comments

**Limit:** Custom theme *files* like Pi’s theme JSON are **not** a third-party Face API today; only picking among built-in ThemeKinds (plus future first-party theme additions in Face).

### 2.4 ACP-fed fixed widgets

| Surface | How customized |
|---------|----------------|
| Tool cards / thinking / plan | ACP `SessionUpdate` from `pager_agent` |
| Info floats (Overview, Memory, Git, Todos, …) | Fixed `WidgetKind`s; data via ACP / `next-code/*` |
| Reconnect banner | `next-code/connection_status` |
| Extensions modal | Face UI; body via `x.ai/plugins|hooks|skills|mcp/*` → `face_plugins` / hooks |

**Citations:**

- `PLAN-20260721-face-info-widget-floats.md` — WidgetKind table; “scroll-only HUD”
- `docs/plugins.md` — Face Extensions + ACP methods
- `PLAN-20260721-plugins-grok-vs-nextcode.md` — copy UI, wire `~/.next-code`, delete TS QuickJS

**Limit:** Floats and modals are a **closed catalog**. Empty/deferred kinds (WorkspaceMap image, Ambient, Tips) wait on first-party Face work — not plugins.

### 2.5 Workflow extension (outside Face paint)

Already (or planned) without Face fork:

| Surface | Mechanism | Lang |
|---------|-----------|------|
| Lifecycle hooks | `next-code-hooks` argv/HTTP | Any executable |
| Skills | Markdown under plugins / skills dirs | Data |
| MCP | `mcp.json` + stdio/HTTP servers | Any |
| Bundle list/enable | `plugin.json` + Face modal | Declarative |

**Citations:**

- `docs/plugins.md`
- `PLAN-20260721-hooks-follow-opencode.md` — keep out-of-process hooks; defer in-process TS
- `PLAN-20260722-pi-full-custom-platform.md` Part C — baseline table

---

## 3. What **requires** a Face plugin-host (or Face fork)

Anything that matches Pi’s **in-process TUI extension** model:

| Pi capability | Pi API (verified in clone) | Face today |
|---------------|----------------------------|------------|
| Full-screen / overlay custom Component + keyboard | `ctx.ui.custom(factory, { overlay?, overlayOptions? })` | **Missing** |
| Replace footer / header / editor | `setFooter` / `setHeader` / `setEditorComponent` | **Missing** (stock Face chrome only) |
| Arbitrary widgets above editor | `setWidget(key, strings \| Component factory)` | **Missing** as guest API (fixed floats ≠ this) |
| Custom tool/message renderers | Extension “Custom Rendering” slots returning Components | **Missing** (ACP maps to Face’s RenderBlock catalog) |
| Games / wizards while waiting | `examples/extensions/snake.ts` etc. | **Missing** |

**Pi citations (tmp clone):**

- `.tmp-research-plugins/pi/packages/coding-agent/docs/extensions.md` — Custom UI § (~L2433+); key capabilities L9–16 (“Full TUI components… via `ctx.ui.custom()`”)
- `.tmp-research-plugins/pi/packages/coding-agent/src/core/extensions/types.ts` — `custom()` L193–208; `setFooter` / `setWidget` / `setEditorComponent`
- `.tmp-research-plugins/pi/packages/coding-agent/docs/tui.md` — copy-paste `ctx.ui.custom` + overlay patterns

**What a Face plugin-host would mean (honest engineering):**

1. A **guest runtime** inside Face (dynlib, WASM, or resurrected JS/TS VM) that can construct ratatui widgets and receive key/mouse focus.
2. Or a **foreign process** that paints into a Face-owned pane via a new protocol (still Face code + focus policy).
3. Stable **ABI + sandbox + lifecycle** across Face upgrades from grok-build.

None of that exists. PR #49 **removed** QuickJS/TS plugin product; roadmap explicitly avoids GrokHost and Face rewrites for migration.

Building this is **not** “wire a stub” — it is a **multi-month product** that fights the migration rule “do not rewrite Face” and the multilang ABI goal (TS-first UI host privileges one language again).

---

## 4. Cost of “full custom TUI everything”

### 4.1 Interpretations

| Interpretation | Cost | Outcome |
|----------------|------|---------|
| **A. Match Pi UI power inside Face** | **Very high** | Face plugin-host + security model + docs + examples; continuous merge tax vs upstream Face |
| **B. Replace Face with next-code-owned TUI** | **Extreme** | Throw away PR1–14 migration; dual UI forever or second full rewrite |
| **C. Keep Face fixed; custom = workflows + data hints + optional external TUI process** | **Medium** (phased) | Pi surface *coverage* without Pi TUI host — Option B′ |

### 4.2 Why A is expensive specifically for next-code

- Face `AppView` / effects / scrollback are a **closed Rust app** (~10k+ LOC app_view class of complexity; SUMMARY evidence). Guest Components need focus, resize, theme, accessibility, and ACP concurrency rules.
- Upstream Face evolves in grok-build; a fork with plugin-host **diverges** and breaks “copy stock Face” discipline (`grok-migration-workflow`).
- Multilang requirement (herdr-style argv) does **not** map cleanly to in-process Component factories (Pi is TS + `@earendil-works/pi-tui`).
- #49 already voted against a privileged guest VM for plugins.

### 4.3 Migration plans already chose “Face fixed”

| Plan | Stance on UI rewrite |
|------|----------------------|
| PR11 | Delete legacy TUI; **no Face rewrite** |
| PR12 | Keep Face API; wire stubs — **do not delete Face UI that calls stubs** |
| PR14 | Delete leftovers; **GrokHost abandoned**; no re-adding legacy TUI |
| Post-PR8 roadmap | Prefer Grok UI; never re-home into TUI |
| Pi platform plan | Explicit non-goal: Pi `ctx.ui.custom`; Phase 4 = hints / external pane only |

---

## 5. Comparison matrix (Pi vs Face embed vs next-code target)

| Capability | Pi | Face nextcode embed today | Sensible next-code target |
|------------|----|---------------------------|---------------------------|
| Custom lifecycle policy | In-process TS `pi.on` | hooks.toml any exe | Package `[[hooks]]` → same runners |
| Custom slash | `registerCommand` + md | Builtins + brand hide + ACP skills | Manifest slash/prompts + existing Face palette |
| Custom tools | `registerTool` TS | MCP + Rust tools | `[[tools]]` argv/MCP |
| Themes | JSON themes + `setTheme` | ThemeKind pick + `[ui].theme` | Optional later token map — not guest Components |
| Status / footer / widgets | `setStatus` / `setFooter` / `setWidget` | Fixed welcome + floats | ACP status hints into **fixed** slots (Phase 4) |
| Arbitrary TUI | `ctx.ui.custom` | **None** | **Defer**; optional OS-terminal pane (herdr-like) |
| Package language | TS (jiti) | Declarative + exe hooks | Manifest + argv (**multilang**) |

---

## 6. Recommendation

### Do now / soon

1. **Treat Face as sealed presentation** — continue PR11–14 discipline: wire ACP, brand, stubs; do not open Face to guest paint.
2. **Invest in workflow ABI** (Option B′): one package manifest declaring hooks/slash/tools/skills/MCP/`[[ui]]` **hints** — runners = argv/HTTP (already proven in `next-code-hooks`).
3. If visible “custom UI” is demanded: ship **data → fixed Face widgets** (status string, toast, tool-card hint fields) — still first-party Face schema, not Components.

### Defer (explicitly)

- Face **plugin-host** / `ctx.ui.custom` equivalent  
- In-process TS/QuickJS UI again  
- Replacing Face with a fully custom next-code TUI for extension reasons  

### Only revisit plugin-host if

Product explicitly bets on an author ecosystem that **must** draw arbitrary TUI *inside* the Face process — and accepts fork/merge cost + sandbox work. That is a separate decision from “full custom like Pi” for **tools/hooks/slash**.

---

## 7. Evidence index

| Claim | Citation | Status |
|-------|----------|--------|
| Embed = welcome status installed | `crates/xai-grok-pager/src/product_welcome.rs` L111–115 | verified |
| Brand-hide list + plugins/hooks visible | same file L122–135, L183–184 | verified |
| Brand applied in AppView | `app/app_view.rs` L1508–1524 | verified |
| Modal “embedded” ≠ product embed | `views/modal_window.rs` L33–49 | verified |
| next-code ACP paint notifications (5) | `acp_handler/mod.rs` L613–617 | verified |
| Effects drive plugins/hooks ACP | `app/effects/mod.rs` (`x.ai/plugins/*`, `x.ai/hooks/*`) | verified |
| PR9: wire events, don’t rewrite Face | `PLAN-20260720-grok-pr9-face-brain-harden.md` | verified |
| PR10 brand + settings wire | `PLAN-20260720-grok-pr10-face-config-settings.md` | verified |
| PR11 delete TUI, no Face rewrite | `PLAN-20260720-grok-pr11-retire-legacy-tui.md` | verified |
| PR14 abandon GrokHost | `PLAN-20260720-grok-pr14-parity-cleanup.md` | verified |
| Plugins = Face modal + ACP; no guest UI | `PLAN-20260721-plugins-grok-vs-nextcode.md`, `docs/plugins.md` | verified |
| Floats = fixed WidgetKinds | `PLAN-20260721-face-info-widget-floats.md` | verified |
| Pi `ctx.ui.custom` API | `.tmp-research-plugins/pi/.../extensions.md`, `types.ts` L193–208 | verified |
| Platform rec: no Face plugin-host in v1 | `PLAN-20260722-pi-full-custom-platform.md` C3/C4/Phase 4 | verified (prior research) |

---

## 8. Open questions (non-blocking)

1. Should Phase 4 “UI hints” be a formal ACP schema (`next-code/status_hint`) before any package ABI ships?
2. Is an **external** plugin pane (spawn user’s TUI binary, herdr-style) desirable, or Face-hints-only forever?
3. Theme authoring: first-party ThemeKind additions in Face vs file-based themes later?

---

## Status

**Research complete.** No production code. Aligns with Option B′: **defer UI plugins; keep Face sealed; customize workflows + data-fed fixed chrome.**
