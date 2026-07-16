# jcode → next-code Implementation Plan

> Companion to [`REBRAND_JCODE_TO_NEXT_CODE_AUDIT.md`](./REBRAND_JCODE_TO_NEXT_CODE_AUDIT.md).  
> Goal: phased, always-buildable rebrand with dual-read migration — not a bulk sed.

**Assumed freeze (Phase 0 defaults — override only via written contract):**

| Surface | Canonical new value |
|---|---|
| CLI binary / clap / artifacts | `next-code` |
| Cargo package | `next-code` / `next-code-foo` |
| Rust crate / `use` path | `next_code` / `next_code_foo` |
| Env family | `NEXT_CODE_*` |
| Home | `~/.next-code` (`NEXT_CODE_HOME`) |
| Project dir | `.next-code/` |
| Keyring services | `next-code-provider-service`, `next-code-secrets` |
| User-Agent | `next-code/{version}` |
| URL scheme | `nextcode` (+ dual `jcode` while needed) |
| Display | `Next Code` / `next-code` |
| Compat window | **≥ 1 major / prefer 6 months**; removal version written in contract |
| Binary alias | optional one-release `jcode` → exec `next-code` |
| iOS bundle id | **TBD** (App Store Connect check blocks Phase 7) |
| Domains | **TBD** (do not sed hosts until DNS/product decision) |

---

## 0. Operating rules (non-negotiable)

1. **Tree always builds.** Every PR (or stacked PR group merged together) must pass `cargo check -p next-code` (or transitional name) and a targeted test subset.
2. **No naive `sed s/jcode/next-code/g`.** Token-aware renames only. Maintain a denylist (third-party OAuth UA, competitor paths, historical plans, `DO_NOT_TRACK`, OpenAI URL paths, etc.).
3. **Dual-read before hard cut.** Env, home, project dir, keyring, provider id, URL scheme, artifacts — never remove old without a published window.
4. **Atomic Cargo identity.** Package name + crate dir + path deps + `[lib]` name + `use`/`jcode::` rewrite + `Cargo.lock` regen + profile package stanzas + feature `dep:` strings = **one green unit**.
5. **Privacy on migrate.** If legacy `no_telemetry` (or equivalent) exists and new marker missing, preserve opt-out.
6. **Write the removal date.** Shims die on a named version, not “later”.
7. **Origin-sync awareness.** This fork syncs upstream `jcode`. Rebrand PRs must either (a) land after a deliberate “stop tracking jcode brand from upstream” policy, or (b) include a rebase/merge playbook so origin-sync doesn’t reintroduce `jcode` wholesale.

---

## 1. Phase map & PR graph

```
P0 Contract ──► P1 Cargo identity ──► P2 Storage/env dual-read ──► P3 Wire/protocol
                      │                        │                         │
                      │                        ▼                         ▼
                      │                  P2b Keyring/provider      P4 CLI UX + install
                      │                        │                         │
                      └────────────────────────┴──────────► P5 CI/packaging
                                                              │
                         P6 iOS/desktop (blocked on bundle decision)
                                                              │
                         P7 Docs ──► P8 Tests/scripts/rg-gate ──► P9 Compat removal
```

| Phase | PR unit(s) | Depends on | Effort (eng-days, 1–2 people) | Risk |
|---|---|---|---|---|
| **P0** Contract + allowlist + tooling | 1 small docs PR | — | 0.5–1 | Low |
| **P1** Cargo identity (dirs/packages/idents) | 1–2 large PRs (must stay green) | P0 | 3–6 | Med (merge conflicts) |
| **P2** Storage + env dual-read + migrate | 1–2 PRs | P1 (or can land *before* P1 if names stay `jcode_*` temporarily — see note) | 3–5 | **High** (data) |
| **P2b** Keyring + provider id alias | 1 PR | P2 helpers | 1–2 | High (creds) |
| **P3** Wire (UA, ACP, URL scheme dual) | 1 PR | P1 for crate paths; P2 optional | 1–2 | Med |
| **P4** CLI UX + installers + proctitle | 1–2 PRs | P1 bins + P2 home | 2–3 | Med |
| **P5** CI/release/packaging/systemd | 1–2 PRs | P4 artifact names | 2–3 | Med (release day) |
| **P6** iOS/desktop | 1–2 PRs | P0 iOS decision | 2–4 | High if App Store |
| **P7** Public docs / AGENTS / examples | 1 PR | P4–P5 names stable | 1–2 | Low |
| **P8** Tests/scripts/goldens + rg gate | 1–2 PRs | All above | 2–4 | Med (flaky e2e) |
| **P9** Compat removal | 1 PR on named major | Published window elapsed | 1–2 | Med |

**Recommended order nuance:**  
Prefer **P2 dual-read helpers first under old names** only if you need a long-running branch for P1. Default path: **P0 → P1 → P2** so new API names (`next_code_dir`, `NEXT_CODE_HOME`) ship with the crate rename and avoid a second rename pass.

**Total calendar estimate:** ~3–6 weeks focused work, or longer if origin-sync / App Store / DNS block.

---

## 2. Phase 0 — Contract, allowlist, rename tooling

### Goal
Stop re-asking naming questions; give agents/humans a single source of truth and safe rewrite tools.

### Deliverables
1. `docs/REBRAND_CONTRACT.md` (or section in this file) with:
   - Naming matrix (table above)
   - Compat removal target: e.g. `next-code v1.0` or `2026-12-01`
   - iOS: `keep com.jcode.mobile` | `new com.nextcode.mobile` | `pending`
   - Domain policy: keep `*.jcode.sh` | migrate | pending
   - Binary alias: yes/no for one release
2. `docs/REBRAND_ALLOWLIST.md` — residual `jcode` strings that are **legal** after cutover:
   - Compat dual-read symbols / comments
   - Historical `changelog/**`, `docs/*_PLAN.md`, origin-sync notes
   - Third-party OAuth UA (`claude-cli/...`, `codex_cli_rs`, antigravity)
   - Competitor import paths (`.claude/`, `.codex/`)
   - Telemetry D1 `database_name` if kept historical
3. Tooling under `scripts/rebrand/`:
   - `rename_crates.sh` — `git mv crates/jcode-FOO crates/next-code-FOO` for all 92
   - `rewrite_cargo_tomls.py` — package names, path deps, members, features, profile stanzas
   - `rewrite_rust_idents.py` — token-aware `jcode_foo` → `next_code_foo`, `jcode::` → `next_code::`
   - `rg_gate.sh` — fail CI if unexpected `jcode` outside allowlist
4. Issue/bead checklist linked to this plan.

### Gates
- [ ] Contract reviewed by product owner
- [ ] Dry-run rewrite scripts on a throwaway worktree produce a tree that *almost* `cargo metadata`s (full green is P1)

### Non-goals
- No product behavior change yet
- No user-visible install change

---

## 3. Phase 1 — Cargo workspace identity

### Goal
Workspace compiles and tests under `next-code` package/crate identity. No user data path change yet (still may read `JCODE_HOME` / `~/.jcode` via unchanged storage logic until P2).

### Scope (measured)
| Item | Count / anchor |
|---|---|
| Crate dirs | 92 under `crates/jcode-*` |
| Cargo.toml files with brand | 96 |
| Root path deps `jcode-foo = { path = ... }` | 69 |
| Workspace members | 93 (91 contain `jcode`) |
| `jcode::` | ~329 |
| `use jcode_` | ~650 |
| Explicit `[lib] name = "jcode*"` | ~42 |
| Profile stanzas | `profile.*.package."jcode-tui-anim"` (dev/selfdev/test) |
| Bins | `jcode`, `jcode-harness` (+ evals) |
| Detector | `is_jcode_repo` in `crates/jcode-build-support/src/paths.rs:565` |

### Steps (atomic PR A — recommended single mega-PR or stacked “merge train”)

#### 1.1 Directory renames
```bash
# scripts/rebrand/rename_crates.sh
for d in crates/jcode-*; do
  git mv "$d" "crates/next-code-${d#crates/jcode-}"
done
git mv evals/jcode-edit-bench evals/next-code-edit-bench  # if rebranding eval
# leave evals/jbench path; only package name may change
```

#### 1.2 Root + crate Cargo.toml rewrite
For every `Cargo.toml`:
- `name = "jcode"` → `name = "next-code"`
- `name = "jcode-foo"` → `name = "next-code-foo"`
- `path = "crates/jcode-foo"` → `path = "crates/next-code-foo"`
- `[lib] name = "jcode"` → `name = "next_code"`
- `[lib] name = "jcode_foo"` → `name = "next_code_foo"`
- `[[bin]] name = "jcode"` → `name = "next-code"`
- `[[bin]] name = "jcode-harness"` → `name = "next-code-harness"`
- Features: `dep:jcode-embedding` → `dep:next-code-embedding`, `jcode-tui/embeddings` → `next-code-tui/embeddings`
- Profiles:
  ```toml
  [profile.dev.package."next-code-tui-anim"]
  ```
  (same for selfdev/test — **silent perf bug if missed**)
- Workspace members list rewrite
- Optional: set `publish = false` on remaining ~40 packages lacking it

#### 1.3 Rust identifier rewrite (token-aware)
| Pattern | Replacement |
|---|---|
| `\bjcode::` | `next_code::` |
| `\bjcode_([a-z0-9_]+)` | `next_code_$1` |
| `\buse jcode\b` | `use next_code` |
| `extern crate jcode` | `extern crate next_code` |

**Do not rewrite in this phase (leave for P2+):**
- String literals `"jcode"`, `"JCODE_"`, `"~/.jcode"`, `"jcode-provider-service"`
- Comments that are historical
- Process title string prefixes (P4)
- User-Agent string constants (P3)

Practical approach:
1. Run mechanical rewrite for idents only.
2. `cargo check --workspace` and fix leftovers (macro paths, `include!`, build.rs).
3. `cargo test -p next-code-storage --lib` etc. for smoke.

#### 1.4 Detectors (must land with package rename)
`crates/jcode-build-support` → `next-code-build-support`, function:

```rust
// paths.rs — after rename
pub fn is_next_code_repo(dir: &Path) -> bool { /* ... */ }
// keep alias during transition:
pub fn is_jcode_repo(dir: &Path) -> bool { is_next_code_repo(dir) }

fn package_name_is_product(content: &str) -> bool {
    content.contains("name = \"next-code\"")
        || content.contains("name = \"jcode\"") // dual-accept during transition
}
```

Update all call sites to prefer `is_next_code_repo`; keep `is_jcode_repo` as `#[deprecated]` wrapper until P9.

#### 1.5 Lockfile
```bash
cargo generate-lockfile   # or cargo check
# never hand-edit Cargo.lock
```

#### 1.6 Minimal binary/clap so CI can still invoke something
In the **same** PR as bin rename:
- `src/cli/args.rs`: `#[command(name = "next-code")]`
- about string can wait for P4 polish, but name must match bin

Optional same PR: second `[[bin]] name = "jcode"` pointing at same `main.rs` **only if** Cargo allows duplicate mains via alias — usually **not**. Prefer install-time symlink (P4/P5) instead of two bins.

### Tests / gates
- [ ] `cargo metadata` succeeds
- [ ] `cargo check --workspace`
- [ ] `cargo test -p next-code-base --lib` (or renamed)
- [ ] `cargo test -p next-code-build-support --lib`
- [ ] `rg 'crates/jcode-' -g 'Cargo.toml'` → empty
- [ ] `rg '\bjcode::' -g '*.rs'` → empty (except allowlisted comments if any)
- [ ] Profile keys reference `next-code-tui-anim` only

### Rollback
Revert the mega-PR. Do not half-revert dirs without Cargo.toml.

### Risks
- Origin-sync merge hell — coordinate freeze window with upstream sync
- Missed profile package → anim crate compiles unoptimized (no error)
- build.rs / include paths with hard-coded `jcode-`

---

## 4. Phase 2 — Storage paths, env dual-read, first-run migrate

### Goal
New users land on `~/.next-code` + `NEXT_CODE_*`. Existing users keep credentials/sessions without manual steps.

### Anchors
| Symbol | File |
|---|---|
| `jcode_dir` | `crates/next-code-storage/src/lib.rs` (~L74) |
| `app_config_dir` | same (~L113) |
| `user_home_path` | same (~L128) |
| `runtime_dir` / `JCODE_RUNTIME_DIR` | same (~L21) |
| `durable_state_dir` | same (~L97) |
| `active_pids` | `.../active_pids.rs` |
| Env test helpers | `crates/next-code-core/src/env.rs` |
| Call sites `jcode_dir()` | ~192 |
| Files reading `JCODE_*` via `env::var` | ~220 |

### Design

#### 2.1 Env dual-read helper (new, central)
Add to `next-code-core` (or `next-code-storage`):

```rust
/// Read NEXT_CODE_{suffix} then JCODE_{suffix}.
pub fn product_env(suffix: &str) -> Result<String, VarError> {
    let new_key = format!("NEXT_CODE_{suffix}");
    let old_key = format!("JCODE_{suffix}");
    match std::env::var(&new_key) {
        Ok(v) => Ok(v),
        Err(VarError::NotPresent) => {
            let v = std::env::var(&old_key)?;
            // once: tracing::debug!(old_key, "using legacy env; prefer {new_key}");
            Ok(v)
        }
        Err(e) => Err(e),
    }
}

pub fn product_env_os(suffix: &str) -> Option<OsString> { /* same order */ }
```

Policy:
| Class | Behavior |
|---|---|
| User-facing (`HOME`, API keys, hooks, telemetry, install, sockets) | dual-read |
| Test-only (`TEST_*`, `E2E_*`, harness) | hard-cut to `NEXT_CODE_*` in tests as you touch them |
| Dynamic prefixes (`PROVIDER_{NAME}_API_KEY`, hook fields) | try `NEXT_CODE_` then `JCODE_` at construction |

#### 2.2 Directory resolution + migrate
```rust
pub fn next_code_dir() -> Result<PathBuf> {
    if let Ok(path) = product_env("HOME") {
        return Ok(PathBuf::from(path));
    }
    let home = dirs::home_dir().ok_or_else(|| anyhow!("No home directory"))?;
    let new = home.join(".next-code");
    let old = home.join(".jcode");
    ensure_migrated(&old, &new)?; // rename/copy + marker
    Ok(new)
}

// Deprecated alias until P9:
pub fn jcode_dir() -> Result<PathBuf> { next_code_dir() }
```

**`ensure_migrated(old, new)` algorithm:**
1. If `new` exists → return (optionally still read legacy opt-out files if marker missing).
2. If `old` missing → create `new`, return.
3. Try `std::fs::rename(old, new)` (same volume).
4. On cross-device error → recursive copy then leave `old` with a `MIGRATED` note file (or delete after verify — prefer leave until uninstall purge window).
5. Write `new.join(".migrated-from-jcode")` with timestamp + source path.
6. Log once user-visible: `Migrated jcode data to ~/.next-code`.
7. **Privacy:** before any telemetry init, if `old/no_telemetry` or equivalent exists and new lacks explicit opt-in, copy opt-out forward.

Same pattern for:
- `app_config_dir`: `~/.config/next-code` dual with `~/.config/jcode`; under `*_HOME` sandbox: `$HOME/config/next-code` (and dual-read old segment `jcode` under sandbox).
- Windows install layout (P4/P5): `%LOCALAPPDATA%\next-code` dual `%LOCALAPPDATA%\jcode` and `%USERPROFILE%\.jcode`.

#### 2.3 Project-local dual-read
Discovery order for agents/skills/mcp/notepad:
1. `$PROJECT/.next-code/`
2. `$PROJECT/.jcode/`
3. (existing competitor paths unchanged)

Provide `next-code migrate-project` (optional P4) to `git mv .jcode .next-code`.

#### 2.4 Sockets / runtime
| Old | New |
|---|---|
| `JCODE_RUNTIME_DIR` | `NEXT_CODE_RUNTIME_DIR` (dual) |
| `jcode.sock` | `next-code.sock` |
| `jcode-daemon.lock` | `next-code-daemon.lock` |
| named runtime dir `.../jcode/` | `.../next-code/` |

**Rule:** client and server from the **same release** must agree. During dual-publish, prefer new socket; if connect fails and legacy socket exists, fall back once (optional, document).

#### 2.5 Call-site strategy
- Prefer renaming function to `next_code_dir` + deprecate `jcode_dir` alias (zero call-site churn first).
- Bulk-replace env string literals via helper, not 220 one-off edits without tests.

### Tests (required)
| Test | Assert |
|---|---|
| `migrates_legacy_home_on_first_run` | empty new + populated old → new has marker + data |
| `prefers_next_code_home_env` | both envs set → uses `NEXT_CODE_HOME` |
| `falls_back_to_jcode_home_env` | only `JCODE_HOME` → works |
| `fresh_user_gets_next_code_dir` | neither legacy → creates `~/.next-code` |
| `preserves_telemetry_opt_out` | legacy opt-out → still opted out |
| `app_config_dual_read` | secrets found under old config segment |
| `project_dir_prefers_new` | both `.jcode` and `.next-code` → new wins |
| Windows (if CI): both LocalAppData trees |

### Gates
- [ ] Unit tests above green
- [ ] Manual: copy a real `~/.jcode` fixture → launch → sessions list non-empty
- [ ] Uninstall dry-run docs list both trees

### Risks
- Agents hardcoding `home.join(".jcode")` bypassing `jcode_dir()` — grep and fix in this phase:
  `rg 'join\("\.jcode"\)|\.jcode"' -g '*.rs'`
- Menubar lock hardcoding `~/.jcode` — dual-check both pid paths
- Partial migrate (crash mid-copy) — prefer rename; if copy, write marker only after fsync verify

---

## 5. Phase 2b — Keyring + provider id

### Goal
Credentials survive rebrand; new writes use new service names.

### Anchors
| Service | File |
|---|---|
| `jcode-provider-service` | `crates/next-code-provider-service/src/store/keyring.rs:21` |
| index account `__index__` | same |
| `jcode-secrets` | `crates/next-code-secrets/src/lib.rs:364` |
| Provider id `"jcode"` | `jcode-base` auth lifecycle, `jcode-provider-metadata` catalog |

### Algorithm (provider keyring)
```text
load(id):
  try service=next-code-provider-service
  if miss: try jcode-provider-service
  if hit on legacy:
      save copy to new service
      update new index
      (optional) delete legacy
save(id):
  always write new service (+ update new index)
  optional: also write legacy during dual window (usually NO — avoid split brain)
list:
  union(new index, legacy index) de-duped
```

Same for `next-code-secrets` / `jcode-secrets`.

### Provider id
- Catalog display name → “Next Code” / id `next-code` (or keep internal id `jcode` if product wants stable id — **contract decision**).
- **Recommended:** new canonical id `next-code`; accept `jcode` | `subscription` | `jcode-subscription` as aliases in auth lifecycle maps (existing alias table in `auth/lifecycle.rs`).

### Tests
- Mock keyring: legacy-only entry loads and copy-forwards
- Mock keyring: new entry does not read legacy
- Auth lifecycle: stored provider `jcode` still activates

### Gates
- [ ] macOS manual: `security find-generic-password -s jcode-provider-service` still works via app load
- [ ] After load, entry exists under `next-code-provider-service`

---

## 6. Phase 3 — Wire / protocol / HTTP identity

### Goal
Outbound product identity says `next-code`; inbound accepts legacy where needed.

### Items
| Item | Action |
|---|---|
| `JCODE_USER_AGENT` / `jcode/{ver}` | Rename const → `NEXT_CODE_USER_AGENT` = `next-code/{ver}` |
| `jcode-updater`, `jcode-embedding/...` | Rename UA strings |
| ACP `_jcode/*` meta | Dual-advertise `_next_code/*` + `_jcode/*` one release |
| URL scheme `jcode://` | Emit `nextcode://`; parse **both** |
| Plugin JS global `jcode` / `__jcode_api` | Dual-bind `nextcode` + legacy (contract: JS token form) |
| package.json key `"jcode"` | Prefer `"next-code"` or `"nextcode"`; dual-read |
| Telemetry product field | Add `product=next-code`; keep D1 names historical |
| Third-party UA (Claude CLI, codex_cli_rs, antigravity) | **Do not change** |

### Anchors
- `crates/next-code-provider-core` `JCODE_USER_AGENT`
- `crates/next-code-app-core` update.rs user agents
- `ios/.../Info.plist` CFBundleURLSchemes
- `src/cli/commands.rs` pair URI format
- `crates/next-code-plugin-runtime` API injection

### Tests
- UA unit test expects `next-code/`
- PairURI.parse accepts `jcode://` and `nextcode://`
- Plugin runtime exposes both globals (if dual-bind chosen)

### Gates
- [ ] No product UA still starts with `jcode/` except allowlisted dual-read tests
- [ ] iOS scheme dual-registered if listing kept

---

## 7. Phase 4 — CLI UX, process titles, install/uninstall

### Goal
User-visible brand and install paths match shipped binary.

### 4.1 Clap / help / chrome
- `src/cli/args.rs`: name `next-code`, about `Next Code: ...`
- TUI titles, menubar, ACP strings (`src/cli/acp.rs`)
- Login prompts mentioning `jcode`

### 4.2 Process titles (Linux 15-char limit)
Current (`crates/next-code-base/src/process_title.rs`):
```rust
const KILLALL_PROCESS_NAME: &str = "jcode"; // 5 chars
// prefixes: "jcode:s:", "jcode:c:", "jcode:d:", "jcode:selfdev", "jcode:client"
```

After binary `next-code` (9 chars):
| Role | New compact prefix | Notes |
|---|---|---|
| killall / prctl name | `next-code` | exactly 9 ≤ 15 |
| server | `nc:s:` | **not** `next-code:s:` (overflow) |
| client | `nc:c:` | |
| selfdev client | `nc:d:` | |
| generic | `nc:` / `next-code` | fit limit via `compact_process_title` |

Update uninstall `pkill` patterns accordingly (match both during dual window).

### 4.3 Installers
| File | Changes |
|---|---|
| `scripts/install.sh` | REPO `quangdang46/next-code` (or contract); artifact `next-code-{os}-{arch}`; home `~/.next-code`; bin `next-code`; dual-download fallback to `jcode-*` one release; `next_code_configure_path` |
| `scripts/install.ps1` | `%LOCALAPPDATA%\next-code\bin`; same dual-download |
| `scripts/uninstall.sh` | purge both trees; kill `next-code` and legacy `jcode` |
| `scripts/install_release.sh` | launcher paths |
| `scripts/lib/configure_path.sh` | rename function; keep old function as wrapper one release |

Optional: install symlink `jcode` → `next-code` when `NEXT_CODE_INSTALL_ALIAS=1` or always for one release.

### 4.4 Self-update / selfdev paths
- Update GitHub latest-release URLs in Rust (`update.rs`, selfdev) to new repo **after** contract
- Build output dir names under `~/.next-code/builds`

### Tests
- `scripts/test_install_conversion.sh` (or equivalent) against temp dir
- Process title unit tests for 15-char bound
- Snapshot of `next-code --help` first line

### Gates
- [ ] Fresh curl\|bash install → `command -v next-code` and `next-code --version`
- [ ] Uninstall removes new layout; `--purge` removes legacy too

---

## 8. Phase 5 — CI/CD, packaging, systemd

### Goal
CI produces and verifies `next-code-*` artifacts; packaging docs match.

### Workflows
| File | Touch points |
|---|---|
| `.github/workflows/ci.yml` | bin path, `JCODE_E2E_*` → `NEXT_CODE_E2E_*` (or dual), artifact dirs |
| `release.yml` | artifact names `next-code-linux-x86_64` etc.; dual-upload `jcode-*` one cycle; brew formula path |
| `windows-smoke.yml` / freebsd-* | exe names, package `-p next-code` |
| `ios-testflight.yml` | only if P6 renames schemes |

### Packaging
| Channel | Action |
|---|---|
| Homebrew | New formula `next-code.rb` in owned tap; dual formula one cycle if possible |
| AUR | File `next-code-bin`; update `scripts/update_packages.sh` `/usr/lib/next-code` |
| Nix `flake.nix` | package/bin rename |
| `packaging/linux/*.desktop` | rename file + `Exec=` + `Icon=` |
| phone-server systemd | `next-code-pair.service`, `next-code-serve.service`; runbook: `systemctl disable jcode-*.service && enable next-code-*.service` |
| Windows Startup | write `next-code-hotkey.lnk`; remove `jcode-hotkey.lnk` on upgrade |

### Gates
- [ ] Dry-run release workflow on tag from branch
- [ ] Windows verify script asserts new exe + Startup name
- [ ] FreeBSD artifact name consistent with install.sh map

---

## 9. Phase 6 — iOS / desktop assets

### Blocked on App Store Connect answer

| If | Then |
|---|---|
| `com.jcode.mobile` **never** shipped | Rename to `com.nextcode.mobile`; schemes `nextcode` primary; types `NextCodeKit` / `NextCodeMobile` |
| **Shipped** / TestFlight live | **Keep** bundle id + team; dual URL schemes; optional display name “Next Code”; keyring service dual-read `com.jcode.mobile.servers` |

### Mechanical (either path)
- `git mv` Swift modules if renaming types
- `ios/project.yml`, `Package.swift`, TestHarness, `scripts/phone-server/*`
- Desktop crate already renamed in P1; assets: `Jcode.icns` → `NextCode.icns`, demo filenames optional
- QR emitter + pair service URI

### Gates
- [ ] Decision recorded in contract
- [ ] Pair flow works with CLI from P4
- [ ] Keychain credentials still load on device upgrade path

---

## 10. Phase 7 — Docs, AGENTS, examples, evals

### Goal
Human/agent onboarding matches shipped binary.

### Do rewrite
- `README.md`, `AGENTS.md`, `CONTRIBUTING.md`, `RELEASING.md`, `OAUTH.md`, `TELEMETRY.md`
- `docs/*.md` that are **current** runbooks (not archaeology)
- Plugin examples (`"jcode"` package key, globals)
- `entities.json` project name

### Do not bulk-rewrite
- `docs/*_PLAN.md`, `docs/plans/**`, old changelog JSON (add one-line header: “Historical: product was named jcode”)
- Upstream origin-sync notes that quote old remote

### Eval packages
- Rename package `jcode-jbench` → `next-code-jbench` (P1 may have done this)
- Keep binary `jbench` unbranded unless product wants full rebrand

### Gates
- [ ] README install one-liner works against real release or dry-run
- [ ] AGENTS.md paths use `~/.next-code` and `NEXT_CODE_*`

---

## 11. Phase 8 — Tests, scripts, goldens, rg gate

### Goal
No test still assumes `jcode` binary or `JCODE_HOME` without dual-read; CI enforces allowlist.

### Work
1. `CARGO_BIN_EXE_jcode` → `CARGO_BIN_EXE_next-code` (hyphen env: cargo uses `CARGO_BIN_EXE_next-code` — verify escaping in tests).
2. E2E fixtures: env keys, socket names, expected help strings.
3. Scripts under `scripts/**`: default `JCODE_BIN` → `NEXT_CODE_BIN` with fallback.
4. Demo scripts titles `/tmp/jcode-demo` → `/tmp/next-code-demo`.
5. Desktop gallery goldens: **only regen if chrome shows brand**; otherwise leave.
6. `.gitignore`: `/.next-code/generated-images/`, `libnext_code_base.rlib`; keep old entries one cycle.
7. Land `scripts/rebrand/rg_gate.sh` in CI:
   ```bash
   rg -i 'jcode' --glob '!target/**' ... | filter_allowlist || exit 1
   ```

### Gates
- [ ] Full CI green
- [ ] `rg_gate.sh` green on main
- [ ] Definition of Done checklist in audit doc ticked

---

## 12. Phase 9 — Compat removal (named major)

### Goal
Delete dual-read debt on the version written in the contract.

### Remove
- `JCODE_*` fallbacks in `product_env`
- Auto-migrate from `~/.jcode` (optional: keep read-only warn)
- Keyring legacy service reads
- Provider id alias `jcode`
- URL scheme `jcode`
- Dual artifact publish + `jcode` binary alias
- `is_jcode_repo` deprecated alias
- `jcode_dir` deprecated alias
- Install dual-download

### Communicate
- CHANGELOG migration section
- Blog/README “breaking: remove jcode compat”
- Uninstall notes for leftover `~/.jcode` if any

### Gates
- [ ] Contract removal version == release version
- [ ] rg allowlist shrinks to pure history / third-party only

---

## 13. Automation & agent playbook

### Safe rewrite pipeline (per PR)
```text
1. worktree / branch from main
2. run scripts/rebrand/* dry-run → diff stat
3. apply
4. cargo check --workspace
5. cargo test -p <touched>
6. scripts/rebrand/rg_gate.sh (phase-appropriate)
7. human review of denylist hits
8. PR
```

### Anti-patterns
| Don’t | Do instead |
|---|---|
| Global sed across repo | Phase-scoped token rewrite |
| Rename packages without dirs | Atomic P1 |
| Change `~/.jcode` without migrate | P2 `ensure_migrated` |
| Change keyring service only | Dual-read + copy-forward |
| Rewrite historical changelogs | Allowlist |
| Sed `jcode.sh` domains | Product/DNS ticket |
| Two half-merged origin-sync + rebrand | Freeze sync or rebrand branch isolation |

### Suggested agent fleet split (when implementing)
| Agent wave | Scope |
|---|---|
| Wave A | P1 scripts + Cargo.toml/dirs only |
| Wave B | P1 rust idents by crate cluster (base / tui / provider / app-core / rest) |
| Wave C | P2 storage + tests |
| Wave D | P2b keyring + provider aliases |
| Wave E | P3–P4 strings + installers |
| Wave F | P5 workflows |
| Wave G | P7–P8 docs/tests/rg |

Use worktrees per wave; merge train ordered P1 → P2 → …

---

## 14. Release-day runbook (first rebranded release)

1. Tag with dual artifacts (`next-code-*` + `jcode-*` if still dual-publishing).
2. Update install scripts on `main` default branch (raw.githubusercontent URLs).
3. Publish brew/AUR/nix.
4. Smoke:
   - macOS/Linux: curl install → `next-code --version` → login still works on migrated home
   - Windows: install.ps1 → PATH → Startup shortcut
   - iOS pair QR if applicable
5. Announce compat window end date.
6. Monitor telemetry for crash spikes / auth failures / “home dir” errors.

---

## 15. Definition of Done (implementation)

Copy from audit; must all be true before declaring rebrand complete (pre-P9):

- [ ] Workspace package + primary bin are `next-code`
- [ ] `cargo test` CI green
- [ ] Fresh install → `~/.next-code` only
- [ ] Legacy `~/.jcode` auto-migrates; keyring dual-read works
- [ ] `NEXT_CODE_*` documented canonical; `JCODE_*` dual-read live
- [ ] Install/uninstall/help/AGENTS consistent
- [ ] CI artifacts `next-code-*`
- [ ] iOS decision recorded and implemented
- [ ] `rg_gate` allowlist-only residuals
- [ ] Compat removal version published

---

## 16. Immediate next actions (when you say “go”)

1. **Confirm/edit Phase 0 defaults** (especially iOS + domains + alias + removal date).  
2. Land **P0** (`REBRAND_CONTRACT.md` + `scripts/rebrand/*` stubs + allowlist).  
3. Open **P1 worktree** and run crate `git mv` + Cargo.toml rewrite until `cargo check --workspace` is green.  
4. Land **P2** migrate tests before advertising any install URL change.  

---

## 17. Traceability

| Plan phase | Audit sections |
|---|---|
| P0 | §1 matrix, §2 decisions |
| P1 | Blockers B1–B5, H4, H7 |
| P2 | B6–B8, H1–H3, M1–M3 |
| P2b | B9, H5 |
| P3 | H6, wire findings, plugin DOC-005 |
| P4 | CLI findings, install UX, proctitle |
| P5 | CI/packaging, systemd, Windows Startup |
| P6 | iOS bundle/scheme/keychain |
| P7–P8 | Docs/tests DoD |
| P9 | Compat removal |

**Primary code anchors:**  
`Cargo.toml` · `crates/*/src` (post-rename `next-code-*`) · `crates/next-code-storage/src/lib.rs` · `crates/next-code-provider-service/src/store/keyring.rs` · `crates/next-code-build-support/src/paths.rs` · `crates/next-code-base/src/process_title.rs` · `scripts/install.sh` · `.github/workflows/release.yml`
