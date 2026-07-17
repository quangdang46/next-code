# next-code → next-code Rebrand Status

> Snapshot: 2026-07-17 (final residual sweep)  
> Branch: `rebrand/next-code`  
> Contract: [`REBRAND_CONTRACT.md`](./REBRAND_CONTRACT.md) · Allowlist: [`REBRAND_ALLOWLIST.md`](./REBRAND_ALLOWLIST.md)

## Verdict

| Check | Status |
|---|---|
| Crate dirs renamed (`crates/next-code-*`) | **Done** — no `jcode-*` crate dirs remain |
| Root package / bin names | **Done** — `name = "next-code"`, bin `next-code`, lib `next_code` |
| Workspace members | **Done** — all `next-code-*` |
| `cargo check -p next-code --bins` | **OK** (warnings only; 0 errors) |
| `cargo test -p next-code-storage --lib` | **OK** — 8/8 |
| `cargo test -p next-code-core --lib` | **OK** — 29/29 |
| `scripts/rebrand/rg_gate.py` | **Red debt meter** — ~1099 unexpected lines / 289 files after final residual sweep |

**Primary package compiles: YES.**

---

## What's done

### Structural rename

- All workspace crate directories under `crates/` are `next-code-*`.
- Root `Cargo.toml` package is `next-code`; binary is `next-code`.
- Path deps and `use next_code_*` imports resolve; workspace builds.

### Dual-read / migration shims (intentional residuals)

| Surface | Location | Behavior |
|---|---|---|
| Env dual-read | `crates/next-code-core/src/env.rs` — `product_env` / `product_env_os` / `product_var_full*` | `NEXT_CODE_{suffix}` then `JCODE_{suffix}` |
| Home dir | `crates/next-code-storage/src/lib.rs` — `next_code_dir` | `$NEXT_CODE_HOME` → `$JCODE_HOME` → `~/.next-code` with auto-migrate from `~/.jcode` |
| Project dir candidates | `PROJECT_DIR_CANDIDATES` | Prefer `.next-code`, fall back to `.jcode` |
| Runtime dir | `runtime_dir()` | via `product_env("RUNTIME_DIR")` |
| Keyring dual-read | `jcode-provider-service` / `jcode-secrets` | Write new service names; load legacy |
| Product provider id | `LoginProviderTarget::Jcode` / `provider::jcode::JcodeProvider` | First-party provider surface (not package rename miss) |
| Domains | `*.jcode.sh` | Domain freeze (contract §2.3) |
| iOS bundle | `com.jcode.mobile` | Keep until App Store rename |

### Mop-up landed this pass

- Bulk string / env rewrite WIP across crates, `src/`, scripts, docs, iOS, CI (~915 files).
- Env call sites migrated toward `product_env` / `NEXT_CODE_*` with dual-read preserved in `product_env`.
- Subscription catalog string values moved to `NEXT_CODE_*` / `next-code-subscription.env` (const *names* still `JCODE_*` during compat).
- User-facing strings in auth/account flows: “Next Code” / `next-code` where rewritten.
- Compile fixes after mechanical rewrite:
  - Restored `#[path]` modules in `crates/next-code-tui/src/tui/app/auth.rs` (import was inserted between attribute and `mod`).
  - Fixed `use next_code::env::…` inside the root package lib modules → `crate::env::…`.
  - Fixed `include!` dual-import of `product_env` in `src/cli/auth_test/*`.
- `rg_gate.py` allowlist expanded for intentional dual-read / provider module / keyring / domains / historical trees (`.beads/`, origin-sync skill) so the gate measures **product string debt**, not process history.

### Tooling

- `scripts/rebrand/{rename_crates.sh,rewrite_cargo.py,rewrite_rust_idents.py,rewrite_strings.py,rewrite_env_tokens.py,mop_env.py,rg_gate.py,run_p1.sh}`
- Plan / audit / contract / allowlist under `docs/REBRAND_*`

---

## Residual counts (2026-07-17 final residual sweep)

Filtered scan (excludes `.git`, `target`, `Cargo.lock`, `changelog/**`, `docs/*_PLAN.md`, `docs/plans/**`, `assets/**`, `docs/REBRAND_*`):

| Bucket | Count | Notes |
|---|---:|---|
| **Files with residual `jcode` (case-insensitive)** | **342** | Down from ~395 prior mop / ~1021 at structural cutover |
| **`JCODE_` tokens in `*.rs`** | **117** | Dual-read tests, dual-name consts, remaining call sites |
| **`rg_gate` allowlisted hits** | **1091** | Dual-read, domains, provider module, keyring, beads/origin-sync |
| **`rg_gate` unexpected hits** | **1099** | Product string / comment / script debt |
| **`rg_gate` unexpected files** | **289** | Gate still red; useful debt meter |

### Top unexpected debt surfaces (post-allowlist)

| Hits | Path | Class |
|---:|---|---|
| 46 | `crates/next-code-base/src/subscription_catalog.rs` | dual-name consts / tier labels |
| 43 | `telemetry-worker/README.md` | infra docs |
| 29 | `scripts/dev_cargo.sh` | scripts |
| 23 | `telemetry-worker/package.json` | package display names |
| 19 | `scripts/onboarding_sandbox.sh` | scripts |
| 17 | `crates/next-code-storage/src/lib.rs` | dual-read home paths |
| 17 | `crates/next-code-base/src/import_tests.rs` | import dual-read fixtures |
| … | scripts, provider-doctor, Cargo.toml comments | mop follow-ups |

### By token class (approx)

| Class | Intent |
|---|---|
| `product_env` + dual-read `JCODE_*` fallbacks | **Keep** until compat removal |
| `JCODE_*_ENV` const names (values often `NEXT_CODE_*`) | Compat window; rename later |
| `JcodeProvider` / `LoginProviderTarget::Jcode` | Product provider id (allowlisted) |
| `*.jcode.sh` / `com.jcode.mobile` | Domain / bundle freeze |
| User-facing `"jcode"` strings / comments | **Debt** — continue mop |
| Root `*_PLAN.md` historical plans | Skipped by measure globs / gate plan skip |

---

## Known follow-ups

### High priority

1. Finish remaining env/comment/user-string mop in top residual files (terminal-launch, config-types, subscription_catalog idents, provider-doctor labels).
2. Scripts (`dev_cargo.sh`, onboarding, benches) prefer `NEXT_CODE_*` with dual-read only where installers need it.
3. Docs narrative: **current runbooks cleaned** (`docs/plugins/**`, `docs/plugin-threat-model.md`, `changelog/README.md`, `PLAN_PARITY.md` crate paths, examples plugin comments). Remaining intentional residuals: dual-read notes, domain freeze (`*.jcode.sh`), live keyword ids (`canceljcode`/`stopjcode`), historical `*_PLAN.md` / `docs/plans/**`, REBRAND audit trees.
4. iOS display / type renames; keep `com.jcode.mobile`.
5. Telemetry worker package/display names when infra rename is scheduled.

### Locked / deferred by contract

| Item | Policy |
|---|---|
| Domains `*.jcode.sh` | Do not rewrite until DNS decision |
| iOS bundle id `com.jcode.mobile` | Keep until explicit App Store rename |
| Binary alias `jcode` → `next-code` | One release only |
| Dual-read env/home/keyring/URL | ≥ 1 major / prefer ≥ 6 months |
| `jbench` | Stay unbranded |
| Provider product id `Jcode` | Stable id until deliberate auth-state migration |

---

## Compat shims in place

```
next_code_core::env::product_env("X")
  → NEXT_CODE_X then JCODE_X

next_code_storage::next_code_dir()
  → NEXT_CODE_HOME | JCODE_HOME | ~/.next-code
  → auto-migrate ~/.jcode → ~/.next-code (+ .migrated-from-jcode)

PROJECT_DIR_CANDIDATES prefer .next-code then .jcode
runtime_dir() via product_env("RUNTIME_DIR")
keyring: next-code-* write, jcode-* dual-read
```

Removal tracked under contract §2.5 — **TODO version not yet filled**.

---

## How to re-measure

```bash
# Structural
ls crates | rg 'jcode' || echo 'no jcode crate dirs OK'
rg 'name\s*=\s*"jcode' -g 'Cargo.toml' || echo 'no package name jcode OK'

# Compile + dual-read tests
cargo check -p next-code --bins
cargo test -p next-code-storage --lib
cargo test -p next-code-core --lib

# Residual measure (same globs as mop-up)
rg -i 'jcode' --glob '!.git/**' --glob '!target/**' --glob '!Cargo.lock' \
  --glob '!changelog/**' --glob '!docs/*_PLAN.md' --glob '!docs/plans/**' \
  --glob '!assets/**' --glob '!docs/REBRAND_*' -l | wc -l
rg -c 'JCODE_' -g '*.rs' --glob '!.git/**' --glob '!target/**' \
  | awk -F: '{s+=$2} END{print s+0}'

# Debt meter
python3 scripts/rebrand/rg_gate.py --max-print 50
```

---

## Summary

Structural rebrand is **complete and builds**. Dual-read env/home/keyring paths are **implemented and tested** (storage 8/8, core 29/29). Final residual sweep continued narrative/comment/script mop, dual-name const polish, and embedding-path dual-read while preserving provider id aliases and domain/bundle freezes. **342 files / 117 `JCODE_` rust tokens / 1099 unexpected gate hits** remain as product string debt — gate stays red as a meter until those land. Domains, iOS bundle id, and dual-read shims stay per contract.
