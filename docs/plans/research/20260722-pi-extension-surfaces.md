# Research: Pi extension surfaces (“dynamic like Pi”)

**Kind:** DEEP RESEARCH only — no production code  
**Date:** 2026-07-22  
**Upstream:** [earendil-works/pi](https://github.com/earendil-works/pi)  
**Local clone:** `C:\Users\ADMIN\Documents\Projects\next-code\.tmp-research-plugins\pi`  
**Clone tip:** `dd6bea4` (2026-07-21) — coding-agent package version **0.81.1**  
**Scope:** Every author-facing extension surface that makes Pi “self-extensible,” with package path, API shape, language, and composition with core.

Companion product plan (surfaces × herdr multilang ABI): `docs/plans/PLAN-20260722-pi-full-custom-platform.md`.

---

## 1. Architecture map (how surfaces compose)

```
┌─────────────────────────────────────────────────────────────────┐
│  @earendil-works/pi-coding-agent  (CLI + TUI modes + SDK)      │
│  ResourceLoader → PackageManager → Extensions (jiti)            │
│  ExtensionAPI / ExtensionContext / ExtensionUIContext           │
└───────────────┬─────────────────────┬───────────────────────────┘
                │                     │
    ┌───────────▼──────────┐  ┌───────▼────────────────────────┐
    │ pi-agent-core        │  │ pi-ai                          │
    │ Agent loop, tools,   │  │ Providers, stream APIs, OAuth, │
    │ state, compaction    │  │ model catalogs                 │
    └──────────────────────┘  └────────────────────────────────┘
                │
    ┌───────────▼──────────┐
    │ pi-tui               │
    │ Component, overlays, │
    │ differential render  │
    └──────────────────────┘
```

| Package | Path | Role in extension story |
|---------|------|-------------------------|
| `@earendil-works/pi-coding-agent` | `packages/coding-agent` | **Primary extension host**: load TS modules, events, tools, commands, packages, trust, SDK |
| `@earendil-works/pi-tui` | `packages/tui` | Render primitives for custom UI (`Component`, overlays, editors) |
| `@earendil-works/pi-agent-core` | `packages/agent` | Agent loop + tool execution modes; coding-agent wraps tools into this |
| `@earendil-works/pi-ai` | `packages/ai` | Provider/stream registry; `registerProvider` plugs here |
| `@earendil-works/pi-server` | `packages/server` | Experimental server wrapping coding-agent (not an extension ABI) |

**Language of extensions:** TypeScript/JavaScript only, loaded via **jiti** (no compile step). Peer imports of core packages are virtualized into the loader (`packages/coding-agent/src/core/extensions/loader.ts` → `VIRTUAL_MODULES` / aliases for `typebox`, `pi-ai`, `pi-tui`, `pi-agent-core`, `pi-coding-agent`).

**Canonical API type:** `ExtensionAPI` in  
`packages/coding-agent/src/core/extensions/types.ts` (≈ L1172–1407).  
**Docs:** `packages/coding-agent/docs/extensions.md` (large; lifecycle diagram ≈ L273–348).

**Philosophy (from README + packages docs):** minimal core; capabilities ship as **packages** bundling extensions + skills + prompts + themes. Extensions run with **full user process privileges** — no in-process sandbox (`docs/security.md`, root `README.md` Permissions section).

---

## 2. Surface catalog (exhaustive)

### 2.1 TUI / custom UI

| Item | Detail |
|------|--------|
| **Package path** | Host: `packages/coding-agent` (`ExtensionUIContext`); primitives: `packages/tui` (`@earendil-works/pi-tui`) |
| **Docs** | `packages/coding-agent/docs/tui.md`, `docs/themes.md`, `docs/keybindings.md` |
| **Language** | TypeScript implementing `Component` / factories |
| **API shape** | See below |

**`ExtensionUIContext`** (`types.ts` ≈ L129–280) — mode-backed (TUI / RPC / print):

| Method | Purpose |
|--------|---------|
| `select` / `confirm` / `input` / `editor` | Dialog primitives (+ optional `signal` / `timeout`) |
| `notify` | Toast-style notice |
| `custom(factory, { overlay?, overlayOptions?, onHandle? })` | Full focusable TUI component; overlays with anchors/sizing |
| `setWidget` / `setStatus` / `setFooter` / `setHeader` | Persistent chrome |
| `setWorkingIndicator` / `setWorkingMessage` / `setWorkingVisible` | Streaming loader |
| `setHiddenThinkingLabel` | Collapsed thinking label |
| `setEditorComponent` / `getEditorComponent` | Replace main editor (extend `CustomEditor`) |
| `addAutocompleteProvider` | Stack autocomplete |
| `pasteToEditor` / `setEditorText` / `getEditorText` | Editor I/O |
| `onTerminalInput` | Raw key stream (interactive only) |
| `theme` / `getTheme` / `setTheme` / `getAllThemes` | Theme control |
| `getToolsExpanded` / `setToolsExpanded` | Tool output expand state |
| `setTitle` | Terminal title |

**`Component` contract** (`docs/tui.md`):

```ts
interface Component {
  render(width: number): string[];
  handleInput?(data: string): void;
  wantsKeyRelease?: boolean;
  invalidate(): void;
}
```

**Composition with core:** Interactive mode injects a real `ExtensionUIContext`; RPC mode implements a subset (see `examples/extensions/rpc-demo.ts`). Print/JSON modes: `hasUI` is false — dialogs must be guarded. Custom tool `renderCall` / `renderResult` also return `Component`s and receive `Theme` from the interactive theme system.

**Related non-code UI surfaces:**

- **Themes:** JSON files (`docs/themes.md`); schema at `packages/coding-agent/src/modes/interactive/theme/theme-schema.json`
- **Keybindings:** `~/.pi/agent/keybindings.json` (`docs/keybindings.md`); extensions can also `registerShortcut`
- **Message/entry renderers:** `pi.registerMessageRenderer` / `registerEntryRenderer` for custom session entry types

**Examples:** `examples/extensions/snake.ts`, `doom-overlay/`, `modal-editor.ts`, `custom-footer.ts`, `overlay-qa-tests.ts`, `preset.ts` (SelectList pattern).

---

### 2.2 Hooks / lifecycle events (`pi.on`)

| Item | Detail |
|------|--------|
| **Package path** | `packages/coding-agent/src/core/extensions/{types,runner,loader}.ts` |
| **Docs** | `docs/extensions.md` § Events |
| **Language** | TypeScript handlers |
| **API shape** | `pi.on(event, handler)` with typed event + optional result |

**Full event union** (`types.ts` `ExtensionEvent` ≈ L1023–1048):

| Event | Intercept / mutate? | Typical use |
|-------|---------------------|-------------|
| `project_trust` | Return `{ trusted, remember? }` | Own project trust decision (user/global/CLI extensions only) |
| `resources_discover` | Return extra `skillPaths` / `promptPaths` / `themePaths` | Dynamic resource roots |
| `session_start` / `session_shutdown` / `session_info_changed` | Observe | Init/cleanup, rename |
| `session_before_switch` / `session_before_fork` | `{ cancel? }` | Guard `/new`, `/resume`, `/fork` |
| `session_before_compact` / `session_compact` | Cancel or supply custom summary | Custom compaction |
| `session_before_tree` / `session_tree` | Cancel / custom branch summary | `/tree` |
| `input` | `continue` / `transform` / `handled` | Prompt rewrite, early handle |
| `before_agent_start` | Inject `message`, replace `systemPrompt` | Prompt/system injection |
| `agent_start` / `agent_end` / `agent_settled` | Observe | Status integrations |
| `turn_start` / `turn_end` | Observe | Per-turn hooks |
| `message_start` / `message_update` / `message_end` | `message_end` → replace message | Overflow normalize, logging |
| `context` | Replace `messages` | Pre-LLM context edit |
| `before_provider_headers` | Mutate `headers` in place | Tracing |
| `before_provider_request` | Replace payload | Proxy shaping |
| `after_provider_response` | Observe status/headers | Diagnostics |
| `tool_call` | `{ block?, reason? }` (mutate `input` in place) | Permission gates |
| `tool_result` | Override content/details/isError/usage | Sanitize / enrich |
| `tool_execution_start` / `_update` / `_end` | Observe | Progress UI |
| `user_bash` | Custom `operations` or full `result` | Sandbox / SSH / interactive shell |
| `model_select` / `thinking_level_select` | Observe | Status chrome |

**Composition:** `ExtensionRunner` binds handlers after load; agent loop in `pi-agent-core` emits tool/message events that coding-agent maps into extension events. Lifecycle diagram in `docs/extensions.md` L273–348 is the authoritative order (trust → session_start → resources_discover → input → before_agent_start → turns → settle).

**Note:** Pi does **not** use a separate “hooks.toml” file — hooks are **in-process TypeScript** subscribed at extension factory time.

**Examples:** `permission-gate.ts`, `protected-paths.ts`, `project-trust.ts`, `custom-compaction.ts`, `inline-bash.ts`, `interactive-shell.ts`, `bash-spawn-hook.ts`.

---

### 2.3 Slash commands / prompt templates / skill commands

Three slash sources (`packages/coding-agent/src/core/slash-commands.ts`):

```ts
export type SlashCommandSource = "extension" | "prompt" | "skill";
```

#### A. Extension commands — `pi.registerCommand`

```ts
// types.ts ≈ L1149–1155
interface RegisteredCommand {
  name: string;
  sourceInfo: SourceInfo;
  description?: string;
  getArgumentCompletions?: (argumentPrefix: string) => AutocompleteItem[] | null | Promise<...>;
  handler: (args: string, ctx: ExtensionCommandContext) => Promise<void>;
}
```

`ExtensionCommandContext` extends `ExtensionContext` with session control: `newSession`, `fork`, `navigateTree`, `switchSession`, `reload`, `waitForIdle`, `getSystemPromptOptions` (`types.ts` ≈ L344–378).

Handlers run **immediately**, even during streaming (SDK docs: extension commands bypass queue).

#### B. Prompt templates — markdown → `/name`

| Item | Detail |
|------|--------|
| **Path** | `packages/coding-agent/src/core/prompt-templates.ts` |
| **Docs** | `docs/prompt-templates.md` |
| **Locations** | `~/.pi/agent/prompts/*.md`, `.pi/prompts/*.md` (trusted), packages, settings, `--prompt-template` |
| **Shape** | Frontmatter `description`, `argument-hint`; body with `$1`, `$@`, `${1:-default}`, etc. |
| **Language** | Markdown only (no code) |

#### C. Skill commands — `/skill:name`

When `enableSkillCommands` is true, skills register as `/skill:name` (optional args appended as `User: <args>`). Docs: `docs/skills.md`.

#### D. Built-in slash set

`BUILTIN_SLASH_COMMANDS` in `slash-commands.ts` (settings, model, fork, tree, trust, login, reload, compact, …). Extensions **add** to this catalog via `registerCommand`; `pi.getCommands()` returns the live list.

**Also:** `pi.registerShortcut(keyId, { description?, handler })` and `pi.registerFlag(name, { type: "boolean"|"string", ... })` + `pi.getFlag` — CLI/TUI affordances, not slash, but same registration family (`ExtensionAPI` L1236–1256).

**Examples:** `examples/extensions/commands.ts`, `preset.ts`, `plan-mode/`, `handoff.ts`.

---

### 2.4 Tools

| Item | Detail |
|------|--------|
| **Package path** | Definition: coding-agent `ToolDefinition`; execution: `pi-agent-core` |
| **Docs** | `docs/extensions.md` § Custom Tools; SDK `docs/sdk.md` § Tools |
| **Language** | TypeScript; params via **typebox** |
| **API** | `pi.registerTool(definition)` or SDK `defineTool` / `customTools` |

**`ToolDefinition`** (`types.ts` ≈ L440–487):

| Field | Role |
|-------|------|
| `name`, `label`, `description` | LLM + UI |
| `parameters` | TypeBox schema |
| `execute(toolCallId, params, signal, onUpdate, ctx)` | Implementation |
| `promptSnippet` / `promptGuidelines` | System prompt contribution |
| `executionMode` | `"sequential"` \| `"parallel"` |
| `prepareArguments` | Pre-validate shim |
| `renderShell` | `"default"` \| `"self"` |
| `renderCall` / `renderResult` | Custom TUI |

**Built-in tools** (coding-agent factories): `read`, `bash`, `edit`, `write`, `grep`, `find`, `ls` — overridable by registering the same name. Active set controllable via `pi.setActiveTools` / `getActiveTools` / `getAllTools`, and session settings.

**Dynamic tools:** can register after `session_start` or from a command (`examples/extensions/dynamic-tools.ts`, `kimi-deferred-tools.ts`).

**Composition:** Registered tools are wrapped (`wrapper.ts`) into `pi-agent-core` `AgentTool`s; `tool_call` / `tool_result` extension events sit around execution. SDK consumers can also pass `customTools` without disk extensions.

**Examples:** `hello.ts`, `todo.ts`, `tool-override.ts`, `ssh.ts`, `subagent/`, `structured-output.ts`, `truncated-tool.ts`.

---

### 2.5 Skills / resources / themes / context files

#### Skills (Agent Skills standard)

| Item | Detail |
|------|--------|
| **Path** | `packages/coding-agent/src/core/skills.ts` |
| **Docs** | `docs/skills.md` |
| **Language** | Markdown `SKILL.md` + optional scripts/assets (model-invoked) |
| **Shape** | Frontmatter: `name`, `description` (required); optional `license`, `compatibility`, `metadata`, `allowed-tools`, `disable-model-invocation` |

**Discovery:**

- Global: `~/.pi/agent/skills/`, `~/.agents/skills/`
- Project (trusted): `.pi/skills/`, ancestor `.agents/skills/`
- Packages / settings / `--skill`
- Progressive disclosure: names/descriptions in system prompt; full body via `read` or `/skill:name`

#### Resource loader (composition hub)

`DefaultResourceLoader` (`packages/coding-agent/src/core/resource-loader.ts`):

```ts
interface ResourceLoader {
  getExtensions(): LoadExtensionsResult;
  getSkills(): { skills; diagnostics };
  getPrompts(): { prompts; diagnostics };
  getThemes(): { themes; diagnostics };
  getAgentsFiles(): { agentsFiles };
  getSystemPrompt(): string | undefined;
  getAppendSystemPrompt(): string[];
  extendResources(paths): void;
  reload(options?): Promise<void>;
}
```

**Dynamic contribution:** `resources_discover` event + `extendResources` after session start (`examples/extensions/dynamic-resources/`).

#### Themes

JSON color tokens; locations mirror skills/prompts (`docs/themes.md`). Packages declare `pi.themes` or `themes/`.

#### Context files

`AGENTS.md` / `CLAUDE.md` walked from cwd + global agent dir (`loadProjectContextFiles` in `resource-loader.ts`). **Not** gated by project trust (security.md). Also `.pi/SYSTEM.md` / `APPEND_SYSTEM.md` when trusted.

#### Prompt system customization (non-event)

Settings / CLI / SYSTEM.md; extensions can further rewrite via `before_agent_start.systemPrompt` chaining.

---

### 2.6 Packages (distribution unit)

| Item | Detail |
|------|--------|
| **Path** | `packages/coding-agent/src/core/package-manager.ts`, `package-manager-cli.ts` |
| **Docs** | `docs/packages.md` |
| **Language** | Manifest JSON + contained TS/MD/JSON resources |
| **CLI** | `pi install` / `remove` / `update` / `list` / `config` |

**Manifest** in `package.json`:

```json
{
  "keywords": ["pi-package"],
  "pi": {
    "extensions": ["./extensions"],
    "skills": ["./skills"],
    "prompts": ["./prompts"],
    "themes": ["./themes"],
    "video": "...",
    "image": "..."
  }
}
```

Convention dirs if no `pi` key: `extensions/`, `skills/`, `prompts/`, `themes/`.

**Sources:** `npm:@scope/pkg@version`, `git:host/path@ref`, local path/file.  
**Scopes:** user (`~/.pi/agent/{npm,git}/`), project (`.pi/{npm,git}/`), temporary (`-e` / `--extension`).  
**Filtering:** object form in settings with globs / `+path` / `-path` / empty arrays.  
**Enable/disable:** `pi config` TUI.  
**Precedence:** project settings > project auto > user settings > user auto > package (`resourcePrecedenceRank` in `package-manager.ts` L172–188).  
**Deps:** runtime deps in `dependencies`; core pi packages as `peerDependencies: "*"`; nested packages via `bundledDependencies` + `node_modules/` paths in manifest.

**Security note (docs):** packages = arbitrary code; review before install.

---

### 2.7 Providers / models

| Item | Detail |
|------|--------|
| **Path** | Registration: coding-agent `ExtensionAPI.registerProvider`; implementations: `packages/ai/src/providers/*` |
| **Docs** | `docs/providers.md`, `docs/custom-provider.md`, `docs/models.md` |
| **Language** | TypeScript (extension or models.json) |

**Three registration layers:**

1. **Built-in catalogs** in `pi-ai` (Anthropic, OpenAI, Google, Bedrock, …)
2. **`models.json`** — OpenAI-compatible / Anthropic / Google endpoints without code
3. **Extensions** — `pi.registerProvider(name, ProviderConfig)` or native `Provider` from `createProvider(...)`; `unregisterProvider`

**`ProviderConfig`** (`types.ts` ≈ L1414–1451): `baseUrl`, `apiKey`, `api`, `headers`, `authHeader`, `models`, `streamSimple`, `refreshModels`, `oauth.{login,refreshToken,getApiKey}`.

**Auth storage:** `~/.pi/agent/auth.json` (0600); OAuth via `/login`; env vars; key resolution `!command` / `$ENV` / literal (`docs/providers.md`).

**Composition:** `ModelRuntime` / `ModelRegistry` in coding-agent compose `models.json` **above** registered native providers. Streaming uses `pi-ai` API types (`anthropic-messages`, `openai-completions`, `openai-responses`, …). Overflow recovery can be helped by rewriting `message_end.errorMessage` (custom-provider.md).

**Examples:** `examples/extensions/custom-provider-anthropic/`, `custom-provider-gitlab-duo/`.

---

### 2.8 Config / trust / settings

| Item | Detail |
|------|--------|
| **Settings** | `~/.pi/agent/settings.json` + `.pi/settings.json` (`docs/settings.md`, `SettingsManager`) |
| **Trust store** | `~/.pi/agent/trust.json` (`trust-manager.ts`) |
| **Docs** | `docs/security.md`, settings § Project Trust |

**Project trust gate** (`TRUST_REQUIRING_PROJECT_CONFIG_RESOURCES` in `trust-manager.ts`):

- `.pi/settings.json`, `extensions`, `skills`, `prompts`, `themes`, `SYSTEM.md`, `APPEND_SYSTEM.md`
- Project `.agents/skills` in cwd/ancestors  
Bare `.pi` dir alone does **not** require trust.

**Resolution order:** extension `project_trust` (first yes/no wins) → nearest saved `trust.json` → `defaultProjectTrust` (`ask` \| `always` \| `never`). Non-interactive: no prompt; `--approve` / `--no-approve` override. `/trust` saves decision (restart needed for full effect).

**What trust is not:** not a sandbox; tools still have full FS/shell rights after trust.

**Other config surfaces:** `keybindings.json`, `models.json`, `auth.json`, `models-store.json` (cached catalogs), themes, packages array, skills/prompts/extensions path arrays in settings.

---

### 2.9 Install / update (CLI + self)

| Command | Behavior (from `package-manager-cli.ts` + `docs/packages.md`) |
|---------|----------------------------------------------------------------|
| `pi install <source> [-l]` | Persist to user or project settings; npm/git clone + `npm install` |
| `pi remove <source> [-l]` | Uninstall + settings |
| `pi list` | Configured packages |
| `pi update` | Self only by default |
| `pi update --self [--force]` | Reinstall CLI (npm/bun/binary-aware; Windows quarantine helpers) |
| `pi update --extensions` / `--all` / `--models` / named source | Packages and/or model catalogs |
| `pi config [-l]` | Enable/disable resources TUI |
| `pi -e` / `--extension` | Ephemeral package/extension for one run |

Version check: `https://pi.dev/api/latest-version` (disable via `PI_SKIP_VERSION_CHECK` / `--offline` / `PI_OFFLINE`). Install telemetry separate from update checks (`enableInstallTelemetry`).

**Composition:** On trusted project startup, missing project packages auto-install. Package resolution feeds `ResourceLoader` → extensions/skills/prompts/themes.

---

### 2.10 MCP

**Finding (verified):** **No MCP (Model Context Protocol) support** in this Pi tree.

- Repo-wide search under `.tmp-research-plugins/pi` for `\bmcp\b` / `Model Context Protocol`: **0 hits** (packages + docs).
- Tool extensibility is **in-process `registerTool`**, not MCP servers.
- Contrast: next-code already uses MCP as an out-of-process tool bus; Pi does not.

If “dynamic like Pi” includes third-party tools, Pi’s answer is **TS packages + registerTool**, not MCP.

---

### 2.11 SDK / embed / RPC (host for custom UIs)

Not a “package extension,” but a first-class **platform surface** for replacing the TUI:

| Path | Role |
|------|------|
| `docs/sdk.md` | `createAgentSession`, `createAgentSessionRuntime`, `DefaultResourceLoader`, `InteractiveMode`, `runPrintMode`, `runRpcMode` |
| `docs/rpc.md` | Subprocess JSON-RPC (`pi --mode rpc`) |
| `packages/coding-agent/src/index.ts` | Public exports |

Custom UIs embed the same ResourceLoader → extensions/tools/skills stack. Inline extensions: `extensionFactories` / named `InlineExtension`. Shared `EventBus` via `pi.events` for inter-extension messaging (`examples/extensions/event-bus.ts`).

---

### 2.12 Adjacent surfaces (still “dynamic”)

| Surface | Mechanism | Path / docs |
|---------|-----------|-------------|
| Shell aliases / `!` bash | User bash + `user_bash` hook | `docs/shell-aliases.md`, extensions |
| Compaction policy | Settings + `session_before_compact` | `docs/compaction.md` |
| Containerization | Gondolin / Docker / OpenShell extensions | `docs/containerization.md`, `examples/extensions/gondolin/`, `sandbox/` |
| Llama.cpp | Built-in extension under `src/extensions/llama/` | `docs/llama-cpp.md` |
| Sessions / tree / labels | SessionManager + `setLabel` / `setSessionName` | `docs/sessions.md`, `session-format.md` |
| Experimental server | `@earendil-works/pi-server` | `packages/server` |

---

## 3. Extension factory contract (summary)

```ts
// packages/coding-agent — docs/extensions.md + types.ts
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export default function (pi: ExtensionAPI) { /* sync or async */ }

// Locations (trusted project for .pi/*):
//   ~/.pi/agent/extensions/*.ts | */index.ts
//   .pi/extensions/*.ts | */index.ts
//   settings.packages / settings.extensions
//   CLI -e
```

**`ExtensionAPI` method groups:**

1. `on(event, handler)` — full lifecycle  
2. `registerTool`  
3. `registerCommand` / `registerShortcut` / `registerFlag` / `getFlag` / `getCommands`  
4. `registerMessageRenderer` / `registerEntryRenderer`  
5. `sendMessage` / `sendUserMessage` / `appendEntry`  
6. `setSessionName` / `getSessionName` / `setLabel`  
7. `exec`  
8. `getActiveTools` / `getAllTools` / `setActiveTools`  
9. `setModel` / `getThinkingLevel` / `setThinkingLevel`  
10. `registerProvider` / `unregisterProvider`  
11. `events` (EventBus)

Factory must not start long-lived resources; use `session_start` + `session_shutdown`. Async factories are awaited before `session_start` / provider flush (`docs/extensions.md`).

---

## 4. Mode matrix (where surfaces work)

| Mode | Extensions load? | Full `ctx.ui.custom`? | Notes |
|------|------------------|----------------------|-------|
| Interactive TUI | Yes | Yes | Full surface |
| RPC | Yes | Subset via RPC UI protocol | `rpc-demo.ts` |
| Print / JSON / `-p` | Yes (trust rules differ) | No (`hasUI` false) | Guard UI calls |
| SDK embed | Optional via ResourceLoader | Host-provided UI context | Build your own UI |

---

## 5. Implications for next-code platform layer

These implications are for designing next-code’s “dynamic like Pi” platform — **not** for copying Pi’s TS/jiti host.

### 5.1 Surface set to cover (product checklist)

To claim Pi-like dynamism, next-code’s package/platform ABI should eventually declare or host equivalents of:

1. **Lifecycle hooks** (trust, input, tool gate, compaction, session replace)  
2. **Slash** (commands + prompt templates + skill invokes)  
3. **Tools** (schema + execute + optional render hints)  
4. **Skills / prompts / themes** (declarative resources + progressive disclosure)  
5. **Packages** (install/update/filter/enable from npm/git/local)  
6. **Providers** (or a clear “models.json + register” story)  
7. **Config + project trust** before loading executable/project resources  
8. **UI customization** — Pi’s deepest surface; next-code should **not** port in-process TUI components into Face; map to Face/ACP hints, status, and optional external panes  

**Do not treat MCP as a Pi gap to fill from Pi** — Pi has no MCP; next-code already ahead here for out-of-process tools.

### 5.2 Hosting model difference (critical)

| Pi | Recommended next-code |
|----|------------------------|
| In-process TS via jiti, same privileges as CLI | Manifest + **argv/HTTP/stdio** runners (herdr-shaped); Face stays Rust |
| Extensions import `@earendil-works/pi-tui` | No Face plugin component host in v1; toast/status/ACP + optional sidecar TUI |
| `registerProvider` mutates process model registry | Prefer config + adapter process, or agent-side provider plugins with explicit trust |

Copy **surfaces and lifecycle semantics**, not the **in-process TypeScript ABI**.

### 5.3 Highest-value semantic borrows

1. **`project_trust` before project extensions** — Pi’s `trust.json` + `defaultProjectTrust` + extension override is a clear RCE-prevention pattern for executable project plugins.  
2. **Unified package resource kinds** — one install unit → extensions + skills + prompts + themes + filters (`pi` manifest). next-code’s Grok `plugin.json` / bundles should converge on one manifest that lists **all** Pi-like kinds.  
3. **Slash source taxonomy** — `extension | prompt | skill` clarifies autocomplete and precedence.  
4. **Tool gate as first-class hook** — `tool_call` → `{ block, reason }` maps cleanly onto existing next-code hooks.  
5. **`resources_discover`** — late binding of skill/prompt roots without restart (reload path).  
6. **Session-scoped lifecycle** — `session_shutdown` on `/new`/`/resume`/`fork`/`reload` prevents leaked watchers; any long-lived plugin process needs the same.  
7. **Progressive skills** — description always in context; body on demand (`/skill:` or read).  

### 5.4 Lowest-value / high-risk copies

- Full `ctx.ui.custom` + overlay game loops inside the agent process  
- jiti + peer-virtualized monorepo imports as the only author language  
- Provider `streamSimple` in-process (prefer external gateway or agent crate)  
- Assuming packages are “safe” because they came from npm  

### 5.5 Mapping sketch (Pi → next-code)

| Pi surface | next-code today | Platform gap |
|------------|-----------------|--------------|
| `pi.on` events | `next-code-hooks` (~28 events, multilang) | Align event names/semantics; trust hook |
| `registerCommand` / prompts | Builtins + ACP slash | Package-declared slash + markdown templates |
| `registerTool` | Rust tools + **MCP** | Package-declared tools (MCP and/or argv tool runner) |
| Skills | Bundle skills ingest | Package install + `/skill:` parity |
| Packages | Partial Face plugins | Unified install/update/filter/enable |
| Providers | Agent config | Optional package provider adapters |
| Trust | Weaker / different | Explicit project trust for executable resources |
| Custom TUI | Face fixed + ACP | Hints only; no Pi-style Component host |
| MCP | Present | Keep; do not wait on Pi |

### 5.6 Design slogan

**Pi defines the breadth of “what authors can customize.”**  
**Herdr/next-code hooks define “how (any language, out of process).”**  
**MCP is next-code’s out-of-process tool bus — a Pi non-feature, keep it.**

---

## 6. Primary citations (real files)

| Topic | Path |
|-------|------|
| ExtensionAPI + events | `packages/coding-agent/src/core/extensions/types.ts` |
| Loader / jiti / virtual modules | `packages/coding-agent/src/core/extensions/loader.ts` |
| Runner | `packages/coding-agent/src/core/extensions/runner.ts` |
| Resource loader | `packages/coding-agent/src/core/resource-loader.ts` |
| Package manager | `packages/coding-agent/src/core/package-manager.ts` |
| Package CLI | `packages/coding-agent/src/package-manager-cli.ts` |
| Skills | `packages/coding-agent/src/core/skills.ts` |
| Slash builtins | `packages/coding-agent/src/core/slash-commands.ts` |
| Trust | `packages/coding-agent/src/core/trust-manager.ts` |
| Docs hub | `packages/coding-agent/docs/extensions.md`, `packages.md`, `skills.md`, `tui.md`, `custom-provider.md`, `providers.md`, `sdk.md`, `security.md`, `settings.md`, `prompt-templates.md`, `themes.md`, `keybindings.md` |
| Examples index | `packages/coding-agent/examples/extensions/README.md` |
| Agent tool loop | `packages/agent/src/types.ts`, `agent-loop.ts` |
| Providers | `packages/ai/src/providers/*` |
| TUI package | `packages/tui/package.json` |
| Monorepo overview | root `README.md` |

---

## 7. Status

Research complete. No production code changed.  
For implementation sequencing aligned with herdr multilang ABI, see `docs/plans/PLAN-20260722-pi-full-custom-platform.md`.
