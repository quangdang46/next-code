# jcode → next-code Rebrand Contract

> Phase 0 source of truth. Override only by editing this file.  
> Companion: [`REBRAND_JCODE_TO_NEXT_CODE_AUDIT.md`](./REBRAND_JCODE_TO_NEXT_CODE_AUDIT.md),  
> [`REBRAND_IMPLEMENTATION_PLAN.md`](./REBRAND_IMPLEMENTATION_PLAN.md),  
> [`REBRAND_ALLOWLIST.md`](./REBRAND_ALLOWLIST.md).

**Status:** LOCKED for Phase 0–1 tooling (2026-07-17)  
**Owner:** product / maintainers of `quangdang46/next-code`

---

## 1. Naming matrix (canonical)

| Surface | Old | New | Notes |
|---|---|---|---|
| Display name | jcode / J-Code / Jcode | **Next Code** / next-code | User-facing chrome |
| CLI binary | `jcode` | **`next-code`** | Hyphen primary |
| Binary alias | — | **`jcode` → `next-code`** | Install symlink for **one release** only |
| Harness bin | `jcode-harness` | `next-code-harness` | |
| Eval bin | `jcode-edit-bench` | `next-code-edit-bench` | `jbench` stays unbranded |
| Cargo package | `jcode` / `jcode-foo` | **`next-code` / `next-code-foo`** | |
| Crate dir | `crates/jcode-foo` | `crates/next-code-foo` | All product crates |
| Eval dir | `evals/jcode-edit-bench` | `evals/next-code-edit-bench` | |
| Rust lib / path | `jcode` / `jcode_foo` | **`next_code` / `next_code_foo`** | |
| Root import | `jcode::` | `next_code::` | |
| Env vars | `JCODE_*` | **`NEXT_CODE_*`** | Not `NEXTCODE_*` |
| Home override | `JCODE_HOME` | **`NEXT_CODE_HOME`** | Dual-read during compat |
| Home dir | `~/.jcode` | **`~/.next-code`** | Dual-read + auto-migrate |
| Project dir | `.jcode/` | **`.next-code/`** | Long dual-read (git-committed) |
| XDG config | `~/.config/jcode` | `~/.config/next-code` | |
| XDG cache | `~/.cache/jcode` | `~/.cache/next-code` | Abandonable |
| Runtime socket | `jcode.sock` | `next-code.sock` | Same-release client+server |
| Daemon lock | `jcode-daemon.lock` | `next-code-daemon.lock` | |
| Windows install | `%LOCALAPPDATA%\jcode` | `%LOCALAPPDATA%\next-code` | Also migrate `%USERPROFILE%\.next-code` |
| Keyring (providers) | `jcode-provider-service` | **`next-code-provider-service`** | Dual-read + copy-forward |
| Keyring (secrets) | `jcode-secrets` | **`next-code-secrets`** | Dual-read + copy-forward |
| User-Agent | `jcode/{ver}` | **`next-code/{ver}`** | |
| URL scheme | `jcode://` | **`nextcode://`** + dual `jcode://` | Dual-read while needed |
| Process title | `jcode` | `next-code` | Linux 15-char OK |
| Types (Swift/docs) | `Jcode*` / `JCode*` | `NextCode*` / display "Next Code" | See iOS rules |
| Brew formula/tap | `homebrew-jcode` / `jcode.rb` | `homebrew-next-code` / `next-code.rb` | Dual-publish 1 cycle |
| AUR | `jcode-bin` | `next-code-bin` | |
| Lib path | `/usr/lib/jcode` | `/usr/lib/next-code` | |
| systemd units | `jcode-*.service` | `next-code-*.service` | |
| GitHub (live) | `1jehuang/jcode` (historical) | **`quangdang46/next-code`** | Installers / product URLs |
| Hotkey id | `jcode-hotkey` | `next-code-hotkey` | |
| Native lib | `libjcode_base` | `libnext_code_base` | |

**Do not use as primary identity:**

- `nextcode` as CLI binary (URL scheme only)
- `NEXTCODE_*` env family
- `Next-Code` / `nextCode` in Rust idents
- Rewriting competitor / third-party brand strings (see allowlist)

---

## 2. Locked product decisions

### 2.1 Binary alias

**Yes — one release.** Installers may install:

- Primary: `next-code`
- Compat symlink/copy: `jcode` → `next-code` for the first next-code release cycle only

Remove the alias in the following major (compat removal, §2.5). Document in release notes.

### 2.2 iOS bundle ID

**KEEP `com.jcode.mobile` for now** (safer App Store default).

- Bundle identifier strings: **do not rewrite** `com.jcode.mobile`
- URL schemes: register **dual** `nextcode` + `jcode` (`CFBundleURLSchemes`)
- Display name / marketing strings: **Next Code**
- Pairing URL name may use `com.nextcode.mobile.pair` only if it is a URL name, not the App Store bundle id

Phase 6 may revisit only after App Store Connect inventory.

### 2.3 Domains / hosts

**DO NOT rewrite `*.jcode.sh` hosts in this rebrand pass.**

Includes (non-exhaustive): `jcode.sh`, `api.jcode.sh`, `telemetry.jcode.sh`.

Reason: pending DNS / product decision. Code may dual-document new endpoints later; mechanical host rewrites are out of scope for P1–P5.

### 2.4 GitHub repository

| Use | Value |
|---|---|
| Live origin / installers / badges | **`quangdang46/next-code`** |
| Historical notes / changelog / origin-sync | `1jehuang/jcode` may remain |

Rewrite install scripts and live product URLs to `quangdang46/next-code`. Leave `1jehuang/jcode` in historical changelog and origin-sync narrative docs unless they are live install paths.

### 2.5 Compat removal

Dual-read / shims remain for **at least one major version and prefer ≥ 6 months** after the first next-code release that ships dual-read.

| Item | Policy |
|---|---|
| Window | ≥ 1 major; prefer 6 months from first dual-read release |
| Removal target | **TODO version** — fill before shipping dual-read (e.g. `next-code vX.Y` or calendar date) |
| Tracking | Open a tracked issue/bead when dual-read lands; close only when shims are deleted |

Shims that must die on the named version (not “later”):

- Env dual-read `JCODE_*` / `NEXT_CODE_*`
- Home dual-read `~/.jcode` / `~/.next-code`
- Project dir dual-read `.jcode/` / `.next-code/`
- Keyring dual-read
- URL scheme dual `jcode://`
- Binary alias `jcode`
- Provider-id / ACP legacy aliases if introduced

### 2.6 Eval suite

- Package / dir: `jcode-edit-bench` → `next-code-edit-bench`
- Binary `jbench` under `evals/jbench`: **leave unbranded** unless a later product decision expands scope

### 2.7 Origin-sync

This fork syncs upstream jcode. Rebrand PRs must either:

1. Land after a deliberate “stop tracking jcode brand from upstream” policy, or  
2. Include a rebase/merge playbook so origin-sync does not reintroduce `jcode` wholesale.

---

## 3. Phase boundaries (what this contract does *not* authorize)

| In scope for mechanical P1 tooling | Out of scope until later phase |
|---|---|
| Crate dirs, Cargo package/lib/bin names | User data migration behavior (P2) |
| Rust path idents `jcode::` / `jcode_foo` | Keyring copy-forward (P2b) |
| String rewrites per allowlist rules | Domain/DNS cutover |
| Install script GitHub owner rewrite | App Store bundle rename |
| | Compat shim *removal* (P9) |

---

## 4. Tooling entrypoints

| Script | Role |
|---|---|
| `scripts/rebrand/rename_crates.sh` | `git mv` crate/eval dirs |
| `scripts/rebrand/rewrite_cargo.py` | Cargo.toml package/path/lib/bin/feature rewrite |
| `scripts/rebrand/rewrite_rust_idents.py` | Token-aware Rust path/ident rewrite |
| `scripts/rebrand/rewrite_strings.py` | Multi-pass non-Cargo string rebrand |
| `scripts/rebrand/rg_gate.py` | Fail if unexpected residual `jcode` |
| `scripts/rebrand/run_p1.sh` | Orchestrate dir + Cargo + Rust ident passes |

**Do not run mass rename until Phase 1 is intentionally started.** Phase 0 only lands contract + tooling.

---

## 5. Acceptance for Phase 0

- [x] Naming matrix locked in this file
- [x] Compat removal noted as TODO version (≥6 months / next major after window)
- [x] iOS: keep `com.jcode.mobile` + dual URL schemes
- [x] Domains: no `*.jcode.sh` rewrite this pass
- [x] Binary alias: yes, one release
- [x] Live GitHub: `quangdang46/next-code`
- [x] Allowlist document exists
- [x] Rewrite scripts exist and are executable
