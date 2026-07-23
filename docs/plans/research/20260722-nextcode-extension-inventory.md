# Research — next-code extension inventory (core vs pluggable vs nextcode defaults)

**Date:** 2026-07-22  
**Scope:** Current `next-code` tree only — inventory, no production code.  
**Vision (user):** Platform as dynamic as Pi; today’s wired product logic becomes the default **nextcode** product bundle; other products can disable/rewrite surfaces without forking the host.

**Sources read:**
- `docs/plans/PLAN-20260722-pi-full-custom-platform.md` (Option B′: Pi surfaces × herdr ABI)
- `docs/plans/PLAN-20260721-plugins-grok-vs-nextcode.md`
- `docs/plans/PLAN-20260721-hooks-follow-opencode.md`
- `docs/plans/PLAN-20260720-grok-pr10-face-config-settings.md`
- `docs/plans/PLAN-20260722-http-mcp-port.md`
- `docs/plugins.md`, `docs/HOOKS.md`, `docs/CONFIG_REFERENCE.md`
- Code: `src/cli/face_plugins.rs`, `src/cli/pager_agent.rs`, `crates/next-code-hooks/`, `crates/next-code-base/src/skill.rs`, `crates/next-code-base/src/mcp/`, `crates/xai-grok-pager/src/product_welcome.rs`, `crates/xai-grok-pager/src/slash/commands/mod.rs`, `crates/xai-grok-pager/src/settings/defs.rs`, `crates/next-code-app-core/src/tool/mod.rs`

**Missing in-tree:** `PLAN-20260722-plugins-pi-opencode-research.md` — noted as absent in the Pi platform plan; content was absorbed into `PLAN-20260722-pi-full-custom-platform.md`.

---

## Summary (read this first)

| Layer | Meaning today | Target under “platform + nextcode product” |
|-------|---------------|---------------------------------------------|
| **Core / hardwired** | Face Rust UI, ACP transport, agent turn loop, tool *dispatch*, permission engine, builtin tool *implementations* | Stay host-owned; expose seams, don’t guest-host UI kits |
| **Already pluggable** | `hooks.toml` argv/HTTP, `mcp.json`, skill dirs, provider profiles, prompt overlays, notepad files | Become first-class **package ABI** entrypoints (Phase 2+) |
| **Product “nextcode” defaults** | Brand-hidden slash set, Face Extensions chrome, welcome/status, settings catalog defaults, Grok-layout bundles under `~/.next-code` | Ship as **default nextcode bundle** — disable/replace when rebranding |

**Honest gaps (verified):**
1. Bundle `hooks/hooks.json` and `.mcp.json` are **listed/counted** in Face Plugins UI — **not** merged into `next-code-hooks` / MCP runtime.
2. `plugins-state.json` enable/disable affects Face list — **skill ingest does not filter** by that state (`SkillRegistry::load_global` loads all `plugins/*/skills`).
3. Slash builtins are **compiled into Face**; ACP only advertises **skills** as dynamic commands (not arbitrary package slash).
4. Marketplace ACP is stub/hidden; no unified `next-code-plugin.toml` ABI yet.

---

## Classification legend

| Tag | Definition |
|-----|------------|
| **CORE** | Must remain in host (security, process, ACP, Face render) |
| **PLUG** | Already user/config extensible without core PR |
| **GAP** | Declared or UI-visible but not fully runtime-wired |
| **PROD** | nextcode-product opinion (brand, defaults, chrome) — should become a disableable bundle |

---

## Master table

| surface | today | move-to-platform | keep-as-default-nextcode-bundle | notes |
|---------|-------|------------------|--------------------------------|-------|
| **Face UI shell (Grok pager)** | CORE — Rust TUI; single product embed | Keep host; no Pi `ctx.ui.custom` in v1 | nextcode chrome (welcome, status, quit hint) | `xai-grok-pager`; PR11 retires legacy TUI |
| **ACP transport + session** | CORE — `pager_agent` ↔ Face | Keep; extend method family | nextcode method set (`x.ai/*` remapped) | Skills/MCP/plugins/hooks/auth/session list wired |
| **Slash: Face builtins** | CORE+PROD — `builtin_commands()` ~60 cmds hardcoded | Package `[[slash]]` + ACP advertise for *extensions*; builtins stay or move to product pack | Entire nextcode slash palette + brand-hide list | `slash/commands/mod.rs` |
| **Slash: brand-hidden** | PROD — `EMBED_BRAND_RESTRICTED_COMMANDS` | Platform: product profile declares hide/show | nextcode hides: `gboom`, `imagine`, `imagine-video`, `announcements`, `marketplace`, `privacy`, `share`, `usage` | `/plugins` + `/hooks` **visible** |
| **Slash: ACP skills** | PLUG — skills → `AvailableCommandsUpdate` → Face `/name` InjectSkill | Same; also package prompts → slash | Default skill pack (if any) | `pager_agent::emit_available_skills` |
| **Slash: markdown prompts** | PLUG (data) / GAP (Face) — `~/.next-code/prompts/*.md` + `prompt_templates.rs`; CONFIG docs say TUI `/name` | Platform: prompt slash from packages | nextcode may ship starter prompts | Face registry not importing templates today (legacy/CLI path) |
| **Hooks runtime (`next-code-hooks`)** | PLUG — v2 multilang (command/HTTP/plugin exe) | **Primary** platform runner (herdr-aligned) | Optional nextcode policy hooks examples | ~28 events; layers below |
| **Hooks config layers** | PLUG — user → project → `$NEXT_CODE_HOOKS_CONFIG`; kill `DISABLE_NEXT_CODE_HOOKS` | Package `[[hooks]]` compile into same registry | Product-default hooks.toml (empty or soft policies) | Append handlers; settings override |
| **Hooks v1 `[hooks]` in config.toml** | PLUG (legacy merge) | Deprecate into v2 / package | Keep compat in nextcode | `legacy_v1_to_v2_handlers` |
| **Face `/hooks` + ACP** | PLUG+UI — list/enable/disable/reload/add/remove wired | Keep UI; semantics = next-code-hooks | Extensions modal as nextcode UX | Not OpenCode JS hooks |
| **Bundle plugins disk** | PLUG (discover) — `~/.next-code/plugins`, project `.next-code/plugins`, `installed-plugins/`, Claude compat | Unified package install/link | nextcode ships zero or demo bundles | `face_plugins::discover_plugins` |
| **`plugin.json` / convention dirs** | PLUG (manifest recognition) | Evolve → `next-code-plugin.toml` (open Q) | Grok-compat layouts as nextcode default schema | Also `.grok-plugin` / `.claude-plugin` |
| **`plugins-state.json` enable/disable** | GAP — Face list only | Platform: gate skills/MCP/hooks by enable | nextcode UI defaults “all enabled” | Skills load **ignores** disabled today |
| **Bundle `skills/`** | PLUG — ingested into `SkillRegistry` | Keep; respect enable state | Product skill pack | Global + project overlays |
| **Standalone skills dirs** | PLUG — `~/.next-code/skills`, `~/.agents/skills`, project `.next-code/.agents/.claude/skills` | Keep | Import-on-first-run from Claude/Codex = nextcode DX | `SkillRegistry::import_from_external` |
| **Face `/skills` + `$skill`** | PLUG+UI | Keep | — | ACP `x.ai/skills/*` |
| **Bundle `agents/`** | GAP — named/counted in Plugins list | Platform agent defs or drop | If kept, nextcode agent recipes | Runtime agent load from bundles **not** verified as first-class |
| **Bundle `hooks/hooks.json`** | GAP — `has_hooks` UI only | Package → merge into hooks registry | — | Deferred in hooks plan Phase 3 |
| **Bundle `.mcp.json`** | GAP — count only in Plugins UI | Package → merge into MCP load | — | Runtime MCP ignores plugin-local `.mcp.json` |
| **MCP `mcp.json` layers** | PLUG — user + project + Claude/Codex import; stdio + HTTP/SSE | Keep; packages may contribute servers | nextcode may ship example mcp.json | Trust gate optional (`--require-mcp-trust`) |
| **Face `/mcp` + ACP list** | PLUG+UI | Keep | — | HTTP port landed 2026-07-22 |
| **Builtin tools (Rust)** | CORE — registry in `tool/mod.rs` (bash, read/write/edit, ffs_*, web*, swarm, memory, …) | Allowlist disable via config; **new** tools via MCP/`[[tools]]` argv later | nextcode default toolset + `[tools] disabled` | No guest `registerTool` in-process |
| **MCP tools** | PLUG — any-lang servers | First-class platform tool path | Optional curated MCP set | Already language-agnostic |
| **Providers / models** | PLUG (config) + CORE (Rust clients) — `config.toml` `[providers.*]`, OAuth files | Gateway/HTTP profiles only; no guest `registerProvider` | nextcode default provider prefs | Face `/model` via ACP History |
| **System prompt** | CORE default `system_prompt.md` + PLUG overlays | Overlays + package prompt fragments | nextcode default prompt = product | `SYSTEM.md` / `APPEND_SYSTEM.md` / AGENTS.md / prompt-overlay.md |
| **Notepad** | PLUG — `.next-code/notepad/` tiers | Keep as core feature with config | Default enabled in nextcode | Not a package surface |
| **Settings modal** | CORE UI + PROD catalog — `settings/defs.rs`; persist `[ui].*` / provider to `~/.next-code/config.toml` | Product profile can hide settings groups | Full nextcode settings catalog | PR10 wired shell `set_*` |
| **Theme / keybinds** | PLUG-ish — `[ui].theme` etc. | Later: theme token files from packages | nextcode theme defaults | Face ThemeKind; not Pi themes JSON |
| **Marketplace** | PROD+stub — brand-hidden; `x.ai/marketplace/list` stub | Optional platform registry later | Hidden in nextcode | Do not advertise until real |
| **Permissions / trust** | CORE — DCG / permission rules / managed config | Project trust for executable plugins (Phase 1) | nextcode permission defaults | Executable hooks = RCE risk |
| **Sessions store** | CORE — `~/.next-code/sessions` | Keep | — | Face `/resume` via `x.ai/session/list` |
| **Auth / login Face flows** | CORE+PROD — nextcode provider connect (not Grok OAuth) | Keep host | nextcode auth UX | `/login` `/connect` remapped |
| **Legacy TUI** | CORE (retiring) | Delete per PR11 | — | Don’t extend |
| **QuickJS / TS plugins** | Removed (#49) | Do **not** resurrect as platform | — | Multilang = argv/HTTP |
| **Package ABI (manifest + argv)** | Missing | **Build** (Pi surfaces × herdr runners) | nextcode = first consumer package | See platform plan Option B′ |
| **Custom Face panes / widgets** | Missing (fixed Face) | Phase 4: status/toast hints + optional external pane argv | nextcode chrome widgets only | No Doom-in-Face |

---

## Surface deep-dives (evidence)

### 1. Face slash + ACP

**Builtins (hardwired):** `crates/xai-grok-pager/src/slash/commands/mod.rs` → `builtin_commands()` registers Face-local commands (exit, help, plugins, hooks, skills, mcps, settings, model, theme, login/connect, … including xAI-oriented ones still compiled in).

**Brand hide (product):** `product_welcome.rs` `EMBED_BRAND_RESTRICTED_COMMANDS` — applied when `is_nextcode_embed()`. Marketplace stays hidden; plugins/hooks intentionally **not** hidden after PR49 wiring.

**Dynamic slash today:** only skills (and skill-shaped ACP commands). `pager_agent` loads `SkillRegistry` into `InitializeResponse` meta and `AvailableCommandsUpdate`. Face `sync_acp_commands` merges into the slash registry. There is **no** general “package registered `/review` command” path yet.

**Implication for platform:** move *extension* slash to ACP advertise + package prompts; keep Face builtins as **nextcode product pack** (or thin core + product overlay).

### 2. Hooks.toml layers (already platform-shaped)

| Layer | Path |
|-------|------|
| User | `~/.next-code/hooks.toml` |
| Project | `<cwd>/.next-code/hooks.toml` |
| Env override | `$NEXT_CODE_HOOKS_CONFIG` |
| Kill | `DISABLE_NEXT_CODE_HOOKS` |
| Legacy | `config.toml` `[hooks]` + `NEXT_CODE_HOOK_*` env |

**Handlers:** `command` | `http` | `agent` (stub) | `plugin` (external exe) — stdin JSON, exit 0/1/2.  
**Events:** 29 standard `HookEvent`s (`config.rs` `all_standard`); several declared but **not yet dispatched** (Subagent*, Task*, AutoCompactionControl) — see `docs/HOOKS.md`.

**Face:** `x.ai/hooks/list|action` → real registry (reload / enable / disable / merge add / remove). Trust/untrust Unsupported.

This is the strongest existing **herdr-like** multilang seam. Package ABI should compile *into* this registry, not fork a second one.

### 3. `~/.next-code/plugins` bundles

**Discovery** (`face_plugins.rs`): user plugins, install registry, project plugins, `~/.claude/plugins` (list/skill ingest; uninstall blocked).

**Recognized if:** `plugin.json` (or grok/claude nested), and/or `skills/`, `agents/`, `hooks/hooks.json`, `.mcp.json`.

**Actually runtime-effective today:**
- Skills from plugin trees → yes (but **no** enable/disable gate)
- Agents → UI metadata
- Bundle hooks / MCP → UI counts only

**Install:** local path + git via Face action; state in `plugins-state.json` + `installed-plugins/registry.json`.

### 4. Skills

**Load order (global):** Claude plugins → `~/.next-code/plugins` + `installed-plugins` → `~/.next-code/skills` → `~/.agents/skills`. Project overlay last.  
**Invocation:** `$skill` / Face InjectSkill → system_reminder expansion in `pager_agent`.

**Product default:** first-run copy from `~/.claude/skills` / `~/.codex/skills` into `~/.next-code/skills` — nextcode DX, not core necessity.

### 5. MCP

**Config:** `~/.next-code/mcp.json` + project `.next-code/mcp.json` + `.mcp.json` + `.claude/mcp.json` (+ Codex import).  
**Transports:** stdio + streamable HTTP (recent).  
**Face:** `/mcp` + `x.ai/mcp/list`.  
**Not:** auto-load of plugin-dir `.mcp.json`.

### 6. Tools

**Hardwired set** built in `ToolRegistry` (read/write/edit modes, bash, browser, ffs_*, websearch/webfetch, swarm, memory, gmail, schedule, …). Disable via session/`[tools]` policy — not replace via packages.  
**Extension path today:** MCP only.  
**Platform target:** Phase 3 `[[tools]]` argv runners (language-agnostic) + MCP.

### 7. Settings + config

**Canonical home:** `~/.next-code/` (`$NEXT_CODE_HOME` / XDG mode).  
**Main:** `config.toml` — providers, features, `[ui]` (Face settings persist), `[hooks]` legacy, `[tools]`, terminal prefs.  
**Settings modal:** large Face catalog (`settings/defs.rs`); SHELL-owned keys write next-code toml (PR10). Many Grok-era labels remain — product polish vs platform.

### 8. Prompt / identity surfaces

| Mechanism | Role |
|-----------|------|
| Built-in `system_prompt.md` | CORE default identity |
| `SYSTEM.md` / `APPEND_SYSTEM.md` | PLUG replace/append |
| `AGENTS.md` (home + project) | PLUG |
| `prompt-overlay.md` | PLUG |
| `prompts/*.md` templates | PLUG data; Face slash wiring **GAP** |
| Notepad priority tier | PLUG session memory |

---

## Mapping to Pi surfaces (from platform plan)

| Pi surface | next-code today | Platform move | nextcode default bundle |
|------------|-----------------|---------------|-------------------------|
| TUI custom | Face fixed | Hints / external pane later | Face chrome |
| Lifecycle hooks | hooks.toml | Package `[[hooks]]` | Optional policies |
| Slash / prompts | builtins + skills + prompts GAP | Manifest slash + prompts | Builtin slash pack + brand hide |
| Tools | Rust + MCP | MCP + argv tools | Default toolset |
| Skills / packages | dirs + Grok bundles | Unified package ABI | Demo/empty + skill import DX |
| Providers | config.toml | Same (no guest register) | Default provider UX |
| Config / trust | config + permissions | Trust gate for exe plugins | nextcode permission defaults |
| Install/update | git/local Face | `plugin link` / git | Marketplace hidden until real |

---

## Recommended split (product architecture)

```text
┌─────────────────────────────────────────────────────────┐
│ HOST (immutable core)                                   │
│  Face render · ACP · agent loop · permissions ·         │
│  hook dispatcher · MCP client · skill ingest engine     │
└───────────────────────────┬─────────────────────────────┘
                            │ loads
┌───────────────────────────▼─────────────────────────────┐
│ PLATFORM ABI (declarative + argv/HTTP runners)          │
│  next-code-plugin.toml → hooks / slash / skills / mcp   │
│  / tools / ui hints                                     │
└───────────────────────────┬─────────────────────────────┘
                            │ default install
┌───────────────────────────▼─────────────────────────────┐
│ PRODUCT BUNDLE: "nextcode"                              │
│  brand-hidden slash · welcome/status · settings catalog │
│  default tool enablement · optional starter skills/MCP  │
│  Face Extensions chrome labels · auth connect flows     │
│  Disable → another product can ship its own bundle      │
└─────────────────────────────────────────────────────────┘
```

**Rule of thumb:** if a fork would want different *opinion* without changing security/isolation, it belongs in the **nextcode bundle**. If every product needs it for the agent to work, it stays **host**. If authors must add behavior without a core PR, it must be **ABI**.

---

## Priority gaps (platform Phase 0–2 aligned)

1. **Enable-state gating** for plugin skills (and later MCP/hooks) — UI already has state; runtime doesn’t.
2. **Merge bundle hooks/MCP** into real registries (or stop advertising Active/Blocked as if live).
3. **Package manifest** declaring hooks/slash/skills/mcp with argv/HTTP runners (reuse hooks execute path).
4. **Product profile** table: which slash/settings/welcome pieces are nextcode-only.
5. **Prompt templates → Face slash** (or ACP advertise) so docs match product.
6. **Trust** for project executable hooks/plugins before marketing “any language extensions.”

---

## Open questions (unchanged from platform plan)

1. Manifest name: `next-code-plugin.toml` vs extend `plugin.json`?
2. `[[tools]]` argv in Phase 3 vs MCP-only until then?
3. External plugin pane (spawn TUI) in Phase 4 vs Face-hints-only?

---

## Status

**Research only.** No production code.  
Next: use this matrix to ticketize Phase 0 (capability contract) and Phase 1 (trust + make bundle counts real) from `PLAN-20260722-pi-full-custom-platform.md`.
