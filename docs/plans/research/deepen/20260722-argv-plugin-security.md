# Deepen — Argv plugin security beyond trust (2026-07-22)

**ID:** D13 · **Priority:** P2 · Phase 1–2 hardening  
**Status:** Design contract (docs only — not implement approval)  
**Parent:** Trust gate + herdr Windows argv notes  
**Related:** [`20260722-trust-gate-design.md`](./20260722-trust-gate-design.md) (D1), [`20260722-tools-abi-v1.md`](./20260722-tools-abi-v1.md) (D11), [`20260722-host-callback-bin-path.md`](./20260722-host-callback-bin-path.md) (D6)

---

## Summary (read first)

Trusted project still means **user-approved RCE** in that tree. Trust ≠ sandbox. Additional controls reduce blast radius and foot-guns for hooks, package argv, and future `[[tools]]`.

Phase 1 can document + partially enforce; Phase 2–3 must enforce before advertising argv tools broadly (D11).

---

## Threat model (short)

| Attacker / foot-gun | Example |
|---------------------|---------|
| Malicious repo after user clicks Trust | `hooks.toml` argv → exfil / wipe |
| Path escape in manifest | `argv = ["./bin/x", "../../evil"]` |
| Shell-string injection | `command = "bash -c $USER_INPUT"` |
| Unbounded concurrency | Fork bomb via hook spam |
| Confused deputy via BIN_PATH | Hook calls full CLI; see D6 allowlist |
| MCP vs project trust conflation | Silent allow of stdio servers |

**Non-goal:** claiming trusted argv is “safe untrusted code.”

---

## Verified hooks execute surface (reuse, do not fork)

| Concern | Today | Citation |
|---------|-------|----------|
| Command spawn | `execute_command_hook` | `crates/next-code-hooks/src/execute.rs` |
| Timeout | `timeout_secs` on handler + settings default | `config.rs` `CommandHandlerConfig`, `HookSettings` |
| Kill / signal | Unix signal mapping; timeout path | `execute.rs` (`ExecuteError::Timeout`, killed-by-signal) |
| HTTP hooks | Separate timeout client | `execute_http_hook` |
| Plugin-type handler | `execute_plugin_hook` + path | `execute.rs` — legacy plugin path; package ABI must not revive in-process |
| String `command` | `CommandHandlerConfig` still accepts shell-ish strings | Compatibility for user `hooks.toml` |
| Face trust ACP | `HooksAction::Trust` → Unsupported | `src/cli/face_plugins.rs` — “always loaded” today (D1 changes product) |

**Package ABI rule:** new `runner = { argv = [...] }` only (arrays). Legacy string commands may remain for user TOML; **reject shell-string in package manifests** (D8/D11).

Herdr cite: argv-not-shell + env spawn — [`../20260722-herdr-multilang-abi.md`](../20260722-herdr-multilang-abi.md).

---

## Controls (target)

| Control | Intent | Phase |
|---------|--------|-------|
| Allowlist interpreter prefixes (optional) | e.g. only `python3`, `node`, `bash`, absolute paths under plugin root | 2 |
| No shell-string hooks by default | Prefer argv arrays (herdr lesson) | 1 docs / 2 enforce for packages |
| Timeouts + max concurrency | Already in hooks settings — enforce for packages | 1–2 |
| Provenance in logs | `plugin:<id>` on every spawn | 1 with D4 |
| Path confinement (soft) | Reject `..` escape in manifest argv entries | 2 |
| Separate MCP trust | Inventory `--require-mcp-trust` | keep separate |
| BIN_PATH allowlist | D6 — not “entire CLI is API” | 1 docs |

### Argv array rule (frozen stance)

```toml
# Good
runner = { argv = ["python3", "hooks/pre_tool.py"] }

# Bad — reject for package ABI
runner = { shell = "python3 hooks/pre_tool.py && curl evil" }
```

`hooks.toml` may still support legacy string commands for compatibility; **new package ABI = arrays only**.

### Path confinement sketch

1. Resolve each path-like argv element against plugin root.
2. `canonicalize` / `dunce` on Windows.
3. Reject if result is outside plugin root **unless** allowlisted interpreter (absolute `python3` / `node` from PATH policy).
4. Log reject with provenance.

### Timeout / concurrency floor (align with existing)

| Knob | Existing hooks | Package / tools |
|------|----------------|-----------------|
| Per-handler timeout | `timeout_secs` (default settings ~30s) | Required on package runners; hard cap documented |
| Concurrent spawns | Dispatch settings | Cap tools lower than hooks (D11 sketch 4–8) |
| Cancel | Session abort → kill tree | Same OS notes |

Do not invent a second timeout subsystem — extend `execute_*` / DispatchConfig.

---

## Windows notes (herdr lessons)

- Prefer argv arrays over `cmd /c` strings.
- PATHEXT / `.exe` suffix awareness when checking allowlists.
- Do not assume POSIX shell available for package runners.
- Kill trees on timeout — document OS differences.
- `execute_command_hook` currently uses `Command::new("sh")` on some paths — **package argv** should spawn argv[0] directly (herdr), not wrap in `sh -c`.

Cite: herdr `plugin_command.rs` research in [`../20260722-herdr-multilang-abi.md`](../20260722-herdr-multilang-abi.md).

---

## Layering with D1 / D2 / D6 / D11

```text
enabled? → trusted? → path/argv validate → timeout/concurrency → spawn
                                              ↑
                                    BIN_PATH allowlist (callbacks)
```

### Freeze vs D11 (no contradiction)

| Doc | Stance |
|-----|--------|
| Master open Q #2 | MCP-only **or** argv in Phase 3? |
| **D11 freeze** | **MCP-first** ship order; `[[tools]] kind=argv` only after this file’s controls (or named tickets) |
| This file | Security gate for argv — does **not** force MCP-only forever |

See audit **Contradiction freeze** note.

D11 argv tools **must not** ship without these controls (or explicit follow-up tickets with owners).

---

## Explicit non-goals

- Full OS sandbox / seccomp as Phase 1 requirement.
- WASM as the Phase 1 security story (Phase 5 only).
- Treating trust as “safe.”
- Expanding BIN_PATH to arbitrary agent mutation verbs.
- Claiming Face Trust ACP already enforces D1 (it does not today).

---

## Acceptance tests (design)

| ID | Scenario | Pass |
|----|----------|------|
| SEC-01 | Manifest argv with `..` | Reject on enable/link |
| SEC-02 | Shell-string in package runner | Reject |
| SEC-03 | Spawn log includes `plugin:<id>` | Present |
| SEC-04 | Timeout kills hung runner | No session hang forever |
| SEC-05 | Docs security section | Package ABI / HOOKS.md |
| SEC-06 | MCP trust not implied by project trust | Documented |
| SEC-07 | Package argv does not go through `sh -c` string | Direct argv spawn |
| SEC-08 | Untrusted project | No package argv spawn (D1) |

Extend: `tests/hooks_integration.rs` timeout / deny cases; add package-link unit tests when D8 lands.

---

## Exit criteria

- [ ] Security section in package ABI docs.
- [ ] At least path-escape + argv-array rules enforced or documented as follow-up tickets.
- [ ] Provenance on spawns when D4 lands.
- [ ] D6 allowlist cross-linked.
- [ ] D11 references this file as gate for argv tools.
- [ ] Windows/direct-argv spawn path documented vs legacy `sh` command hooks.

---

## Ticketing rule

If Phase 2 ships manifest without full enforcement, each missing control becomes a **named follow-up** with owner — not a silent TODO in chat.

---

## Status

**Design contract for hardening.** Waiting **go ahead**; enforce progressively with D4/D8/D11.
