# Deepen — Host callback `NEXT_CODE_BIN_PATH` (2026-07-22)

**ID:** D6 · **Priority:** P1 · Phase 1–2  
**Status:** **FROZEN design contract** (docs only — not implement approval)  
**Parent:** [`../20260722-herdr-multilang-abi.md`](../20260722-herdr-multilang-abi.md) (`HERDR_BIN_PATH`)  
**Master:** [`../../PLAN-20260722-pi-full-custom-platform.md`](../../PLAN-20260722-pi-full-custom-platform.md)  
**Siblings:** [`20260722-hooks-cookbook-layout.md`](./20260722-hooks-cookbook-layout.md), [`20260722-plugin-manifest-abi-v1.md`](./20260722-plugin-manifest-abi-v1.md), [`20260722-argv-plugin-security.md`](./20260722-argv-plugin-security.md)

---

## Summary (read first)

Hooks and future package argv runners need a **portable way to call back into next-code** without depending on Unix sockets vs Windows named pipes. Herdr solved this with `HERDR_BIN_PATH` = absolute path of the running binary. next-code freezes the same pattern as `NEXT_CODE_BIN_PATH`.

**v1 ships BIN_PATH + a tiny documented CLI surface.** Socket/ACP control plane is Phase 2+ optional.

---

## Problem

Today (`crates/next-code-hooks/src/execute.rs` → `build_command_env`):

| Injected today | Purpose |
|----------------|---------|
| `NEXT_CODE_HOOK_EVENT` | Event name |
| `NEXT_CODE_HOOK_SESSION_ID` | Session id |
| `NEXT_CODE_HOOK_CWD` | Working directory |
| `NEXT_CODE_HOOK_TOOL_NAME` / `TOOL_INPUT` | Gate helpers (v1 compat) |
| `NEXT_CODE_HOOKS_DISABLED=1` | Recursion guard |

**Missing:** a stable absolute path to *this* next-code binary so a Bash/Python/Node hook can run `next-code …` without guessing `PATH`, launcher shims, or Face vs daemon channel.

Package authors cannot implement herdr-style “open pane / notify host / list state” workflows portably.

---

## Frozen name

```text
NEXT_CODE_BIN_PATH
```

| Property | Contract |
|----------|----------|
| Type | Absolute filesystem path string |
| Value | `std::env::current_exe()` of the **process that spawns** the hook/plugin child (agent/daemon worker), UTF-8 display form |
| Fallback if `current_exe` fails | **Omit** the variable (do not invent `"next-code"`); log once at warn |
| Override | **Forbidden** for authors to require a user-set override; host may honor pre-set only if already absolute and same-binary policy later — v1: host **always overwrites** on spawn |

Optional later (not v1):

| Name | Role |
|------|------|
| `NEXT_CODE_SOCKET` / ACP | Long-lived control plane (herdr `HERDR_SOCKET_PATH`) |
| `NEXT_CODE_ENV=1` | Marker “running under next-code host” (herdr `HERDR_ENV=1`) — **include in v1** as cheap sibling |
| `NEXT_CODE_PLUGIN_*` | Package context when spawn provenance is `plugin:<id>` (Phase 2) |

### Frozen companion env (v1, spawn of command hooks)

```text
NEXT_CODE_BIN_PATH=<absolute path or absent>
NEXT_CODE_ENV=1
# existing:
NEXT_CODE_HOOKS_DISABLED=1
NEXT_CODE_HOOK_EVENT=...
NEXT_CODE_HOOK_SESSION_ID=...
NEXT_CODE_HOOK_CWD=...
```

When provenance is a package (Phase 2+), also inject:

```text
NEXT_CODE_PLUGIN_ID=<manifest id>
NEXT_CODE_PLUGIN_ROOT=<absolute plugin root>
```

---

## When injected

| Context | Inject `NEXT_CODE_BIN_PATH`? | Notes |
|---------|------------------------------|-------|
| Hook `type=command` argv/shell spawn | **Yes** (Phase 1 if in slice) | Primary consumer |
| Hook `type=http` | No | Use HTTP; document “no BIN_PATH” |
| Hook `type=plugin` (path runner) | **Yes** | Same child env builder |
| Future package `[[hooks]]` / `[[slash]]` argv | **Yes** | Same as herdr plugin commands |
| Future `[[tools]]` argv | **Yes** | After tools ABI lands |
| User interactive shell / CI bare `next-code` | Not required | Authors use PATH normally |
| Nested callback child (hook → BIN_PATH → next-code) | See recursion | Child sees `NEXT_CODE_HOOKS_DISABLED=1` from parent env inherit unless stripped |

---

## Herdr / research citations (verified under `.tmp-research-plugins`)

### Docs

1. **Prefer BIN_PATH over raw socket** — `.tmp-research-plugins/herdr/docs/next/website/src/content/docs/plugins.mdx` L27–30, L278–284: plugins should call Herdr through `HERDR_BIN_PATH` for Unix socket vs Windows named-pipe portability.
2. **Injected env list** — same file L254–262: `HERDR_SOCKET_PATH`, `HERDR_BIN_PATH`, `HERDR_ENV=1`, `HERDR_PLUGIN_ID`, `HERDR_PLUGIN_ROOT`, …
3. **Research summary** — `docs/plans/research/20260722-herdr-multilang-abi.md` Layer 2 table + “Callback preference.”

### Runtime proof

4. **Set from `current_exe`** — `.tmp-research-plugins/herdr/src/app/api/plugins/runtime.rs` L49–54:

```rust
if let Ok(current_exe) = std::env::current_exe() {
    env.push((
        "HERDR_BIN_PATH".to_string(),
        current_exe.display().to_string(),
    ));
}
```

Same pattern in `panes.rs` (~L255).

### Cookbook examples

5. **Bash** — `.tmp-research-plugins/herdr-plugin-examples/github-link-preview/open.sh` L4 + L12: `herdr_bin="${HERDR_BIN_PATH:-herdr}"` then `exec "$herdr_bin" plugin pane open …`
6. **Lua** — `…/dev-layout-bootstrap/setup.lua` L1: `local herdr_bin = os.getenv("HERDR_BIN_PATH") or "herdr"`
7. **Node** — `…/agent-telegram-notify/lib.mjs` ~L94: prefers `process.env.HERDR_BIN_PATH` then `"herdr"`
8. **Smoke** — herdr fixture argv may be `["herdr", "workspace", "list"]` proving host binary is a valid plugin command (`20260722-herdr-multilang-abi.md`).

### next-code today

9. **Hook env without BIN_PATH** — `crates/next-code-hooks/src/execute.rs` `build_command_env` L296–335; docs `docs/HOOKS.md` “Environment” section.
10. **Terminal launch strips herdr env** — `crates/next-code-terminal-launch/src/lib.rs` lists `"HERDR_BIN_PATH"` among vars to clear when launching terminals (do **not** clear `NEXT_CODE_BIN_PATH` for hook children; only for unrelated terminal handoff if product requires isolation).

---

## Frozen CLI surface for callbacks (v1 allowlist)

Herdr exposes “entire CLI is the plugin API.” next-code **does not** freeze that breadth for v1 (security + product scope). Freeze a **small allowlist** of verbs that hooks/packages may call via `$NEXT_CODE_BIN_PATH`.

### Allowlisted (v1)

| Verb | Purpose | Stdout contract |
|------|---------|-----------------|
| `hooks list [--event E] [--json]` | Introspect registry after reload | Human or JSON |
| `hooks test <Event> [--execute]` | Dry-run / execute one event | Existing CLI |
| `hooks metrics` | Observability | Existing CLI |
| `version` / `--version` | Prove binary identity | Semver string |

### Explicitly deferred (not allowlisted as “stable callback API” in docs)

| Verb | Why deferred |
|------|--------------|
| Arbitrary `exec` / tool invocation | Re-enter agent loop / RCE amplifier |
| Session mutate / prompt inject | Face/ACP ownership; needs Phase 2 design |
| Plugin install/enable | Prefer Face ACP + trust gate |
| New `hook-callback notify` mega-RPC | Prefer reuse `hooks …` until need proven |

### Documentation rule

HOOKS.md / package ABI must say:

> Callbacks via `NEXT_CODE_BIN_PATH` are supported only for the **allowlisted** subcommands above in ABI v1. Other CLI entrypoints may exist but are **unstable** for plugins until listed here.

### Open Q — exact subcommand set

**Resolved for v1:** allowlist = `hooks list|test|metrics` + `version`. Expand only by deepening this file + changelog row — not silently.

Cookbook demo callback: `hooks list --json` (log/notify style proof), **not** a new binary verb.

---

## Process contract

1. **Path absolute + executable** for the OS (Windows: path may include `.exe`; no shell wrapping required to invoke).
2. **cwd:** hooks keep today’s rule (handler `cwd` > `HookInput.cwd`). Packages (Phase 2) use plugin root as cwd like herdr — BIN_PATH still absolute so cwd does not matter for locating the binary.
3. **Args:** pass ids via argv/env JSON; do not assume relative paths to host state.
4. **Timeout:** callback must not hang the parent hook past `[settings].timeout_secs` / handler timeout. Parent spawn already times out; callback is part of that wall clock.
5. **Recursion:** nested next-code from a hook **must** honor `NEXT_CODE_HOOKS_DISABLED=1` (already set). Callbacks that only run `hooks list` are read-only and safe.
6. **Failure:** non-zero exit from callback is the **hook script’s** problem; host does not special-case. Cookbook shows checking exit status.
7. **No PATH dependency:** scripts should use `"$NEXT_CODE_BIN_PATH"` first; fallback to `next-code` only for local interactive debug (herdr pattern).

### Pseudocode (host — when implementing)

```rust
// in build_command_env / package spawn env
env.insert("NEXT_CODE_ENV".into(), "1".into());
if let Ok(exe) = std::env::current_exe() {
    env.insert("NEXT_CODE_BIN_PATH".into(), exe.display().to_string());
}
```

### Pseudocode (author — Bash)

```bash
bin="${NEXT_CODE_BIN_PATH:-next-code}"
"$bin" hooks list --json >/tmp/hooks.json || true
```

### Pseudocode (author — Node)

```js
import { spawnSync } from "node:child_process";
const bin = process.env.NEXT_CODE_BIN_PATH ?? "next-code";
const r = spawnSync(bin, ["hooks", "list", "--json"], { encoding: "utf8" });
```

### Pseudocode (author — Python)

```python
import os, subprocess
bin = os.environ.get("NEXT_CODE_BIN_PATH", "next-code")
subprocess.run([bin, "hooks", "list", "--json"], check=False)
```

---

## Frozen JSON schemas (callback-adjacent)

Hooks already speak stdin/stdout JSON (`docs/HOOKS.md`). BIN_PATH does not change HookInput. Freeze the **identity** object a future `hooks list --json` machine consumer can rely on when verifying callback binary:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "next-code://abi/v1/bin-path-identity.json",
  "title": "NextCodeBinPathIdentity",
  "type": "object",
  "required": ["bin_path", "version"],
  "additionalProperties": false,
  "properties": {
    "bin_path": {
      "type": "string",
      "description": "Absolute path equal to NEXT_CODE_BIN_PATH when set"
    },
    "version": {
      "type": "string",
      "description": "Semver from `next-code --version`"
    },
    "env_marker": {
      "type": "string",
      "const": "1",
      "description": "Value of NEXT_CODE_ENV when under host spawn"
    }
  }
}
```

Acceptance: cookbook or unit test may write this object by combining env + `--version` stdout — no new CLI required in Phase 1.

---

## Non-goals (v1)

- Full NDJSON control plane parity with herdr socket.
- Letting plugins re-enter the agent turn / inject prompts via BIN_PATH.
- Shipping `NEXT_CODE_SOCKET` as required.
- Guaranteeing BIN_PATH equals the Face UI process (may be daemon worker exe — document that).
- Windows `cmd /c` string expansion as the *primary* ABI (prefer argv arrays in package manifests; shell hooks remain for `hooks.toml` command strings today).

---

## Security notes (pointer)

Trust gate still applies before project executables run ([`20260722-trust-gate-design.md`](./20260722-trust-gate-design.md)). BIN_PATH amplifies capability of an already-trusted script (full CLI allowlist above). See [`20260722-argv-plugin-security.md`](./20260722-argv-plugin-security.md).

Do **not** document “call any next-code subcommand” as supported.

---

## Acceptance tests (design — implement later)

| ID | Scenario | Pass |
|----|----------|------|
| BP-01 | Spawn command hook; child env contains `NEXT_CODE_BIN_PATH` absolute path | Path exists; `metadata.is_file()` |
| BP-02 | Same spawn sets `NEXT_CODE_ENV=1` | Exact |
| BP-03 | `current_exe` failure → var absent; hook still runs | No crash |
| BP-04 | Hook script runs `"$NEXT_CODE_BIN_PATH" --version` within timeout | Exit 0; stdout non-empty |
| BP-05 | Hook script runs `"$NEXT_CODE_BIN_PATH" hooks list --json` | Exit 0; valid JSON array/object |
| BP-06 | Nested `next-code` from hook does not re-fire hooks | `NEXT_CODE_HOOKS_DISABLED` honored |
| BP-07 | HTTP hook does not require BIN_PATH | Documented; no regression |
| BP-08 | Cookbook (Bash/Python/Node) each performs one list callback | README matrix green |
| BP-09 | Windows: path with spaces works when quoted | SpawnSync / argv form |
| BP-10 | Docs: HOOKS.md documents env + allowlist | Link check |

Suggested unit location (when coding): `crates/next-code-hooks` env builder tests + cookbook smoke under `examples/hooks-cookbook-pretool/`.

---

## Implement checklist (when coding — not this doc)

1. [ ] Extend `build_command_env` (+ plugin path spawn) with `NEXT_CODE_BIN_PATH` / `NEXT_CODE_ENV`.
2. [ ] Document in `docs/HOOKS.md` + package ABI.
3. [ ] Cookbook one callback using BIN_PATH (`hooks list --json`).
4. [ ] Tests BP-01…BP-06.
5. [ ] Do **not** add socket env in same PR unless Phase 2 ticket open.

---

## Exit criteria

- [ ] Env set on command-hook spawn.
- [ ] Cookbook shows one callback using BIN_PATH.
- [ ] Documented in HOOKS.md / package ABI when landed.
- [ ] Allowlist frozen in this file; no “entire CLI is API” claim for next-code v1.

---

## Open questions

| Q | Resolution |
|---|------------|
| Exact subcommand set? | **Frozen:** `hooks list\|test\|metrics` + `version` |
| Face binary vs daemon exe? | **Daemon/agent spawn process** `current_exe`; document asymmetry |
| Need `hook-callback` verb? | **No for v1** — reuse hooks CLI |
| Inject on HTTP hooks? | **No** |

---

## Status

**Docs contract frozen.** Waiting for master / readiness **go ahead** before production Rust.
