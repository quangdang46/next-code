# jcode → next-code Rebrand Audit

## 0. Executive summary

**Scale (unique census; surface sums are not additive)**

| Metric | Value |
|---|---|
| Unique brand hits (whole repo, excl. noise) | ~**17,648** |
| Files with brand surface | ~**900+** (Cargo 96 manifests; env 479; Rust idents 574 `.rs`) |
| Cargo packages still `jcode*` | **95 / 96** (only `scripts/repro/tls-bad-record-mac` is unbranded) |
| Crate dirs `crates/jcode-*` | **92** (90 workspace members + 2 non-members) |
| Workspace members | **93** (91 paths contain `jcode`) |
| Explicit `[lib] name = "jcode*"` | **42** |
| Branded bins | `jcode`, `jcode-harness`, `jcode-edit-bench` |
| `jcode_*` Rust idents | ~**3,749** hits / **237** unique tokens in `.rs` |
| `jcode::` root-crate paths | **329** raw / ~**318** real in **22** files |
| `JCODE_*` env tokens | **~5,067** / **691** unique names / **151** in `CONFIG_ENV_KEYS` |
| Dual-read / migrate shims today | **0** |

**Current state**

- GitHub/repo directory is already **next-code**; product identity in code is still **100% jcode**.
- Root package/lib/bin: `name = "jcode"` (`/Users/tranquangdang21/Projects/next-code/Cargo.toml`).
- Distribution is **brew/curl**, not crates.io — still a hard break for installers, PATH, CI artifacts, selfdev, and user data under `~/.next-code`.
- Hardcoded product URLs still point at `1jehuang/jcode`, `jcode.sh`, `api.jcode.sh`, `telemetry.jcode.sh`.

**Risk posture: HIGH / data-loss capable**

Hard-cutting without dual-read loses:

1. Sessions, OAuth tokens, memory, MCP trust under `~/.next-code`
2. Provider keyring (`jcode-provider-service`)
3. Project-committed `.jcode/` agents/skills
4. Install PATH + brew formula + CI artifact consumers

**Inventory corrections (dropped/adjusted)**

- Cargo.toml `jcode-` hits: **600** (not 944); any-`jcode`: **673**
- `jcode_*` files: **574** `.rs` (not 534); tokens **3,749** (not 3,307); unique **237** (not 149)
- `publish=false`: **56** packages (not 55); **40** still missing
- Surface hit sums (~21k inventory / ~38k deep) **double-count** shared lines — use unique ~17.6k for capacity
- Non-jcode package exists: `tls-bad-record-mac-repro` — leave alone
- No snap/flatpak/Docker/VS Code/man-page channels exist — do not invent them for rebrand parity

---

## 1. Naming convention matrix (LOCK THIS IN)

| Surface | Old | New | Notes |
|---|---|---|---|
| Display name | jcode | Next Code / next-code | User-facing chrome |
| CLI binary | `jcode` | **`next-code`** | Hyphen primary; optional temp alias `jcode` |
| Harness bin | `jcode-harness` | `next-code-harness` | |
| Eval bin | `jcode-edit-bench` | `next-code-edit-bench` | `jbench` stays unbranded unless full eval rebrand |
| Cargo package | `jcode` / `jcode-foo` | `next-code` / `next-code-foo` | |
| Crate dir | `crates/jcode-foo` | `crates/next-code-foo` | All 92 + evals dir |
| Rust lib / path | `jcode` / `jcode_foo` | `next_code` / `next_code_foo` | |
| Root import | `jcode::` | `next_code::` | ~318 sites |
| Env vars | `JCODE_*` | **`NEXT_CODE_*`** | Not `NEXTCODE_*` |
| Home dir | `~/.jcode` | `~/.next-code` | Dual-read + auto-migrate |
| Project dir | `.jcode/` | `.next-code/` | Long dual-read (git-committed) |
| XDG config | `~/.config/jcode` | `~/.config/next-code` | Secrets live here |
| XDG cache | `~/.cache/jcode` | `~/.cache/next-code` | Abandonable |
| Runtime socket | `jcode.sock` | `next-code.sock` | Same-release client+server |
| Named socket dir | `runtime/jcode/` | `runtime/next-code/` | |
| Daemon lock | `jcode-daemon.lock` | `next-code-daemon.lock` | |
| Windows install | `%LOCALAPPDATA%\jcode` | `%LOCALAPPDATA%\next-code` | Also migrate `%USERPROFILE%\.next-code` |
| Keyring service | `jcode-provider-service` | `next-code-provider-service` | Dual-read + copy-forward |
| User-Agent | `jcode/{ver}` | `next-code/{ver}` | |
| Bundle ID | `com.jcode.*` | **decision** | Keep if App Store shipped |
| URL scheme | `jcode://` | `nextcode://` + dual | |
| Types | `Jcode*` | `NextCode*` | Provider id dual-read |
| Brew formula/tap | `homebrew-jcode` / `jcode.rb` | `homebrew-next-code` / `next-code.rb` | Dual-publish 1 cycle |
| AUR / lib path | `/usr/lib/jcode`, `jcode-bin` | `/usr/lib/next-code`, `next-code-bin` | |
| systemd units | `jcode-*.service` | `next-code-*.service` | phone-server |
| Process title | `jcode` | `next-code` | Linux 15-char OK |

**Do not use:** `nextcode` as primary binary, `NEXTCODE_*` env, `Next-Code` / `nextCode` in Rust.

---

## 2. Critical product decisions (need human)

1. **Public CLI slug?**  
   Options: `next-code` / `nextcode` / `next-code` + `nc` alias.  
   **Rec:** `next-code` primary everywhere; short alias later only.

2. **Env / home / socket family?**  
   Options: `NEXT_CODE_*`+`~/.next-code` / `NEXTCODE_*`+`~/.nextcode` / keep jcode forever.  
   **Rec:** `NEXT_CODE_*` + `~/.next-code` + `next-code.sock`; dual-read ≥1 major (prefer 2 / 6+ months).

3. **GitHub repo, domains, Homebrew, AUR?**  
   Options: hard cut / rename+redirects+dual-publish / keep `1jehuang/jcode` forever.  
   **Rec:** Lock new repo+domain **before** packaging. Dual-publish artifacts one cycle. Keep telemetry infra names historical unless funded migration.

4. **iOS App Store listing?**  
   Options: keep `com.jcode.mobile` / new bundle / unknown.  
   **Rec:** **Block on App Store Connect check.** If ever shipped, keep bundle id + dual URL schemes.

5. **Provider id / OAuth / keyring keys?**  
   Options: hard rename / keep forever / display rename + dual-read.  
   **Rec:** Display+UA → next-code; dual-read provider id `jcode`, keyring service, ACP `_jcode/*` for ≥2 majors.

6. **Root Rust lib name?**  
   Options: `next_code` / shorter facade / keep `jcode::`.  
   **Rec:** `next_code` + rewrite all `jcode::` in lockstep. Temp **binary** alias only, not crate root.

7. **Eval suite brand?**  
   Package `jcode-jbench` → `next-code-jbench`; bin `jbench` stay?  
   **Rec:** Rename packages; leave `jbench` binary unbranded unless product wants full eval rebrand.

8. **`JCODE_USE_XDG`?** Documented, **not implemented**.  
   **Rec:** Drop docs claim during rebrand unless product wants real XDG mode (then migrate `~/.local/share/jcode` too).

9. **Domain endpoints** (`api.jcode.sh`, `jcode.sh`, `telemetry.jcode.sh`)?  
   **Rec:** Product/DNS decision separate from code rename; do not sed-rename hosts blindly.

10. **Compat removal date?**  
    **Rec:** Write explicit major/version now — shims must not become permanent.

---

## 3. Confirmed findings by severity

### Blockers

**B1 · cargo-package-names** — All 95 product packages still `jcode` / `jcode-*`  
- Evidence: root `Cargo.toml:2 name = "jcode"`; `crates/jcode-base/Cargo.toml`; 95/96 manifests  
- Proposed: `next-code` / `next-code-foo`; set `publish = false` on remaining 40  
- Migration: Atomic with dirs, deps, lib/bin names, lock regen  

**B2 · crate-dir-renames** — 92 `crates/jcode-*` dirs (incl. non-members `ext-hello`, `import-core`)  
- Evidence: 92/92 dirs; import-core still path-dep of app-core/base  
- Proposed: `git mv` → `crates/next-code-*` all 92  
- Migration: Same PR as members + path deps  

**B3 · workspace-members + path-deps** — 91 member paths + 69 root path deps + ~600 `jcode-` Cargo.toml hits  
- Evidence: 93 members; every Cargo.toml contains jcode  
- Proposed: members/deps → `next-code-*` paths and keys  
- Migration: One commit; cargo metadata fails on partial  

**B4 · lib names + rust idents + `jcode::`** — 42 `[lib]` overrides; ~3749 `jcode_*`; ~318 `jcode::`  
- Evidence: `[lib] name = "jcode"`; top root `jcode_base` 815 hits; `src/main.rs` `jcode::run()`  
- Proposed: `next_code` / `next_code_foo`; rewrite imports  
- Migration: Mechanical crate-root pass first; defer `jcode_dir` helpers to storage PR  

**B5 · binary-names** *(needs decision on form)* — `jcode`, `jcode-harness`, `jcode-edit-bench`  
- Evidence: `Cargo.toml` bins; `src/cli/args.rs name = "jcode"`; release artifacts `jcode-linux-*`  
- Proposed: `next-code` (+ optional alias bin `jcode` one release)  
- Migration: clap, install.sh/ps1, CI, scripts, tests  

**B6 · E-001/E-002 home plane** — `jcode_dir()`: `$JCODE_HOME` → `~/.next-code` (~1037 `JCODE_HOME` hits)  
- Evidence: `crates/jcode-storage/src/lib.rs:74-81`; install/uninstall; sessions/auth/memory  
- Proposed: `next_code_dir()` / `NEXT_CODE_HOME` / `~/.next-code`  
- Migration: First-run rename/copy + `.migrated-from-jcode`; dual-read env  

**B7 · E-003 config secrets** — `app_config_dir()` → `~/.config/jcode` (provider `*.env`)  
- Evidence: `jcode-storage` app_config_dir; login/provider_init  
- Proposed: `~/.config/next-code`  
- Migration: Same first-run pass; dual-read if new missing  

**B8 · E-004 project `.jcode/`** — agents/skills/mcp/notepad/theme (often committed)  
- Evidence: agent-runtime registry; notepad; mcp protocol; skill.rs  
- Proposed: `.next-code/` + dual-read `.jcode/`  
- Migration: Long dual-read; `next-code migrate-project` for git mv  

**B9 · E-005 keyring** — service `jcode-provider-service`  
- Evidence: `crates/jcode-provider-service/src/store/keyring.rs`  
- Proposed: `next-code-provider-service`  
- Migration: Read new→old; copy-forward on hit  

### High

**H1 · Bulk env `JCODE_*` → `NEXT_CODE_*`** (~691 unique / 5067 tokens / 151 CONFIG_ENV_KEYS)  
- Central dual-read helper; hard-cut test-only vars; dynamic prefixes (`JCODE_PROVIDER_{}_API_KEY`, hooks, plugin kill)  

**H2 · Runtime sockets** — `JCODE_SOCKET`, `jcode.sock`, named dir, daemon lock, Windows pipes  
- Same-release client+server; hyphen OK on Windows pipe sanitizer  

**H3 · Windows dual layout** — `%LOCALAPPDATA%\jcode` + `%USERPROFILE%\.next-code` + Startup `jcode-hotkey.lnk`  
- Migrate both trees; fix uninstall purge; rewrite Startup shortcut  

**H4 · `is_jcode_repo()`** — hard-checks `name = "jcode"` in root Cargo.toml  
- Evidence: `jcode-build-support/src/paths.rs`  
- Dual-accept `next-code` + legacy `jcode` or selfdev/telemetry break silently  

**H5 · Domain provider types** — `JcodeProvider`, `LoginProviderTarget::Jcode`, runtime key `"jcode"`  
- Dual-read stored auth/provider ids; separate from pure Cargo rename  

**H6 · GitHub/update URLs in Rust** — `1jehuang/jcode` in selfdev, update.rs, catalog, telemetry  
- Functional constants must change after repo decision  

**H7 · Features + profile overrides** — `dep:jcode-embedding`, `jcode-tui/*`, `profile.*.package."jcode-tui-anim"`  
- Feature miss = broken optional deps; profile miss = **silent** anim CPU regression  

**H8 · systemd phone-server units** — `jcode-pair.service`, `jcode-serve.service`, socket paths  
- Ship new units + migration runbook  

**H9 · Packaging** — brew tap `homebrew-jcode`, AUR `/usr/lib/jcode`, release artifact basenames  
- Dual-publish one cycle  

**H10 · Doc/code drift `JCODE_USE_XDG`** — docs only; not in `jcode_dir()`  
- Drop or implement; if users followed docs, check `~/.local/share/jcode`  

### Medium

**M1 · Menubar lock hardcodes `~/.next-code`** (bypasses `JCODE_HOME`) — dual-PID check during coexistence  
**M2 · Agents discovery uses `home.join(".jcode")` not `jcode_dir()`** — centralize during rebrand  
**M3 · harden_user_config_permissions hardcodes segment** — harden `app_config_dir()` + legacy  
**M4 · User-Agent / subscription host branding** — UA yes; hosts need product plan  
**M5 · Install helper `jcode_configure_path()`** — scripts/lib + install.sh inline copy  
**M6 · Telemetry dashboard** — `jcode_dash_token` localStorage; CORS `jcode.sh`; keep D1 names historical  
**M7 · XDG cache segment** — abandonable cold-start only  
**M8 · `.gitignore`** — `/.jcode/generated-images/`, `libjcode_base.rlib`  
**M9 · Self-dev clone path** `.../source/jcode` + binary names in builds  

### Low / bulk mechanical

- Docs/README/AGENTS (~126 `~/.next-code` in md; GitHub URL hits)
- Demo scripts (`record_demo.sh` titles, `/tmp/jcode-demo`)
- `entities.json` project name `"jcode"`
- Eval package renames; unbranded bins stay
- `publish=false` + optional authors/license/repository metadata
- Cargo.lock: **regenerate only**, never hand-edit (94 jcode stanzas)

### Do-not-change / historical

| Keep | Why |
|---|---|
| `DO_NOT_TRACK` | Industry standard |
| `config.toml` basename | Unbranded |
| Competitor dual-reads (`.claude/`, `.mcp.json`, `.agents/skills/`) | Intentional interoperability |
| Claude Code disclaimer strings | Product copy, not brand residual |
| Third-party crates (cosmic-text, etc.) | Not jcode |
| Historical `*_PLAN.md`, changelog archaeology | Footnotes; don't bulk-rewrite |
| Telemetry D1/dataset names | Infra cost; add `product=next-code` field |
| OpenAI path strings `/v1/chat/completions` | Not brand |
| `tls-bad-record-mac-repro` | Unrelated package |
| No invent snap/flatpak/Docker/VSIX/man channels | Don't exist today |

---

## 4. Compat & migration plan

### Env dual-read
```
next_code_var("HOME") → NEXT_CODE_HOME then JCODE_HOME (log once at debug if legacy)
```
- User-facing (HOME, API keys, hooks, telemetry, install): dual-read ≥1 major  
- Test/CI vars (`JCODE_TEST_*`, `JCODE_E2E_*`, …): hard-cut OK  
- Dynamic prefixes: try `NEXT_CODE_` then `JCODE_` at construction sites  

### Storage path migration (first run)
1. If `~/.next-code` missing and `~/.next-code` exists → **rename** (fallback copy)  
2. Write `~/.next-code/.migrated-from-jcode` + one user-visible log line  
3. Same for `~/.config/jcode` → `~/.config/next-code`  
4. Windows: both `%LOCALAPPDATA%\jcode` and `%USERPROFILE%\.next-code`  
5. Always honor legacy `no_telemetry` if new marker missing (privacy)  
6. Project: resolve `.next-code/` then `.jcode/` (mirror existing `.claude` dual-read)  
7. Uninstall `--purge`: clear **both** trees after migrate window  

### Binary alias
- Ship `next-code` primary  
- Optional one-release `jcode` symlink/wrapper → `exec next-code`  
- Dual-publish CI artifacts `jcode-*` + `next-code-*` one cycle  

### Keyring
- Read: `next-code-provider-service` then `jcode-provider-service`  
- On legacy hit: copy-forward to new service; optional delete old  

### OAuth / provider id / ACP
- Accept stored provider id `"jcode"` as alias of `"next-code"`  
- Dual-advertise ACP `_jcode/*` and `_next_code/*` one release (or keep `_jcode/*` longer if external clients)  
- URL schemes: register both if iOS listing continues  

### Detectors
- `is_jcode_repo`: match `name = "next-code"` **and** temporarily `"jcode"`  

### Allowlist residual `jcode` after cutover
Compat dual-read symbols · historical changelogs/plans · intentional competitor strings · third-party OAuth client IDs · telemetry infra names if kept · origin-sync upstream notes  

---

## 5. Suggested implementation sequencing

Phased so **tree always builds**; freeze decisions in Phase 1 first.

| Phase | Title | Why / key items |
|---|---|---|
| **1** | Freeze naming + ownership | Binary/env/home/domain/bundle-id contract in docs/issue; App Store Connect check; repo/domain/tap ownership |
| **2** | Cargo identity + detectors | git mv 92 crates; packages/libs/deps/features/profile; rewrite `jcode_*`/`jcode::`; regen lock; `is_jcode_repo` dual-accept |
| **3** | Runtime paths + env + data migrate | storage helpers, auto-migrate home/config, keyring, Windows+Startup, project dual-read, `.gitignore` |
| **4** | Wire/protocol + HTTP | User-Agent, ACP meta, provider id aliases, telemetry product field, URL schemes |
| **5** | CLI UX + install | clap name, menubar, install/uninstall, `next_code_configure_path`, process title, optional binary alias |
| **6** | CI/CD + packaging + systemd | workflows, dual artifacts, brew/AUR/nix, phone-server units, FreeBSD names |
| **7** | iOS / desktop assets | Only after Phase 1 iOS decision; schemes dual-accept |
| **8** | Docs / AGENTS / examples / evals | README paths; plugin examples; jbench package; leave `*_PLAN.md` archaeology |
| **9** | Tests / scripts / goldens | `CARGO_BIN_EXE_*`, e2e fixtures, demo scripts, entities.json; final `rg` gate |
| **10** | Compat removal (later major) | Drop dual-read env/home/keyring/URL/artifacts/alias on written date |

**PR atomicity rules**
- Package rename + dir rename + path deps + lib names + source imports = **one** green PR (or stacked PRs that each build)  
- Profile `jcode-tui-anim` must not lag package rename  
- Do not rename env/home without dual-read in same ship  

---

## 6. Definition of Done checklist

- [ ] `cargo build` / workspace tests pass with package+bin **`next-code`**
- [ ] No required package name `jcode` remains except documented alias/compat
- [ ] `rg -i jcode` over source empty **outside allowlist** (history/compat/upstream/lock)
- [ ] Fresh install creates `~/.next-code` (or `$NEXT_CODE_HOME`); new users never need `~/.next-code`
- [ ] Existing `~/.jcode` auto-migrates (or documented migrate command); credentials load via keyring dual-read
- [ ] `JCODE_*` works during published dual-read window; `NEXT_CODE_*` is canonical in docs
- [ ] CLI `--help`, process title, menubar, install/uninstall, AGENTS.md all say **next-code**
- [ ] CI emits `next-code-*` artifacts (and dual `jcode-*` only if still in dual-publish window)
- [ ] Windows verify script asserts `next-code.exe` + Startup shortcut new name
- [ ] iOS decision recorded (keep bundle + dual schemes **or** new listing acceptance)
- [ ] Phone-server units/sockets updated **or** migration runbook published
- [ ] `.gitignore` tracks `.next-code/` + new rlib names; `entities.json` updated
- [ ] Compat **removal version/date written down**
- [ ] No invented snap/flatpak/Docker/VSIX/man channels
- [ ] `is_jcode_repo` accepts `next-code`
- [ ] Telemetry opt-out never silently re-enabled on migrate
- [ ] Uninstall `--purge` knows legacy + new trees on Unix and Windows

---

## 7. Missed / residual risk

| Residual | Severity | Note |
|---|---|---|
| Windows Startup hotkey (`.lnk` / legacy `.vbs`) | High | Completeness critic; not only registry |
| systemd phone-server unit **basenames** on deployed hosts | High | Need disable/enable runbook |
| External shell rc / CI exporting `JCODE_*` | High | Cannot be grepped in-repo |
| Distro mirrors outside repo (AUR, brew tap content) | High | Coordinate release day |
| iOS App Store irreversibility | High | Bundle id decision is product-legal |
| Domain/DNS (`jcode.sh`, telemetry host) | High | Code rename ≠ DNS cutover |
| Menubar dual-install two helpers | Medium | Check both pid files during coexistence |
| `JCODE_USE_XDG` users under `~/.local/share/jcode` | Medium | If any followed broken docs |
| Agent paths ignoring `JCODE_HOME` sandbox | Medium | Pre-existing; fix while rebranding |
| Telemetry dashboard CORS + localStorage | Medium | Worker/dashboard cutover |
| Silent profile opt-level miss on tui-anim | Medium | No compile failure |
| External path/git dependents on `jcode-*` | Low–Med | Assumed path-local only; confirm |
| Historical docs inflation | Low | Don't bulk-rewrite archaeology |
| Completions/man/snap/flatpak/Docker/VSIX | None today | Absence is intentional; add only under new name later |
| Naive `sed jcode→next-code` | Critical process risk | Token-aware renames + denylist required |

**Open process debt:** no live install/keychain exercise was run; Windows `%APPDATA%` vs `%LOCALAPPDATA%` not validated on a real host; full unique `JCODE_*` name list (~691) not line-itemed — mechanical once dual-read helper exists.

---

### Bottom line

Rebrand is an **epic, multi-phase, data-migration project**, not a find-replace. Cargo identity (~8k of the work) is mechanical but must be atomic. User-data plane (`~/.jcode`, keyring, project `.jcode/`, env) is the **real risk**. Lock the naming matrix and ownership decisions (Phase 1) before any mass rename; ship dual-read first, drop shims on a written schedule.

**Primary code anchors**

- `/Users/tranquangdang21/Projects/next-code/Cargo.toml`
- `/Users/tranquangdang21/Projects/next-code/crates/jcode-storage/src/lib.rs`
- `/Users/tranquangdang21/Projects/next-code/crates/jcode-provider-service/src/store/keyring.rs`
- `/Users/tranquangdang21/Projects/next-code/crates/jcode-build-support/src/paths.rs`
- `/Users/tranquangdang21/Projects/next-code/scripts/install.sh` · `install.ps1` · `uninstall.sh`