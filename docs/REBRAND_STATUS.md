# jcode → next-code Rebrand Status

> Snapshot: 2026-07-17  
> Branch: `rebrand/next-code`  
> Contract: [`REBRAND_CONTRACT.md`](./REBRAND_CONTRACT.md) · Allowlist: [`REBRAND_ALLOWLIST.md`](./REBRAND_ALLOWLIST.md)

## Verdict

| Check | Status |
|---|---|
| Crate dirs renamed (`crates/jcode-*` → `crates/next-code-*`) | **Done** — no `jcode` crate dirs remain |
| Root package / bin names | **Done** — `name = "next-code"`, bin `next-code`, lib `next_code` |
| Workspace members | **Done** — all `next-code-*` |
| `cargo metadata --no-deps` | **OK** |
| `cargo check -p next-code-storage` | **OK** |
| `cargo check -p next-code-core` | **OK** |
| `cargo check -p next-code-build-support` | **OK** |
| `cargo check -p next-code` (primary package) | **OK** (warnings only; 0 errors) |
| `cargo test -p next-code-storage --lib` | **OK** — 8/8 after env-lock race fix |
| `scripts/rebrand/rg_gate.py` | **Red** — ~12k unexpected residual lines (expected; string pass incomplete) |

**Primary package compiles: YES.**

---

## What's done

### Structural rename (P1-ish)

- All workspace crate directories under `crates/` are `next-code-*` (no `jcode-*` dirs).
- Root `Cargo.toml` package is `next-code`; binary is `next-code`; harness is `next-code-harness`.
- Rust crate package names in `Cargo.toml` files no longer use `name = "jcode..."`.
- Path deps and most `use next_code_*` / `next_code::` imports resolve; workspace builds.

### Dual-read / migration shims (intentional residuals)

| Surface | Location | Behavior |
|---|---|---|
| Env dual-read | `crates/next-code-core/src/env.rs` — `product_env` / `product_env_os` / `product_var_full*` | `NEXT_CODE_{suffix}` then `JCODE_{suffix}` |
| Home dir | `crates/next-code-storage/src/lib.rs` — `next_code_dir` | `$NEXT_CODE_HOME` → `$JCODE_HOME` → `~/.next-code` with auto-migrate from `~/.jcode` |
| Deprecated alias | `jcode_dir()` | `#[deprecated]`, forwards to `next_code_dir()` |
| Project dir candidates | `PROJECT_DIR_CANDIDATES = [".next-code", ".jcode"]` | Prefer new, fall back to legacy |
| Runtime dir | `runtime_dir()` | `$NEXT_CODE_RUNTIME_DIR` / `$JCODE_RUNTIME_DIR` via `product_env` |
| Migrate marker | `.migrated-from-jcode` | Written after one-shot home migrate |
| Product provider id | `LoginProviderTarget::Jcode` / `provider::jcode::JcodeProvider` / `ProviderChoice::Jcode` | **Product surface** (self-hosted / first-party provider), not a package rename miss |

### Tooling present

- `scripts/rebrand/{rename_crates.sh,rewrite_cargo.py,rewrite_rust_idents.py,rewrite_strings.py,rg_gate.py,run_p1.sh}`
- Plan / audit / contract / allowlist under `docs/REBRAND_*`
- Gate path-allowlists rebrand tooling + `docs/REBRAND_*` (including this status file)

### Fix landed this session

- **Storage test env race:** `lib` tests and `active_pids` tests each held a *private* `Mutex` while mutating `NEXT_CODE_HOME` / `JCODE_HOME`, so parallel runs interleaved and failed (`falls_back_jcode_home`, `session_counts_*`, `streaming_guard_*`).
- **Fix:** crate-level `test_env::{lock_env, clear_home_env}` shared by both modules.
- Files: `crates/next-code-storage/src/lib.rs`, `crates/next-code-storage/src/active_pids.rs`.

---

## Remaining `jcode` hit counts

Filtered scan (excludes `.git`, `target`, `Cargo.lock`, `changelog/**`, `docs/*_PLAN.md`, `docs/plans/**`, `assets/**`):

| Bucket | Files (approx) | Notes |
|---|---:|---|
| **Total files with residual `jcode`** | **~1021** | Case-insensitive |
| `crates/**` | 688 | Bulk of debt — comments, docs in crate trees, `JCODE_*` call sites, tests |
| `scripts/**` | 124 | Budgets, installers, CI helpers |
| `docs/**` | 69 (+ plans excluded) | Narrative + design docs still say jcode |
| `src/**` | 51 | CLI / app spine residual strings & env names |
| `ios/**` | 41 | Module/display renames pending; **keep `com.jcode.mobile`** |
| `tests/**` | 16 | Integration / matrix |
| `telemetry-worker/**` | 11 | Worker / D1 historical names |
| Root prose (`README`, `PARITY`, `AGENTS`, …) | ~10 | User-facing docs |
| Eval / examples | ~10 | Partial |

### By token class (line hits, rough)

| Class | Approx lines | Intent |
|---|---:|---|
| `JCODE_*` env / const tokens | **~4844** | Dual-read window + incomplete call-site rewrite to `NEXT_CODE_*` / `product_env` |
| `NEXT_CODE_*` already present | ~358 | Partial adoption |
| `.jcode` path strings | ~669 | Dual-read + docs/comments still naming old home |
| `jcode.sh` / `*.jcode.sh` domains | ~55 | **Domain freeze** (contract §2.3) — allowlisted |
| `com.jcode` (iOS bundle) | ~27 | **Keep** `com.jcode.mobile` until App Store rename |
| `jcode::` path imports | **3** | Only `provider::jcode::JcodeProvider` (product module) |
| `jcode_` Rust idents | **1** | `jcode_dir()` deprecated alias |
| `name = "jcode…"` in Cargo.toml | **0** | Clean |
| Crate dirs named `jcode*` | **0** | Clean |

### `rg_gate.py` (post allowlist tweak)

```
files_scanned:    ~1893
allowlisted_hits: ~292+ (dual-read / domains / historical / tooling)
unexpected_hits:  ~12000+
unexpected_files: ~1026
```

Gate is a **debt meter**, not a ship gate, until the string / env rewrite pass finishes. Prefer fixing product strings over expanding the allowlist (allowlist §10).

---

## Known follow-ups

### High priority (product correctness / cutover)

1. **Env call-site rewrite** — migrate remaining raw `JCODE_*` reads/writes to `product_env` / `NEXT_CODE_*` (or dual-set where child processes need both during compat).
2. **Comment / description pass** — Cargo.toml `description = "… for jcode"`, module docs still saying `~/.jcode`, `jcode_app_core` in comments.
3. **User-facing strings** — TUI chrome, help text, errors, installers (`jcode` → `next-code` + one-release binary alias).
4. **Scripts / budgets / CI** — `scripts/*budget*.json`, installers, flake.nix, telemetry worker names.
5. **Docs / README / PARITY / MASTER_UI** — narrative rebrand; leave historical changelog and origin-sync logs alone.

### Locked / deferred by contract

| Item | Policy |
|---|---|
| Domains `*.jcode.sh` | **Do not rewrite** until DNS decision |
| iOS bundle id `com.jcode.mobile` | **Keep** until explicit App Store rename |
| Binary alias `jcode` → `next-code` | One release only, then remove |
| Dual-read env/home/keyring/URL | ≥ 1 major / prefer ≥ 6 months; set removal TODO version |
| `jbench` | Stay unbranded |
| `1jehuang/jcode` | Historical only; live origin is `quangdang46/next-code` |

### iOS

- Display / type renames (`Jcode*` → `NextCode*` / “Next Code”) still pending.
- Dual URL schemes: `nextcode` + `jcode`.
- Bundle id stays `com.jcode.mobile` for now.

### Dual-publish / distribution

- Homebrew / AUR / install.sh dual formula for one cycle.
- Release assets `next-code-<os>-<arch>`; optional `jcode` symlink in installers.

### Origin-sync

- Upstream remains historically `jcode`; origin-sync skill logs may keep that name.
- After rebrand, sync process must not reintroduce `jcode-*` crate paths into live product trees without dual-read wrapping.

### Provider naming note

- `JcodeProvider` / `LoginProviderTarget::Jcode` / CLI value `"jcode"` is the **first-party provider id**, not a leftover package name. Decide separately whether the *provider product id* renames to `next-code` (likely yes for consistency) vs keeping a stable id for auth state.

---

## Compat shims in place

```
next_code_core::env::product_env("X")
  → NEXT_CODE_X then JCODE_X

next_code_storage::next_code_dir()
  → NEXT_CODE_HOME | JCODE_HOME | ~/.next-code
  → auto-migrate ~/.jcode → ~/.next-code (+ .migrated-from-jcode)

#[deprecated] jcode_dir() → next_code_dir()

PROJECT_DIR_CANDIDATES = [".next-code", ".jcode"]

runtime_dir() via product_env("RUNTIME_DIR")
app_config_dir / legacy_app_config_dir dual paths
```

Removal tracked under contract §2.5 — **TODO version not yet filled**.

---

## How to re-measure

```bash
# Structural
ls crates | rg 'jcode' || echo 'no jcode crate dirs OK'
rg 'name\s*=\s*"jcode' -g 'Cargo.toml' || echo 'no package name jcode OK'

# Compile
cargo check -p next-code-storage -p next-code-core -p next-code

# Storage dual-read tests
cargo test -p next-code-storage --lib

# Residual gate
python3 scripts/rebrand/rg_gate.py --max-print 50
```

---

## Summary

Structural rebrand (crate dirs, package names, primary bin/lib) is **complete and builds**. Storage dual-read + migration path is **implemented and tested**. Large residual debt remains in **env tokens, comments, docs, scripts, and iOS display strings** — gate will stay red until those passes land. Domains, iOS bundle id, and dual-read shims are **intentionally retained** per contract.
