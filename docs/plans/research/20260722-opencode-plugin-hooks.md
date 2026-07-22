# Research: OpenCode plugin = hooks model

**Date:** 2026-07-22  
**Scope:** Deep research only — no production code.  
**Sources:** local clone `.tmp-research-plugins/opencode` @ `0a601cf` (`dev`), Pi clone `.tmp-research-plugins/pi`, next-code docs/crates, [opencode.ai/docs/plugins](https://opencode.ai/docs/plugins.md), [opencode.ai/docs/config](https://opencode.ai/docs/config.md).  
**Upload note:** `uploads/opencode-2.md` is a GitHub landing-page scrape (stars/install); not useful for plugin ABI detail.

---

## Summary

OpenCode’s public extensibility story is: **a plugin is a Bun/TS module that returns a `Hooks` object** (and optionally a separate **TUI** plugin against OpenTUI). Global vs project is first-class (`~/.config/opencode` vs `.opencode` / `opencode.json`). That is the opposite of next-code’s current split: **out-of-process `hooks.toml`** (lifecycle) **vs Face bundle plugins** (skills/agents data). Pi sits closer to OpenCode (in-process TS extensions with tools + UI), but with a richer `ExtensionAPI` and project trust.

For next-code as **platform + default product**: steal OpenCode’s **named extension points, load-order/provenance, server-vs-UI split idea, and dual file/npm discovery** — implement them on next-code’s **argv/HTTP/ACP** runners. Avoid making “plugin = in-process TS hooks” the product ABI.

---

## 1. OpenCode: what “plugin = hooks” means

### 1.1 Contract

Published package: `@opencode-ai/plugin`  
Clone path: `.tmp-research-plugins/opencode/packages/plugin/`

| Export | Role |
|--------|------|
| `Plugin` | `(input: PluginInput, options?) => Promise<Hooks>` |
| `Hooks` | Optional callbacks + optional `tool` map + `auth` / `provider` |
| `./tool` | `tool({ description, args, execute })` helper (Zod) |
| `./tui` | Separate `TuiPlugin` — OpenTUI/Solid host API |
| `./v2/effect`, `./v2/promise` | Emerging imperative Effect/Promise plugin API |

Core type (abbreviated) from `packages/plugin/src/index.ts`:

```ts
export type Plugin = (input: PluginInput, options?: PluginOptions) => Promise<Hooks>

export interface Hooks {
  dispose?: () => Promise<void>
  event?: (input: { event: Event }) => Promise<void>
  config?: (input: Config) => Promise<void>
  tool?: { [key: string]: ToolDefinition }
  auth?: AuthHook
  provider?: ProviderHook
  "chat.message"?: (input, output) => Promise<void>
  "chat.params"?: (input, output) => Promise<void>
  "tool.execute.before"?: (input, output) => Promise<void>
  "tool.execute.after"?: (input, output) => Promise<void>
  "permission.ask"?: (input, output) => Promise<void>
  // … plus experimental.* transforms
}
```

**Mental model:** one module registers many lifecycle seams by returning a bag of optional named handlers. There is no separate `hooks.toml`; the plugin *is* the hook registry.

### 1.2 PluginInput (host capabilities)

From the same file:

- `client` — `@opencode-ai/sdk` HTTP client (talk to the OpenCode server)
- `project`, `directory`, `worktree`
- `$` — **Bun shell** (`Bun.$`)
- `serverUrl`
- `experimental_workspace.register(...)` — workspace adapters

Authors are expected to run as **in-process TypeScript on Bun**, with full host privileges.

### 1.3 Runtime: load → apply → trigger

Host: `.tmp-research-plugins/opencode/packages/opencode/src/plugin/index.ts`

1. **Internal plugins** (auth providers: Codex, Copilot, Cloudflare, Azure, xAI, …) are imported directly and always return `Hooks`.
2. **External plugins** come from derived `cfg.plugin_origins` (not a persisted field — see config).
3. `PluginLoader.loadExternal` (`plugin/loader.ts`) resolves file/npm → entrypoint → `import()`.
4. Each plugin’s `server` function is awaited; returned `Hooks` are pushed onto an instance-local array.
5. `Plugin.trigger(name, input, output)` walks hooks **sequentially** and mutates the shared `output` object.

```280:292:.tmp-research-plugins/opencode/packages/opencode/src/plugin/index.ts
    const trigger = Effect.fn("Plugin.trigger")(function* <...>(name, input, output) {
      ...
      for (const hook of s.hooks) {
        const fn = hook[name] as any
        if (!fn) continue
        yield* Effect.promise(async () => fn(input, output))
      }
      return output
    })
```

**Control style:** mutate `output` (args, status, env, prompts). Docs/examples also **throw** from `tool.execute.before` to block (e.g. `.env` protection). No exit-code protocol.

Bus fan-out: all hooks’ generic `event` listeners receive session-scoped bus events after init.

### 1.4 Two plugin kinds: server vs TUI

`PluginKind = "server" | "tui"` in `packages/opencode/src/plugin/shared.ts`.

- Package exports can expose `./server` and `./tui` (or legacy default export = server).
- `PluginModule` types enforce mutual exclusion: `server` vs `tui?: never` and vice versa (`packages/plugin/src/index.ts`, `tui.ts`).
- TUI plugins get a large OpenTUI API: routes, slots, dialogs, keymap, toast, theme, plugin install UI (`packages/plugin/src/tui.ts` — `TuiPluginApi`).

This is **one product, two in-process guest surfaces** (agent runtime + terminal UI), both TypeScript.

### 1.5 Global vs project (and load order)

**Docs** ([plugins.md](https://opencode.ai/docs/plugins.md)):

| Source | Path |
|--------|------|
| Global plugins dir | `~/.config/opencode/plugins/` |
| Project plugins dir | `.opencode/plugins/` |
| npm / path specs | `"plugin": [...]` in `opencode.json` |

Documented load order (all hooks run; sequence follows merge):

1. Global config (`~/.config/opencode/opencode.json`)
2. Project config (`opencode.json`)
3. Global plugin directory
4. Project plugin directory

**Code:** auto-discovery scans `{plugin,plugins}/*.{ts,js}` under each config directory:

```18:29:.tmp-research-plugins/opencode/packages/opencode/src/config/plugin.ts
export async function load(dir: string) {
  ...
  for (const item of await Glob.scan("{plugin,plugins}/*.{ts,js}", { cwd: dir, ... })) {
    plugins.push(pathToFileURL(item).href)
  }
}
```

Provenance is kept as `plugin_origins: { spec, source, scope: "global" | "local" }` in `packages/opencode/src/config/config.ts` (derived; stripped on write). Dedup prefers later wins by package name / file URL (`ConfigPlugin.deduplicatePluginOrigins`).

**npm:** installed via Bun into `~/.cache/opencode/node_modules/` (docs). Local plugins may use `.opencode/package.json` + `bun install` at startup.

**Config layers** (broader than plugins): remote `.well-known/opencode` → global → `OPENCODE_CONFIG` → project → `.opencode` dirs → inline → managed/MDM ([config.md](https://opencode.ai/docs/config.md)). Useful enterprise pattern; next-code has weaker “managed” story today.

### 1.6 Secondary: experimental shell hooks

SDK config still has a **thin** shell-command hook surface (`packages/sdk/js/src/gen/types.gen.ts`):

```ts
experimental?: {
  hook?: {
    file_edited?: { [pathPattern: string]: Array<{ command: string[]; environment?: ... }> }
    session_completed?: Array<{ command: string[]; environment?: ... }>
  }
}
```

Only two events. Community notes (e.g. symposium) treat this as incomplete vs the JS plugin path. **Do not** copy this as a second competing system alongside a rich hooks.toml — next-code already has the better multilang side.

### 1.7 V2 Effect plugins (in flight)

`packages/plugin/src/v2/effect/README.md`: plugins `define({ id, effect })` and **imperatively** `ctx.agent.transform` / `ctx.catalog.transform` / runtime AISDK hooks, with scope disposal. Still TypeScript/Effect-first; aimed at platform authors, not shell one-liners.

---

## 2. Pi contrast

Clone: `.tmp-research-plugins/pi`  
Primary doc: `packages/coding-agent/docs/extensions.md`

| Dimension | OpenCode | Pi |
|-----------|----------|----|
| Unit of extension | Plugin → `Hooks` object | Extension factory → `ExtensionAPI` |
| Language | Bun/TS in-process | Node/TS in-process (jiti) |
| Global | `~/.config/opencode/plugins/` | `~/.pi/agent/extensions/` |
| Project | `.opencode/plugins/` | `.pi/extensions/` **after trust** |
| Tools | `Hooks.tool` / `tool()` | `pi.registerTool()` |
| Slash / commands | Via TUI keymap / command layer | `pi.registerCommand()` |
| UI | Separate `TuiPlugin` (OpenTUI slots) | Same process: `ctx.ui.custom()`, dialogs, widgets |
| Packages | npm plugin specs + dirs | `pi install` npm/git packages; manifest or convention dirs |
| Lifecycle API | Named dotted hooks + generic `event` | `pi.on("tool_call" \| "session_start" \| …)` |
| Policy block | mutate / throw | `{ block: true, reason }` from event handlers |

Pi is **broader in one API** (tools + slash + TUI + events). OpenCode **splits** server hooks vs TUI plugins but keeps both in-process TS. Neither is language-agnostic.

next-code plan already recorded this: `docs/plans/PLAN-20260722-pi-full-custom-platform.md` §B3 — OpenCode is “everything is hooks” *ideas*, not multilang ABI.

---

## 3. next-code contrast (hooks.toml + Face plugins)

Canonical docs:

- `docs/HOOKS.md` — v2 lifecycle hooks
- `docs/plugins.md` — Grok-style Face bundles
- Wire DTOs: `crates/xai-hooks-plugins-types/src/lib.rs` (`x.ai/hooks/*`, `x.ai/plugins/*`)

### 3.1 Explicit product split

From `docs/HOOKS.md`:

> Face `/hooks` lists… handlers — it does **not** use OpenCode’s in-process JS plugin `Hooks` model.

| Surface | Config | Runner | Face |
|---------|--------|--------|------|
| Lifecycle | `~/.next-code/hooks.toml` + project `.next-code/hooks.toml` | `command` / `http` / `agent` / `plugin` exe — stdin JSON, exit 0/2 | `/hooks` tab |
| Bundles | `~/.next-code/plugins/<name>/` (+ project) | Declarative: skills, agents; `hooks/hooks.json` ingest **deferred** | `/plugins` tab |

Event name alias table (docs only) already maps OpenCode dotted names → `PreToolUse` / `PostToolUse` / etc. (`docs/HOOKS.md` “OpenCode event name aliases”).

Phase 3 “OpenCode-like in-process TypeScript hooks” is **explicitly deferred** in that same doc.

### 3.2 Why the split matters

OpenCode collapses **policy + tools + auth + (optional) UI** into one TS guest. next-code:

- Already has **multilang** policy (`crates/next-code-hooks/src/execute.rs`).
- Removed QuickJS guest VM (#49 / `docs/plugins.md` “Removed”).
- Face is Rust; third-party UI is ACP widgets, not OpenTUI slots.

Copying OpenCode’s identity “plugin = Hooks” would **re-merge** what next-code deliberately separated and fight the herdr-shaped roadmap in `PLAN-20260722-pi-full-custom-platform.md`.

---

## 4. Comparison matrix (platform-relevant)

| Concern | OpenCode | Pi | next-code today |
|---------|----------|-----|-----------------|
| Extension identity | Plugin module = hooks (+ optional TUI) | Extension = API registrations | **Two products:** hooks.toml vs plugin bundles |
| Host language | Bun runtime | Node + jiti | Rust host |
| Guest language | TS/JS only (primary) | TS only | Hooks: any exe; bundles: data |
| Global vs project | Strong; provenance in `plugin_origins` | Strong + **trust gate** for project | Both layers; Face trust for hooks Unsupported (always load project) |
| Block tool | throw / mutate | `{ block }` | exit 2 / fail_closed |
| Custom tools | In-process `tool()` | `registerTool` | MCP / Rust; argv tools planned |
| Custom UI | TuiPlugin / OpenTUI | `ctx.ui.custom` | Face fixed + ACP |
| Package install | Bun npm cache | `pi install` npm/git | git/local Face install |
| Dual incomplete ABIs | JS plugins + experimental shell hooks | Mostly one TS API | Risk if plugin `hooks/` JSON never merges into hooks.toml |

---

## 5. Steal vs avoid (for “platform + default product”)

### Steal

1. **Named extension-point vocabulary** — keep OpenCode/Pi-style names as *aliases* into next-code `HookEvent` (already started in `docs/HOOKS.md`). Good for docs and migration guides.
2. **Provenance + load order** — `plugin_origins`-style `{ spec, source, scope }` for both hooks handlers and packages; dedupe by identity, later wins. See `packages/opencode/src/config/plugin.ts`.
3. **Server vs UI kind** — conceptually separate **agent lifecycle** from **UI chrome**. OpenCode implements both as TS; next-code should keep lifecycle = hooks runners, UI = Face/ACP (or external pane), never OpenTUI-in-Face.
4. **Dual discovery** — auto-load files from global + project dirs **and** allow registry/npm/git specs in config (OpenCode pattern; Pi packages). Map “files” to argv entrypoints or skill trees, not `import()`.
5. **Shared mutable output bag** — OpenCode’s `(input, output) => void` maps cleanly to next-code stdin JSON + allow/deny; keep that contract for argv plugins.
6. **Internal first-party plugins** — OpenCode ships auth as internal `Hooks` plugins. For next-code **default product**, ship first-party behaviors as Rust/builtin or curated hooks packages, not as a required TS guest.
7. **Enterprise config layering** — remote/org → user → project → managed overrides (OpenCode config.md). Useful for default-product fleets.

### Avoid

1. **Plugin ≡ in-process Hooks object** as the public ABI — conflicts with multilang platform goal and #49.
2. **Bun.$ / Node guest as host assumption** — ties ecosystem to one runtime.
3. **Throw-to-deny** as primary policy — prefer exit codes / structured allow-deny (already next-code).
4. **In-process TUI plugins** (OpenTUI Solid slots, Pi `ui.custom`) inside Face — Face stays Rust; optional external pane only.
5. **A second thin shell-hook config** (`experimental.hook` with two events) alongside a rich hooks system — consolidate into `hooks.toml`.
6. **Effect v2 as third-party authoring surface** — fine for internal platform team; too heavy for default-product extension authors.
7. **Leaving plugin-bundled hooks disconnected** — OpenCode loads plugin code as hooks automatically; next-code currently defers `hooks/` JSON → registry (`docs/HOOKS.md`, `docs/plugins.md`). For a coherent “package”, **compile package `[[hooks]]` into hooks.toml registry** (plan Phase 1–2), don’t invent a third runtime.

### Platform vs default product (how to apply)

| Layer | Recommendation |
|-------|----------------|
| **Platform** | Manifest declares WHAT; runners HOW (argv/HTTP/MCP/ACP). Steal OpenCode’s *seam list* and *scope/provenance*, not its Bun guest. |
| **Default product** | Ship curated skills/agents + a small set of first-party hook recipes; Face `/hooks` + `/plugins` remain sibling UIs wired to one compiled registry over time. |
| **Optional TS** | Treat TS as *one* guest recipe (like herdr’s OpenCode adapter JS), never the only one. |

Aligned one-liner (from existing plan): *Pi-breadth of surfaces, herdr/next-code multilang ABI, Face remains Rust UI.*

---

## 6. Evidence index (paths)

| Claim | Path / URL |
|-------|------------|
| `Plugin` / `Hooks` types | `.tmp-research-plugins/opencode/packages/plugin/src/index.ts` |
| `tool()` helper | `.../packages/plugin/src/tool.ts` |
| TUI plugin API | `.../packages/plugin/src/tui.ts` |
| Trigger loop / internal plugins | `.../packages/opencode/src/plugin/index.ts` |
| Loader resolve/import | `.../packages/opencode/src/plugin/loader.ts` |
| server vs tui kinds | `.../packages/opencode/src/plugin/shared.ts` |
| Dir auto-discovery | `.../packages/opencode/src/config/plugin.ts` |
| `plugin_origins` merge | `.../packages/opencode/src/config/config.ts` |
| Effect v2 plugin README | `.../packages/plugin/src/v2/effect/README.md` |
| Experimental shell hooks schema | `.../packages/sdk/js/src/gen/types.gen.ts` (`experimental.hook`) |
| Official plugins docs | https://opencode.ai/docs/plugins.md |
| Official config / scopes | https://opencode.ai/docs/config.md |
| Pi extensions | `.tmp-research-plugins/pi/packages/coding-agent/docs/extensions.md` |
| next-code hooks | `docs/HOOKS.md`, `crates/next-code-hooks/src/config.rs` |
| next-code Face plugins | `docs/plugins.md`, `crates/xai-hooks-plugins-types/src/lib.rs` |
| Prior synthesis | `docs/plans/PLAN-20260722-pi-full-custom-platform.md` §B3, comparison matrix |

---

## 7. Open questions (research leftovers)

1. How often is `permission.ask` actually invoked in current OpenCode (community bug reports exist)? next-code should not assume parity without a live probe.
2. When does OpenCode flip default authors from v1 `Hooks` return object → v2 Effect `define`? Track before any compatibility layer.
3. Should next-code’s package ABI use OpenCode-like dotted names in manifests, or only document aliases?

---

## Status

Research complete. No code changes. Suitable input to platform package ABI design; does not supersede `PLAN-20260722-pi-full-custom-platform.md` Option B′.
