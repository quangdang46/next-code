# Deepen — Host callback `NEXT_CODE_BIN_PATH` (2026-07-22)

**Priority:** P1 · Phase 1–2  
**Parent:** [herdr research](../20260722-herdr-multilang-abi.md) — `HERDR_BIN_PATH` portable callback  
**Gap:** Hooks get env + stdin; no stable “call next-code as API” for package authors.

---

## Frozen name

`NEXT_CODE_BIN_PATH` — absolute path to the running next-code (or face/daemon helper) binary suitable for subprocess callback.

Optional later: `NEXT_CODE_SOCKET` / ACP methods (Phase 2+). Prefer BIN_PATH first (Windows + Unix).

## When injected

| Context | Inject? |
|---------|---------|
| Hook `type=command` argv spawn | Yes (Phase 1 if cheap) |
| Future package action/tool argv | Yes |
| HTTP hooks | N/A (use HTTP); document alternative |
| User interactive shell | Not required |

## Contract (minimal)

1. Path is absolute and executable for the current OS.
2. Subcommands used by plugins are **documented + versioned** (start tiny: e.g. `next-code hook-callback --help` or reuse existing CLI surface — **open: exact subcommand set**).
3. Plugins must not assume cwd; pass ids via argv/env JSON.
4. Failure of callback must not hang the hook past timeout.

## Non-goals (v1)

- Full NDJSON control plane parity with herdr socket.
- Letting plugins re-enter agent loop arbitrarily without allowlisted verbs.

## Exit criteria

- [ ] Env set on command-hook spawn.
- [ ] Cookbook shows one callback (e.g. log/notify) using BIN_PATH.
- [ ] Documented in HOOKS.md / package ABI when landed.
