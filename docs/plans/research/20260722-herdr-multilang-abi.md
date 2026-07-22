# Research: herdr multilanguage plugin ABI

**Date:** 2026-07-22  
**Scope:** Deep research only — no production code.  
**Subject:** [ogulcancelik/herdr](https://github.com/ogulcancelik/herdr) plugin system across languages (manifest, argv, socket, marketplace).  
**Local clones:**

| Repo | Path | SHA |
|------|------|-----|
| herdr | `.tmp-research-plugins/herdr` | `ef85fa0` |
| herdr-plugin-examples | `.tmp-research-plugins/herdr-plugin-examples` | `18709cd` |

**Primary docs (upstream):** [Plugins](https://herdr.dev/docs/plugins/), [Socket API](https://herdr.dev/docs/socket-api/), [Marketplace](https://herdr.dev/docs/marketplace/), [CLI reference](https://herdr.dev/docs/cli-reference/).

---

## Summary

Herdr’s plugin v1 is a **language-agnostic out-of-process ABI**:

1. **Manifest** (`herdr-plugin.toml`) declares entrypoints as **argv arrays**.
2. **Host** validates, installs/links, injects env + context JSON, spawns processes, logs output.
3. **Plugin** calls back via **`HERDR_BIN_PATH` (CLI)** or **local NDJSON socket** (Unix socket / Windows named pipe).
4. **Marketplace** is discovery only: GitHub topic `herdr-plugin` → `herdr plugin install owner/repo[/subdir]`.

There is **no** required SDK, **no** in-process UI widgets, **no** sandboxed restricted command set, and **no** host-managed plugin storage API. Native non-terminal plugin UI and runtime action registration are explicitly **out of v1**.

next-code already has **stdio/HTTP lifecycle hooks** and **Grok-style content bundles** (`plugin.json` + skills/agents). What is missing for herdr-parity is the **executable workflow package** layer: argv manifests, installable actions/panes/events, host CLI/socket callback contract, and (optionally) a topic-based marketplace for those packages.

---

## Architecture diagram

```text
┌─────────────────────────────────────────────────────────────┐
│ herdr host (Rust)                                           │
│  · plugins.json registry                                    │
│  · manifest validate (min_herdr_version, platforms, ids)    │
│  · keybindings → plugin.action.invoke                       │
│  · events → [[events]] argv spawn                           │
│  · plugin.pane.open → PTY / overlay / popup                 │
└───────────────┬──────────────────────────▲──────────────────┘
                │ spawn argv + env         │ CLI / NDJSON
                ▼                          │
┌──────────────────────────────────────────┴──────────────────┐
│ plugin process (any language)                               │
│  cwd = HERDR_PLUGIN_ROOT                                    │
│  read HERDR_PLUGIN_CONTEXT_JSON / EVENT_JSON                │
│  call `$HERDR_BIN_PATH …`  or  connect HERDR_SOCKET_PATH    │
└─────────────────────────────────────────────────────────────┘
```

---

## Layer 1 — Manifest (`herdr-plugin.toml`)

**Contract location:** plugin root (or path passed to `plugin.link`).  
**Parser:** `src/app/api/plugins/manifest.rs` → `InstalledPluginInfo` in `src/api/schema/plugins.rs`.

### Required top-level fields

| Field | Notes |
|-------|--------|
| `id` | ASCII letters/digits/`.`/`:`/`_`/`-`; max length enforced |
| `name`, `version` | Non-empty trimmed |
| `min_herdr_version` | Semver; refuse link/install if newer than running binary |

### Optional / sections

| Section | Role |
|---------|------|
| `platforms` | `linux` / `macos` / `windows`; omit → link warning |
| `[[build]]` | Install-time argv only (`plugin install`, not `plugin link`) |
| `[[startup]]` | One-shot after session restore + socket ready |
| `[[actions]]` | Invokable workflows (`contexts`, `command`) |
| `[[events]]` | `on = "<event.name>"` + `command` |
| `[[panes]]` | Terminal UI entrypoints (`placement`, optional popup size) |
| `[[link_handlers]]` | URL regex → action id |

### Critical ABI rule: argv, not shell

`command = ["node", "index.js"]` is an **argv array**. Herdr does **not** run a shell (except Windows `.cmd`/`.bat` shim via `cmd /c` in `src/plugin_command.rs`). Language-specific behavior lives inside the program.

Local action/pane/link-handler ids **must not contain dots**; host qualifies actions as `plugin.id.action`.

### Example shape (from docs + schema)

```toml
id = "example.layout"
name = "Layout"
version = "0.1.0"
min_herdr_version = "0.7.0"
platforms = ["linux", "macos", "windows"]

[[build]]
command = ["npm", "ci"]

[[startup]]
command = ["node", "dist/restore.js"]

[[actions]]
id = "apply"
title = "Apply layout"
contexts = ["workspace"]
command = ["node", "dist/apply.js"]

[[events]]
on = "worktree.created"
command = ["node", "dist/on-worktree.js"]

[[panes]]
id = "board"
title = "Board"
placement = "overlay"
command = ["node", "dist/board.js"]

[[link_handlers]]
id = "github-issue"
title = "Open GitHub issue"
pattern = "^https://github\\.com/[^/]+/[^/]+/(issues|pull)/[0-9]+$"
action = "apply"
```

---

## Layer 2 — Process spawn + environment

**Runtime:** `src/app/api/plugins/runtime.rs` (`start_plugin_command`).  
**Panes:** `src/app/api/plugins/panes.rs` (overlay / popup / split / tab / zoomed).

### Injected env (language-neutral)

| Variable | Purpose |
|----------|---------|
| `HERDR_SOCKET_PATH` | Local control socket / named pipe |
| `HERDR_BIN_PATH` | Absolute path to running herdr binary (preferred portable callback) |
| `HERDR_ENV=1` | Marker that process runs under herdr |
| `HERDR_PLUGIN_ID` | Installed plugin id |
| `HERDR_PLUGIN_ROOT` | Plugin cwd / source checkout |
| `HERDR_PLUGIN_CONFIG_DIR` | User config (e.g. `.env`); host creates, does not manage schema |
| `HERDR_PLUGIN_STATE_DIR` | Durable runtime state path |
| `HERDR_PLUGIN_CONTEXT_JSON` | Invocation context object |
| `HERDR_WORKSPACE_ID` / `HERDR_TAB_ID` / `HERDR_PANE_ID` | When available |
| `HERDR_PLUGIN_ACTION_ID` | Action invocations |
| `HERDR_PLUGIN_EVENT` / `HERDR_PLUGIN_EVENT_JSON` | Event + startup hooks |
| `HERDR_PLUGIN_ENTRYPOINT_ID` | Pane entrypoints |
| `HERDR_PLUGIN_CLICKED_URL` / `HERDR_PLUGIN_LINK_HANDLER_ID` | Link handlers |

**Limits:** max 32 concurrent plugin commands; stdout/stderr capped (~64 KiB). Failures are logged; startup failures do not stop the server.

**cwd:** always `HERDR_PLUGIN_ROOT`.

**Callback preference:** docs insist on `HERDR_BIN_PATH` for cross-platform (Unix socket vs Windows named pipe). Raw socket is for long-lived subscribers / custom clients.

---

## Layer 3 — Socket API (host control plane)

**Transport:** newline-delimited JSON. Unix domain socket on Unix; named pipe on Windows (`socket-api.mdx`).

**Stacking (same surface):**

| Layer | Use |
|-------|-----|
| Agent skill | Teach agents inside panes |
| CLI wrappers | `herdr workspace|tab|pane|plugin|wait …` |
| Raw socket | Request/response + `events.subscribe` |

### Plugin-specific methods

`plugin.link` · `plugin.list` · `plugin.unlink` · `plugin.enable` · `plugin.disable` · `plugin.action.list` · `plugin.action.invoke` · `plugin.log.list` · `plugin.pane.open` · `plugin.pane.focus` · `plugin.pane.close`

Registry: `plugins.json` beside session state; CLI can mutate registry **even when server is down**; startup reloads manifests.

### Plugin event hooks vs full event bus

Manifest `[[events]] on` accepts a **narrow allow-list** (`plugin_hook_event_names()` in `src/api/schema/events.rs`), intentionally excluding high-volume events like `pane.output_changed`.

**Hookable today (representative):** workspace.* · worktree.created/opened/removed · tab.create/close/rename/move/focus · pane.create/close/focus/move/exited · pane.agent_detected · pane.agent_status_changed.

Unknown event names → **warning**, not hard fail, on link/list.

Long-lived plugins can also `events.subscribe` on the socket (community Rust helper `herdr-plugin` crate’s `SocketRuntime` wraps this). That is optional convenience, **not** part of the host ABI.

---

## Layer 4 — Marketplace

| Fact | Detail |
|------|--------|
| Index | [herdr.dev/plugins](https://herdr.dev/plugins/) |
| Signal | Public GitHub repos with topic **`herdr-plugin`** |
| Refresh | ~30 minutes |
| Review | **None** — discovery, not curation |
| Install | `herdr plugin install owner/repo[/subdir...]` via `git` + optional `[[build]]` |
| Listing fields | GitHub metadata only; **does not parse** `herdr-plugin.toml` in v1 |

Trust model: interactive install preview; user runs as themselves; **no sandbox**. Docs: “Herdr does not review or sandbox what a plugin does.”

Official cookbook: `ogulcancelik/herdr-plugin-examples` (examples, not maintained product plugins).

---

## How languages differ (same ABI, different argv)

Herdr does **not** special-case languages. Differences are only in **which argv** the author puts in the manifest and whether they use CLI vs socket helpers.

| Language | Official / cookbook pattern | Manifest `command` | Callback style | Notes |
|----------|----------------------------|--------------------|----------------|-------|
| **Bash** | `github-link-preview` | `["bash", "open.sh"]` / `["bash", "preview.sh"]` | `$HERDR_BIN_PATH plugin pane open …` | Link handler + split pane; needs `gh` |
| **Node** | `agent-telegram-notify` | `["node", "notify.mjs"]` | Env JSON only (Telegram HTTP); optional herdr CLI | Event hook `pane.agent_status_changed`; config in `HERDR_PLUGIN_CONFIG_DIR/.env` |
| **Lua** | `dev-layout-bootstrap` | `["lua", "setup.lua"]` | `os.execute(HERDR_BIN_PATH …)` + parse JSON pane ids | Layout orchestration via CLI |
| **Rust** | `rust-release-check` | `[["build"] cargo…]` then `["./target/release/…"]` | Read `HERDR_PLUGIN_CONTEXT_JSON`; may shell out to `git` | Install-time build; binary argv |
| **Python** | Docs claim support; **no first-party example** in cookbook | Would be `["python", "main.py"]` or `["uv", "run", …]` | Same env + CLI/socket | Same ABI as Bash/Node |
| **Community Rust framework** | [Newt6611/herdr-plugin-rust](https://github.com/Newt6611/herdr-plugin-rust) (`herdr-plugin` crate) | Still argv binary in manifest | Typed `OneShotRuntime` / experimental `SocketRuntime` | **Host-optional** convenience; not required |

### Concrete citations (examples clone)

- Bash open: `herdr-plugin-examples/github-link-preview/open.sh` — `exec "$herdr_bin" plugin pane open …`
- Node notify: `…/agent-telegram-notify/notify.mjs` — `JSON.parse(process.env.HERDR_PLUGIN_EVENT_JSON)`
- Lua layout: `…/dev-layout-bootstrap/setup.lua` — `herdr_bin = os.getenv("HERDR_BIN_PATH")`
- Rust check: `…/rust-release-check/src/main.rs` — `env::var("HERDR_PLUGIN_CONTEXT_JSON")`

**Smoke fixture** (host tests): `herdr/tests/fixtures/plugin-smoke/herdr-plugin.toml` uses argv `["herdr", "workspace", "list"]` — proving the host binary itself is a valid plugin command.

---

## What herdr deliberately does NOT do

| Non-goal (v1) | Evidence |
|---------------|----------|
| In-process / native non-terminal plugin UI | Docs: “Runtime action registration and **native non-terminal plugin UI** are not part of plugin v1.” Discussion #1345: plugins stuck with CLI + auto-sync |
| Separate restricted plugin SDK / allow-list of host commands | “Entire Herdr CLI is the plugin API” |
| Sandbox / capability attenuation | Trust section — runs as user |
| Host-managed plugin KV / DB API | Config/state dirs are path discovery only |
| Runtime registration of actions/panes | Manifest-only; no `plugin.action.register` |
| Runtime argv pane creation outside manifest entrypoints | Socket docs |
| Supervised long-running plugin daemons as first-class | Startup hooks are one-shot; daemons are DIY via socket subscribe |
| Marketplace curation / manifest indexing | Topic-only; no `herdr-plugin.toml` parse in index |
| Language runtimes shipped by herdr | Authors document `node`/`cargo`/`lua`/`gh` requirements; host reports build failures |

**UI surface that *does* exist:** terminal panes only (overlay, popup, split, tab, zoomed) — still just argv in a PTY.

**Integrations ≠ plugins:** agent hooks (pi/claude/codex/opencode/…) report state into herdr over the socket; they are a separate install path (`herdr integration install`). Plugins are shareable workflow packages on top of the same CLI/socket.

---

## Language-agnostic ABI pattern next-code could copy

Extracted pattern (host-owned vs plugin-owned):

### Host owns

1. **Package identity** — stable id, version, `min_host_version`, platforms.
2. **Declarative entrypoints** — actions / events / panes / link handlers / startup / build as **argv**.
3. **Lifecycle** — link/install/enable/disable; global registry; re-read manifests on start.
4. **Invocation context** — structured JSON + a few scalar env ids.
5. **Control plane** — one portable host binary path + optional local NDJSON RPC.
6. **Observability** — command logs (status, exit, capped stdout/stderr).
7. **Discovery** — optional unreviewed index (e.g. GitHub topic) layered on git install.

### Plugin owns

1. Language, deps, build, files under package root.
2. Config/state schemas under host-provided directories.
3. Whether to use CLI, raw socket, or neither (pure side effects).

### Minimal “portable plugin” recipe

```text
package/
  <host>-plugin.toml     # or next-code-plugin.toml
  bin-or-script…         # any argv
```

```toml
id = "org.example.thing"
name = "Thing"
version = "0.1.0"
min_next_code_version = "…"   # rename of min_herdr_version
platforms = ["linux", "macos", "windows"]

[[actions]]
id = "run"
title = "Run"
command = ["python", "main.py"]   # or node / bash / ./target/release/…
```

```text
Host injects:
  NEXT_CODE_BIN_PATH | NEXT_CODE_SOCKET | NEXT_CODE_PLUGIN_* | CONTEXT_JSON
Plugin calls back with BIN_PATH for portability.
```

That is the whole multilang story: **manifest + argv + env + host CLI/socket**. No WASM, no JS VM, no language plugins in the host.

---

## next-code implications (hooks already exist — what’s missing)

### What next-code already has

| Capability | Where | Shape |
|------------|-------|-------|
| Lifecycle hooks | `docs/HOOKS.md`, crate `next-code-hooks` | `command` / `http` / `plugin` / `agent` handlers in `hooks.toml` |
| Stdio protocol | `execute.rs` | JSON **stdin** → JSON stdout; exit `0` allow / `2` deny |
| HTTP hooks | same | POST HookInput → HookOutput |
| Content-bundle plugins | `docs/plugins.md` | `plugin.json` + skills/agents/hooks dirs under `~/.next-code/plugins` |
| Face UI | `/plugins`, `/hooks` | List/enable; marketplace brand-hidden |
| Explicit non-goal | `HOOKS.md` | OpenCode-like **in-process** TS hooks deferred |

### Gap vs herdr-parity (executable multilang packages)

| Herdr piece | next-code today | Gap |
|-------------|-----------------|-----|
| `herdr-plugin.toml` argv package | Bundle `plugin.json` (skills/MCP/hooks JSON) | No first-class **executable workflow manifest** with actions/panes/events |
| `plugin.action.invoke` + keybindings | Hook events only; Face plugins enable skills | No shareable **user-invokable actions** as argv packages |
| Managed plugin **terminal panes** | N/A (Face is the UI) | Different product; optional “open tool in pane” not analogous |
| `HERDR_BIN_PATH` host callback | Hooks get env + stdin; no stable “call next-code as API” binary contract for plugins | Missing portable **host CLI/RPC** for package authors |
| Local NDJSON socket as extension bus | ACP/daemon for Face; hooks are push-only | No documented **plugin→host** control socket for third-party packages |
| GitHub topic marketplace for **exec** plugins | Marketplace stub / brand-hidden; git install for bundles | Different packaging model; no `herdr-plugin`-style topic for argv packages |
| Install-time `[[build]]` | Git install for bundles | No cargo/npm build step tied to plugin install |
| Config/state dirs per plugin | Partial via plugin data paths in Grok docs | Not the same documented ABI |
| Gate hooks (deny tool) | **Stronger** than herdr (exit 2 / HTTP decision) | Keep — herdr plugins are mostly observers/orchestrators, not pre-tool gates |

### Recommended copy strategy (research recommendation only)

1. **Do not** replace `next-code-hooks` with herdr’s fire-and-forget event model — next-code’s stdin/HTTP **gating** is a product advantage.
2. **Do** consider a parallel package type (name TBD) that mirrors herdr’s ABI for **multilang automation packages**:
   - TOML manifest + argv
   - `NEXT_CODE_BIN_PATH` + optional local socket/ACP methods for session control
   - actions + event hooks (map herdr-style host events ↔ existing `HookEvent` where it makes sense)
   - install from git + optional build
3. Keep Face **content bundles** for skills/agents/MCP; treat herdr-style packages as **orchestration/automation**, not prompt content.
4. Marketplace: if added, copy herdr’s cheap discovery (topic + git install), not a curated store — but only after the argv ABI exists.
5. Explicitly stay away from herdr’s non-goals that next-code already rejected: in-process UI SDK, sandboxed QuickJS (already removed), runtime action registration.

### Parity checklist (for a future plan, not this research)

- [ ] Manifest schema + validate `min_next_code_version`
- [ ] Registry (link/install/enable) + command logs
- [ ] Env injection + context JSON
- [ ] `plugin.action.invoke` equivalent (CLI + ACP)
- [ ] Event fan-out to manifest `[[events]]` (observer) **without** removing stdin gate hooks
- [ ] Documented host binary / socket callback for authors
- [ ] Optional GitHub-topic marketplace
- [ ] Cookbook: Bash / Python / Node / Rust one-pagers (prove language neutrality)

---

## Evidence index

| Claim | Source |
|-------|--------|
| Plugins = argv packages, any language | `herdr/website/src/content/docs/plugins.mdx` L6–34 |
| No native non-terminal UI / no runtime action reg | same L32–34; `socket-api.mdx` Plugin APIs |
| Manifest schema | `src/app/api/plugins/manifest.rs`, `src/api/schema/plugins.rs` |
| Spawn + env | `src/app/api/plugins/runtime.rs` |
| Windows argv / PATHEXT | `src/plugin_command.rs` |
| Hook event allow-list | `src/api/schema/events.rs` `PLUGIN_HOOK_EVENT_KINDS` |
| Marketplace topic | `website/src/content/docs/marketplace.mdx` |
| Socket NDJSON + plugin methods | `website/src/content/docs/socket-api.mdx` |
| Language examples | `.tmp-research-plugins/herdr-plugin-examples/*` |
| Community Rust helper (optional) | github.com/Newt6611/herdr-plugin-rust |
| next-code hooks stdio/HTTP | `docs/HOOKS.md`, `crates/next-code-hooks/src/execute.rs` |
| next-code bundle plugins | `docs/plugins.md` |

---

## Status

Research complete. No production code changed.  
Local clones ready under `.tmp-research-plugins/` for follow-up design work.
