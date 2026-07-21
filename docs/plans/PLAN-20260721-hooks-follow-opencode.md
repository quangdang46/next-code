# Plan Report — next-code hooks following OpenCode

## Summary (read this first)
- **You asked:** Clone OpenCode, review its **hooks** system, compare to next-code, recommend how next-code hooks should follow OpenCode (plan first, no full rewrite).
- **What is going on:** OpenCode does **not** have a Claude/Grok-style `hooks.toml` / `hooks.json` command runner. Its “hooks” are the **return object of JS/TS plugins** (`Hooks` interface): in-process typed callbacks with mutable `output`, plus a catch-all `event` bus. next-code already has a **different, stronger** product for user automation: `crates/next-code-hooks` (v2 `hooks.toml` + v1 `[hooks]` in `config.toml`) that runs shell/HTTP/plugin **executables** at lifecycle points. Face `/hooks` is Grok Extensions chrome; ACP `x.ai/hooks/list` is currently an **empty stub**.
- **We recommend:** **Do not replace** `next-code-hooks` with OpenCode’s TS-only model. **Copy OpenCode patterns** into (a) how we name/document lifecycle seams and (b) optional in-process plugin callbacks later. **Wire Face `/hooks`** to `~/.next-code/hooks.toml` (and project `.next-code/hooks.toml`) — keep Face UI, use next-code semantics. Treat PR49 Grok-plugins work as a **sibling tab** in the same Extensions modal, not the same runtime.
- **Risk:** Medium if we conflate “OpenCode hooks” with “Face /hooks” and try a rewrite; Low if we wire list/action first and keep runtimes separate.
- **Status:** Phase 0–1 **done** on `pr-face-hooks-wire` — Face `/hooks` lists/enable/disable real `next-code-hooks` config. Later phases still open.

---

## Clone / research sources

| Item | Path |
|------|------|
| OpenCode shallow clone | `C:\Users\ADMIN\Documents\Projects\tmp\opencode` (`git clone --depth 1`) |
| Uploaded doc | `uploads/opencode-0.md` — GitHub landing page only; **not useful** for hooks design (no API detail) |
| OpenCode Hooks type | `packages/plugin/src/index.ts` → `export interface Hooks` |
| OpenCode trigger runtime | `packages/opencode/src/plugin/index.ts` → `Plugin.trigger` sequential fan-out |
| OpenCode docs | `packages/web/src/content/docs/plugins.mdx` |
| next-code hooks crate | `crates/next-code-hooks/` |
| next-code dispatch site | `crates/next-code-app-core/src/tool/mod.rs` (`load_hooks_config` + `dispatch_hooks`) |
| Docs | `docs/HOOKS.md` (v1 legacy); crate docs for v2 |
| Face `/hooks` | `xai-grok-pager` Extensions modal + `src/cli/face_plugins.rs` stubs |
| Parallel plugins plan | `docs/plans/PLAN-20260721-plugins-grok-vs-nextcode.md` |

---

## What OpenCode “hooks” actually are (verified)

### Model
A **plugin** is `async (PluginInput, options?) => Hooks`. Hooks are **in-process functions**, not external scripts.

Canonical type: `packages/plugin/src/index.ts` (`Hooks`):

| Kind | Examples | Behavior |
|------|----------|----------|
| Lifecycle / mutate | `tool.execute.before`, `tool.execute.after`, `chat.params`, `chat.headers`, `permission.ask`, `shell.env`, `command.execute.before`, `tool.definition`, compaction hooks | `(input, output) => Promise<void>` — mutate `output`; sequential across plugins |
| Event bus | `event: ({ event }) => …` | Observes bus events (`session.idle`, `session.created`, `file.edited`, …) |
| Register | `tool: { mytool: tool({…}) }`, `auth`, `provider`, `config`, `dispose` | Side registration, not “run a shell” |

`Plugin.trigger` (`packages/opencode/src/plugin/index.ts` ~280–292): for each loaded hooks object, call `hook[name](input, output)` in order; return the (mutated) `output`. Deterministic order; no parallel semaphore.

Deny pattern (docs example): `tool.execute.before` **throws** to block reading `.env`.

### Layout / config
From `packages/web/src/content/docs/plugins.mdx`:

| Source | Path |
|--------|------|
| Project plugins | `.opencode/plugins/*.js|ts` |
| Global plugins | `~/.config/opencode/plugins/` |
| npm plugins | `opencode.json` → `"plugin": ["pkg", …]` (Bun install → `~/.cache/opencode/node_modules/`) |

Load order (docs): global config → project config → global plugin dir → project plugin dir. All hooks from all plugins run in sequence.

There is **no** separate OpenCode `hooks.toml` / Claude `settings.json` hooks table in this tree — extension authors write TypeScript.

### Event names (docs + type)
Bus / docs list includes: `session.created|idle|updated|deleted|error|diff|compacted|status`, `tool.execute.before|after`, `permission.asked|replied`, `file.edited`, `message.*`, TUI events, etc. Trigger hooks use dotted names; bus uses `event.type` strings.

---

## What next-code has today (verified)

### A. Lifecycle hooks runtime — `next-code-hooks` (primary “hooks product”)
- **Config layers:** `~/.next-code/hooks.toml` → `<cwd>/.next-code/hooks.toml` → `$NEXT_CODE_HOOKS_CONFIG`
- **Events:** 28 PascalCase + `Custom` (`PreToolUse`, `SessionStart`, `Stop`, …) — Claude/Grok-shaped, not OpenCode dotted
- **Handlers:** Command | HTTP | Agent (stub) | Plugin (external exe) — stdin JSON / exit 0|1|2
- **Dispatch:** parallel with concurrency cap; deny > ask > allow for blocking events
- **Wire-in:** `ToolRegistry::new` merges v1 `config.toml [hooks]` via `legacy_v1_to_v2_handlers` so `pre_tool` / `session_start` still work (`docs/HOOKS.md`)
- **CLI:** `next-code hooks list|enable|disable|test|metrics` (`crates/next-code-hooks/src/cli.rs`)

This is closer to **Grok / Claude / Cursor** shell hooks than to OpenCode plugins.

### B. Face `/hooks` (UI only + stub ACP)
- Slash `/hooks` opens Extensions modal Hooks tab (same chrome as `/plugins`)
- Embed: `/hooks` and `/plugins` are **visible** (not brand-hidden); marketplace still hidden (`product_welcome.rs`)
- ACP: `x.ai/hooks/list` → **empty** `HooksListResponse`; `x.ai/hooks/action` → Unsupported (“use next-code hooks config”) — `src/cli/face_plugins.rs`
- Modal UX still assumes Grok paths (`~/.grok/hooks`) in labels/helpers — cosmetic debt for embed

### C. Bundle plugins (PR49 sibling — not this rewrite)
- Face Plugins tab + `face_plugins` discover `~/.next-code/plugins` / project plugins (Grok-style **directory bundles**: skills, agents, optional `hooks/`)
- That is **not** OpenCode’s npm/TS plugin loader
- See `PLAN-20260721-plugins-grok-vs-nextcode.md` — do not merge runtimes; only share the modal

### D. Note on `next-code-plugin-core`
Earlier inventory / docs may mention a QuickJS plugin event enum. On this disk snapshot, **`crates/next-code-plugin-core` is not present**; only `next-code-hooks` (+ Grok marketplace/types stubs). Plan assumes hooks work centers on `next-code-hooks` unless that crate returns.

---

## Comparison (load-bearing)

| Dimension | OpenCode | next-code today | Follow? |
|-----------|----------|-----------------|--------|
| Author language | JS/TS in-process | Shell/HTTP scripts (+ optional plugin exe) | Keep next-code for users; optional later TS author path |
| Config | Plugin files + `opencode.json` plugin list | `hooks.toml` + v1 `[hooks]` | Keep toml; optionally also load hooks from plugin bundles |
| Tool gate | Mutate args / throw | Exit 2 / JSON deny | Keep exit protocol; document OpenCode throw as “author DX” analogue |
| Session idle notify | `event` + `session.idle` | `SessionIdle` + observer commands | Map names; already covered |
| Chat/LLM transforms | `chat.params`, system transform | Not in hooks crate (elsewhere / missing) | Optional Phase 3 — OpenCode-inspired seams |
| UI | Docs + ecosystem; no `/hooks` modal | Face Extensions Hooks tab (stub) | Wire UI → next-code registry |
| Paths | `~/.config/opencode/…` | `~/.next-code/…` | Stay on next-code home |

**Verdict:** “Follow OpenCode” = adopt **patterns** (named seams, sequential mutate hooks for in-process plugins, clear global/project load order, rich event vocabulary), **not** replace shell hooks with Bun plugins.

---

## Recommendation

### Product split (keep forever)
1. **Lifecycle hooks** (`next-code-hooks`) — user/ops automation via `hooks.toml` and CLI. Primary answer to “run something on SessionStart / PreToolUse”.
2. **Bundle plugins** (Face Plugins / `~/.next-code/plugins`) — skills/agents/MCP packaging. May **contribute** hook definitions (Grok `hooks/*.json` inside a plugin) into the same registry later.
3. **OpenCode-style in-process plugins** (future / optional) — TS callbacks that mutate tool args / chat params. Only if we revive a JS runtime; do not block Face wiring.

### What to copy from OpenCode
- Dotted / domain-oriented seam names as **documented aliases** of existing `HookEvent`s (e.g. `tool.execute.before` ↔ `PreToolUse`).
- Sequential **input/output mutation** story for any future in-process handlers (today: command hooks already receive stdin JSON and can deny).
- Auto-discovery layout: global + project dirs with deterministic merge (next-code already layers toml; extend to plugin-bundled hooks).
- Rich session event set (`session.idle`, `session.error`, …) — mostly already in `HookEvent`; ensure dispatch sites fire them.
- Custom tools via plugins as a **separate** registration path (OpenCode `tool: {…}`) — belongs under Plugins tab / tool registry, not Hooks tab.

### What to wire into next-code (`~/.next-code`)
| Path | Role |
|------|------|
| `~/.next-code/hooks.toml` | User lifecycle hooks (canonical) |
| `<cwd>/.next-code/hooks.toml` | Project hooks (trust policy TBD — mirror Grok folder-trust or next-code existing trust) |
| `$NEXT_CODE_HOOKS_CONFIG` | Override file |
| `~/.next-code/config.toml` `[hooks]` | Legacy v1 single-command entries (keep merge) |
| `~/.next-code/plugins/*/hooks/` | Optional later: import Grok-style JSON hook defs into registry |
| **Not** `~/.config/opencode/` | Do not dual-home unless explicit compat flag |

### Face `/hooks` vs OpenCode semantics
- **Copy UI:** keep Extensions modal Hooks tab chrome (list, enable/disable, reload, trust badge).
- **Do not copy OpenCode semantics into the modal:** OpenCode has no equivalent modal; authors edit TS plugins.
- **Do not copy Grok disk layout** (`~/.grok/hooks`) for embed brain — map rows from `next_code_hooks::load_hooks_config` / registry.
- ACP: implement real `x.ai/hooks/list` + useful subset of `x.ai/hooks/action` (reload, enable/disable, maybe “open config path”); editing full handlers can stay “open file” / Unsupported initially.

### Relation to PR49 / plugins plan
- Shared: Extensions modal shell, ACP method family `x.ai/{hooks,plugins}/*`, brand visibility.
- Separate: plugin install/list vs hook list; empty hooks stub must not block plugins work.
- If a bundle plugin ships `hooks/`, list them under Hooks tab with `source=plugin` (integration note only — implement after list wire).

---

## Steps (implement after **go ahead**)

### Phase 0 — Docs + aliases (small)
1. [x] Update `docs/HOOKS.md` to describe v2 `hooks.toml` as canonical; keep v1 as legacy.
2. [x] Add OpenCode → next-code event mapping table (dotted → PascalCase).
3. [x] Clarify in Face/embed docs that `/hooks` manages next-code hooks, not OpenCode/Bun plugins.

### Phase 1 — Face ACP wire (recommended first build)
1. [x] `hooks_list_payload`: load `load_hooks_config()`, map each handler → `xai_hooks_plugins_types::HookInfo` (event, command/url label, enabled, source path user vs project).
2. [x] `hooks_action_payload`: support Reload + Enable/Disable (rewrite toml or overlay state file under `~/.next-code/`); return clear Unsupported for Install/marketplace.
3. [x] Fix Extensions modal source labels for embed (`~/.next-code/hooks.toml` not `~/.grok/hooks`) where cheap.
4. [x] Tests: list non-empty with temp hooks.toml; action enable/disable.

### Phase 2 — Runtime completeness
1. [ ] Audit which `HookEvent`s are actually dispatched from app-core / session / compaction; fill gaps that OpenCode users expect (`SessionIdle`, compaction, permission).
2. [ ] Optional: merge hooks from enabled bundle plugins’ `hooks/` dirs into registry (align with Plugins tab).

### Phase 3 — OpenCode-like in-process (optional, large)
1. [ ] Only if product wants TS plugin authors: revive/add JS runtime hooks with OpenCode-shaped names and sequential mutate triggers.
2. [ ] Do **not** deprecate shell hooks in the same change.

**Out of scope for first PR:** Full OpenCode plugin loader, npm/Bun install, replacing `next-code-hooks`, marketplace hooks.

---

## Files to touch (Phase 1)

- `src/cli/face_plugins.rs` — real list/action
- `src/cli/pager_agent.rs` — already routes `x.ai/hooks/*` (verify)
- `crates/next-code-hooks/src/cli.rs` / config — reuse list helpers; maybe export `list_hook_infos()`
- `crates/xai-hooks-plugins-types` — only if HookInfo fields need next-code fields
- `docs/HOOKS.md` + this plan’s mapping appendix
- Light Face label fixes in `extensions_modal.rs` (embed-only paths)

---

## Evidence citations

1. **OpenCode Hooks interface:** `tmp/opencode/packages/plugin/src/index.ts` — `export interface Hooks` (`tool.execute.before|after`, `event`, `chat.*`, …)
2. **OpenCode trigger:** `tmp/opencode/packages/opencode/src/plugin/index.ts` — `Plugin.trigger` sequential `fn(input, output)`
3. **OpenCode docs:** `tmp/opencode/packages/web/src/content/docs/plugins.mdx` — dirs, npm, examples (`session.idle`, `.env` throw)
4. **next-code config:** `crates/next-code-hooks/src/config.rs` — layers + `HookEvent`
5. **next-code runtime:** `crates/next-code-app-core/src/tool/mod.rs` — load + merge v1 + `dispatch_hooks`
6. **Face stub:** `src/cli/face_plugins.rs` — empty list / Unsupported action
7. **Plugins sibling:** `docs/plans/PLAN-20260721-plugins-grok-vs-nextcode.md`

---

## Open questions (≤3)

1. Should project `.next-code/hooks.toml` require explicit trust (Grok-style) before running, or trust like other next-code project config today?
2. Phase 1 enable/disable: mutate `hooks.toml` on disk, or a separate `~/.next-code/hooks-state.json` overlay?
3. Do you want Phase 3 (OpenCode-like in-process TS hooks) on the roadmap, or only shell/HTTP forever?

---

## Status
**Phase 0–1 implemented** on branch `pr-face-hooks-wire` (Face ACP list/enable/disable + docs). Later phases still open.

Waiting only for review/merge of that PR — not for another “go ahead” on Phase 0–1.
