# Deepen — Argv plugin security beyond trust (2026-07-22)

**Priority:** P2 · Phase 1–2 hardening  
**Parent:** Trust gate + herdr Windows argv notes  
**Related:** [`20260722-trust-gate-design.md`](./20260722-trust-gate-design.md)

---

## Threat model (short)

Trusted project still means **user-approved RCE** in that tree. Trust ≠ sandbox. Additional controls reduce blast radius and foot-guns.

## Controls (target)

| Control | Intent |
|---------|--------|
| Allowlist interpreter prefixes (optional) | e.g. only `python3`, `node`, `bash`, absolute paths under plugin root |
| No shell-string hooks by default | Prefer argv arrays (herdr lesson) |
| Timeouts + max concurrency | Already in hooks settings — enforce for packages |
| Provenance in logs | `plugin:<id>` on every spawn |
| Path confinement (soft) | Reject `..` escape in manifest argv entries |
| Separate MCP trust | Inventory flag |

## Explicit non-goals

- Full OS sandbox / seccomp as Phase 1 requirement.
- Treating trust as “safe untrusted code.”

## Exit criteria

- [ ] Security section in package ABI docs.
- [ ] At least path-escape + argv-array rules enforced or documented as follow-up tickets.
