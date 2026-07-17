# Rebrand residual mop-up plan

> Snapshot: 2026-07-17  
> Branch: `rebrand/next-code`  
> Contract: [`REBRAND_CONTRACT.md`](./REBRAND_CONTRACT.md) · Allowlist: [`REBRAND_ALLOWLIST.md`](./REBRAND_ALLOWLIST.md) · Status: [`REBRAND_STATUS.md`](./REBRAND_STATUS.md)

Structural crate/package rename is done. This plan classifies **residual brand debt** and the mechanical order to mop it without breaking dual-read.

## Contract recap (do not violate)

| KEEP (dual-read / forever) | Canonical target |
|---|---|
| `product_env` falls back to `JCODE_*` | Prefer `NEXT_CODE_*` / `product_env("SUFFIX")` at call sites |
| `jcode_dir` deprecated alias | `next_code_dir` |
| `PROJECT_DIR_CANDIDATES` includes `.jcode` | Prefer `.next-code` first |
| Keyring legacy service names | Write `next-code-*`, dual-read `jcode-*` |
| URL scheme `jcode` dual | Prefer `nextcode://` |
| `is_jcode_repo` wrapper if present | Thin compat |
| `com.jcode.mobile` | Forever until App Store rename |
| `*.jcode.sh` domains | Domain freeze |
| Third-party UA (`claude-cli`, `codex_cli_rs`) | Never rewrite |
| Competitor `.claude` paths | Never rewrite |
| `changelog/**`, `docs/*_PLAN.md`, `docs/plans/**` | Historical |
| `provider::jcode` / `JcodeProvider` product id | Display-only rename optional; module path product |

---

## Scan summary (2026-07-17)

| Bucket | Lines (approx) | Files | Action |
|---|---:|---:|---|
| **A** prod `env::var("JCODE_…")` | ~431 | **119** | → `product_env` / `product_env_os` |
| **B** `set_var`/`remove_var("JCODE_…")` | ~1183 | **157** | tests → `NEXT_CODE_*` (or dual-set); prod writers dual-set |
| **C** user-facing `"jcode"` / J-Code strings | hundreds | dozens | → next-code / Next Code |
| **D** `.rs` comments still saying jcode | large | many | mechanical comment pass |
| **E** `scripts/**` residual | large | **124** | installers first, then benches/CI |
| **F** `docs/**` non-plan residual | large | **66** | narrative rebrand (not plans) |
| **G** iOS display / type names | — | **40** | display + module rename; keep bundle id |
| **H** intentional allowlist | — | — | leave alone |

`product_env` today is only defined/used in:

- `crates/next-code-core/src/env.rs`
- `crates/next-code-storage/src/lib.rs`

Almost all other call sites still hardcode `JCODE_*`.

---

## Bucket A — raw `env::var("JCODE_…")` → `product_env`

**What:** Production (and misclassified test-adjacent) reads of `std::env::var("JCODE_X")` / `var_os` that should dual-read.

**How:**

```rust
// before
std::env::var("JCODE_HOME")
std::env::var_os("JCODE_THEME")

// after
next_code_core::env::product_env("HOME")
next_code_core::env::product_env_os("THEME")
```

For non-suffix dual pairs use `product_var_full("NEXT_CODE_…", "JCODE_…")`.

**Do not convert** inside `product_env` itself, dual-read assertions that *require* the legacy key, or allowlisted domain/host env names.

### Highest-density A targets

| Hits (approx) | Path |
|---:|---|
| 143 | `crates/next-code-base/src/config/env_overrides.rs` |
| 17 | `crates/next-code-provider-openrouter-runtime/src/lib.rs` |
| 13 | `src/cli/provider_init.rs` |
| 12 | `crates/next-code-base/src/auth/lifecycle.rs` |
| 10 | `crates/next-code-provider-bedrock/src/lib.rs` |
| 9 | `crates/next-code-base/src/provider_catalog.rs` |
| 7 | `crates/next-code-base/src/provider/openrouter.rs` |
| 7 | `crates/next-code-tui/src/tui/ui_header.rs` |
| 7 | `crates/next-code-app-core/src/server/util.rs` |
| 6 | `crates/next-code-build-meta/build.rs` |

### Full A file list (119)

```
crates/next-code-app-core/src/agent.rs
crates/next-code-app-core/src/agent/utils.rs
crates/next-code-app-core/src/external_auth.rs
crates/next-code-app-core/src/perf.rs
crates/next-code-app-core/src/prompt_templates.rs
crates/next-code-app-core/src/sandbox.rs
crates/next-code-app-core/src/scoped_models.rs
crates/next-code-app-core/src/server/client_session_tests/reload.rs
crates/next-code-app-core/src/server/client_session_tests/resume.rs
crates/next-code-app-core/src/server/debug_command_exec.rs
crates/next-code-app-core/src/server/debug_swarm_read.rs
crates/next-code-app-core/src/server/jade_relay.rs
crates/next-code-app-core/src/server/reload.rs
crates/next-code-app-core/src/server/socket.rs
crates/next-code-app-core/src/server/util.rs
crates/next-code-app-core/src/session_launch.rs
crates/next-code-app-core/src/tool/bash.rs
crates/next-code-app-core/src/tool/discover.rs
crates/next-code-app-core/src/tool/ffs_support/backend.rs
crates/next-code-app-core/src/tool/selfdev/mod.rs
crates/next-code-app-core/src/tool/selfdev/status.rs
crates/next-code-app-core/src/update.rs
crates/next-code-base/src/auth/copilot.rs
crates/next-code-base/src/auth/cursor.rs
crates/next-code-base/src/auth/gemini.rs
crates/next-code-base/src/auth/lifecycle.rs
crates/next-code-base/src/auth/oauth.rs
crates/next-code-base/src/config/env_overrides.rs
crates/next-code-base/src/disable.rs
crates/next-code-base/src/gmail.rs
crates/next-code-base/src/hooks.rs
crates/next-code-base/src/mcp/trust.rs
crates/next-code-base/src/model_pricing.rs
crates/next-code-base/src/process_memory.rs
crates/next-code-base/src/provider/activation.rs
crates/next-code-base/src/provider/anthropic.rs
crates/next-code-base/src/provider/catalog_routes.rs
crates/next-code-base/src/provider/mod.rs
crates/next-code-base/src/provider/openrouter.rs
crates/next-code-base/src/provider/selection.rs
crates/next-code-base/src/provider/startup.rs
crates/next-code-base/src/provider_catalog.rs
crates/next-code-base/src/runtime_memory_log.rs
crates/next-code-base/src/session.rs
crates/next-code-base/src/todo.rs
crates/next-code-build-meta/build.rs
crates/next-code-build-support/src/paths.rs
crates/next-code-desktop/src/desktop_benchmark.rs
crates/next-code-desktop/src/desktop_benchmarks_transcript.rs
crates/next-code-desktop/src/desktop_config.rs
crates/next-code-desktop/src/desktop_issue_cache.rs
crates/next-code-desktop/src/desktop_log.rs
crates/next-code-desktop/src/desktop_prefs.rs
crates/next-code-desktop/src/desktop_profiling.rs
crates/next-code-desktop/src/session_data.rs
crates/next-code-desktop/src/session_launch.rs
crates/next-code-desktop/src/session_launch/terminal.rs
crates/next-code-desktop/src/single_session_render/body_viewport.rs
crates/next-code-hooks/src/config.rs
crates/next-code-logging/src/lib.rs
crates/next-code-plugin-core/src/config.rs
crates/next-code-plugin-runtime/src/server.rs
crates/next-code-provider-anthropic-runtime/src/lib.rs
crates/next-code-provider-bedrock/src/lib.rs
crates/next-code-provider-claude-cli-runtime/src/lib.rs
crates/next-code-provider-copilot-runtime/src/lib.rs
crates/next-code-provider-core/src/auth_mode.rs
crates/next-code-provider-cursor-runtime/src/agent_transport.rs
crates/next-code-provider-cursor-runtime/src/lib.rs
crates/next-code-provider-doctor/src/lifecycle_driver.rs
crates/next-code-provider-doctor/src/live_provider_probes.rs
crates/next-code-provider-gemini-runtime/src/lib.rs
crates/next-code-provider-openai-runtime/src/lib.rs
crates/next-code-provider-openrouter-runtime/src/lib.rs
crates/next-code-provider-openrouter/src/lib.rs
crates/next-code-provider-service/src/policy.rs
crates/next-code-setup-hints/src/lib.rs
crates/next-code-telemetry-core/src/lib.rs
crates/next-code-telemetry-core/src/state_support.rs
crates/next-code-terminal-launch/src/lib.rs
crates/next-code-tui-mermaid/build.rs
crates/next-code-tui-mermaid/src/mermaid_runtime.rs
crates/next-code-tui-style/src/color.rs
crates/next-code-tui-workspace/src/color_support.rs
crates/next-code-tui/src/tui/app.rs
crates/next-code-tui/src/tui/app/auth.rs
crates/next-code-tui/src/tui/app/commands.rs
crates/next-code-tui/src/tui/app/commands_review.rs
crates/next-code-tui/src/tui/app/debug_cmds.rs
crates/next-code-tui/src/tui/app/helpers.rs
crates/next-code-tui/src/tui/app/inline_interactive.rs
crates/next-code-tui/src/tui/app/navigation.rs
crates/next-code-tui/src/tui/app/remote/server_events.rs
crates/next-code-tui/src/tui/app/state_ui_maintenance.rs
crates/next-code-tui/src/tui/app/tui_lifecycle_runtime.rs
crates/next-code-tui/src/tui/app/tui_state.rs
crates/next-code-tui/src/tui/backend.rs
crates/next-code-tui/src/tui/remote_diff.rs
crates/next-code-tui/src/tui/theme_detect.rs
crates/next-code-tui/src/tui/ui/profile.rs
crates/next-code-tui/src/tui/ui_frame_metrics.rs
crates/next-code-tui/src/tui/ui_header.rs
evals/jbench/src/agent_runner.rs
evals/next-code-edit-bench/src/runner.rs
src/bin/tui_bench.rs
src/cli/auth_test/choice.rs
src/cli/auth_test/run.rs
src/cli/commands.rs
src/cli/commands/menubar.rs
src/cli/dispatch.rs
src/cli/hot_exec.rs
src/cli/provider_doctor.rs
src/cli/provider_init.rs
src/cli/selfdev.rs
src/cli/startup.rs
src/cli/terminal.rs
src/extension_policy.rs
src/model_failover.rs
src/theme.rs
```

**Tooling:** `scripts/rebrand/rewrite_env_tokens.py` (report mode by default; optional apply for simple suffix patterns).

---

## Bucket B — `set_var` / `remove_var("JCODE_…")`

**Counts:** ~1183 call sites across **157** files (mostly tests). Only ~22 call sites already use `NEXT_CODE_*`.

### Policy

| Context | Rewrite |
|---|---|
| `#[cfg(test)]` / `*_tests.rs` / `tests/**` | `JCODE_X` → `NEXT_CODE_X` for set/remove/var used only to drive product code that now prefers `NEXT_CODE_*` |
| Dual-read unit tests of `product_env` itself | **KEEP** explicit `JCODE_*` (assert fallback) |
| Production **writers** that inject env for child processes | Dual-set both `NEXT_CODE_*` and `JCODE_*` during compat window, or set only `NEXT_CODE_*` if children use `product_env` |

### Top B files by set/remove count

```
55  crates/next-code-base/src/provider_catalog_tests.rs
42  crates/next-code-app-core/src/server/socket_tests.rs
40  crates/next-code-base/src/config_tests.rs
39  crates/next-code-base/src/auth/external_tests.rs
33  crates/next-code-tui/src/tui/app/tests/remote_events_reload_04.rs
30  crates/next-code-base/src/provider_catalog.rs
30  crates/next-code-app-core/src/prompt_templates.rs
28  src/cli/provider_init_tests.rs
25  crates/next-code-base/src/prompt_tests.rs
25  crates/next-code-app-core/src/agent_tests.rs
```

Full set-file inventory: 157 paths (see scan via `rg -l 'set_var\("JCODE_|remove_var\("JCODE_' -g '*.rs'`).

---

## Bucket C — user-facing strings (help / errors / TUI / desktop)

Priority product surfaces still saying `jcode`:

| Path | Notes |
|---|---|
| `src/main.rs` | Panic banner `*** jcode PANIC ***` |
| `src/crash_log.rs` | Resume hint `jcode --resume` |
| `crates/next-code-update-core/src/lib.rs` | Release asset names `jcode-linux-*` / `jcode-macos-*` |
| `crates/next-code-desktop/src/desktop_log.rs` | `jcode-desktop:` log prefix + log filenames |
| `crates/next-code-desktop/src/session_launch/server_io.rs` | bail messages “jcode server …” |
| `crates/next-code-desktop/src/desktop_config.rs` | desktop warnings / timing labels |
| `crates/next-code-telemetry-core/src/lib.rs` | onboarding telemetry copy + thread name |
| `crates/next-code-app-core/src/session_rebuild.rs` | “Rebuilding jcode…” |
| `src/cli/debug.rs` | `~/.jcode/config.toml` help |
| `src/cli/commands.rs` | gateway help path `~/.jcode` |
| `src/cli/provider_init.rs` | CLI value `"jcode"` for provider id (**product id — careful**) |
| `crates/next-code-provider-doctor/src/provider_e2e.rs` | login hints `jcode login --provider …` |
| `src/bin/harness.rs` | `#[command(name = "jcode-harness")]` |
| `crates/next-code-swarm-core/src/team/layout.rs` | `TEAM_SESSION_PREFIX = "jcode-team-"` |
| `src/theme.rs` / `src/skill_distillation.rs` | eprintln prefixes |

**Provider product id** (`"jcode"`, `ProviderChoice::Jcode`, `JcodeProvider`) is **not** a mechanical string mop — decide separately whether the stable auth id renames.

---

## Bucket D — comments / module docs in `.rs`

Large volume of `// … jcode …` and `/// … ~/.jcode …` after crate renames. Safe mechanical pass after A/B/C; low runtime risk. Prefer:

- `jcode` → `next-code` in prose
- `~/.jcode` → `~/.next-code` (mention dual-read only in storage docs)
- crate names `jcode-foo` → `next-code-foo`

Skip lines matching allowlist (dual-read, formerly jcode, domains, bundle id).

---

## Bucket E — `scripts/**` residual (**124 files**)

### Ship-critical first

```
scripts/install.sh
scripts/install.ps1
scripts/install_release.sh
scripts/uninstall.sh
scripts/quick-release.sh
scripts/update_packages.sh
scripts/lib/configure_path.sh
scripts/phone-server/units/next-code-serve.service
scripts/phone-server/next-code-pair-service.py
```

### Still named `jcode_*` (rename or dual)

```
scripts/jcode_harbor_agent.py
scripts/jcode_harbor_claude_agent.py
scripts/jcode_memory_snapshot.py
scripts/jcode_monitor.py
```

### Rest

Benches, e2e helpers, budgets JSON, remote build, onboarding sandbox, CI suites — rewrite `JCODE_*` → `NEXT_CODE_*` and bare binary `jcode` → `next-code` with installer alias notes.

**Exclude from mechanical rewrite:** `scripts/rebrand/*` (tooling), allowlisted domain strings if any.

Full list: 124 paths under `scripts/` matching `jcode|JCODE|J-Code|Jcode` (see scan).

---

## Bucket F — `docs/**` non-plan residual (**66 files**)

Narrative / reference docs still say jcode. **Out of scope for this pass:** `docs/*_PLAN.md`, `docs/plans/**`, `docs/REBRAND_*`.

High-traffic user docs first:

```
docs/CONFIG_REFERENCE.md
docs/WINDOWS.md
docs/WINDOWS_SETUP.md
docs/HOOKS.md
docs/IOS_APP.md
docs/AUTH_CREDENTIAL_SOURCES.md
docs/AWS_BEDROCK_PROVIDER.md
docs/PROVIDER_DOCTOR.md
docs/SERVER_ARCHITECTURE.md
docs/MEMORY_ARCHITECTURE.md
docs/DESKTOP_APP_ARCHITECTURE.md
docs/ONBOARDING_SANDBOX.md
docs/plugins.md
docs/plugins/api-reference.md
docs/plugins/README.md
```

Full non-plan list (66): see scan output under `docs/` excluding `*_PLAN.md`, `plans/`, `REBRAND_*`.

---

## Bucket G — iOS display strings (**40 files**, not bundle id)

| Keep forever | Rewrite |
|---|---|
| `com.jcode.mobile` | Display name → “Next Code” |
| dual URL scheme `jcode` | Prefer `nextcode` + keep `jcode` |
| `com.jcode.mobile.pair` / pair URL name as needed | Marketing strings, log prefixes |

### Structural (later PR — not pure string mop)

- `JCodeKit` → `NextCodeKit` (module/package rename)
- `JCodeMobile` → `NextCodeMobile`
- `ios/Package.swift`, `ios/project.yml` type names

### Display-only now

- Comments “jcode server” → “next-code server”
- `jcode-servers.json` → `next-code-servers.json` (with migrate if needed)
- TestHarness README / reward docs
- UserDefaults key `jcode.device.id` → `next-code.device.id` (migrate)

Files (40): `ios/Package.swift`, `ios/project.yml`, all `ios/Sources/**`, `ios/Tests/**`, `ios/TestHarness/**` matching residual scan.

---

## Bucket H — intentional allowlist hits (leave alone)

| Pattern | Why |
|---|---|
| `com.jcode.mobile` | App Store bundle id |
| `*.jcode.sh` / `jcode.sh` | Domain freeze |
| `claude-cli`, `codex_cli_rs` | Third-party UA |
| `.claude/`, `.codex/`, `.cursor/` | Competitor paths |
| `1jehuang/jcode` | Historical upstream |
| `jbench` | Unbranded eval |
| `jcode_dir`, `migrated-from-jcode` | Compat symbols |
| `product_env` body reading `JCODE_` | Dual-read implementation |
| Dual-read tests setting both keys | Correctness |
| `provider::jcode` / `JcodeProvider` / CLI `"jcode"` provider id | Product provider surface |
| `LEGACY_SERVICE_NAME = "jcode-secrets"` etc. | Keyring dual-read |
| `changelog/**`, plans, `docs/REBRAND_*` | Historical / process |
| Dual URL scheme registration of `jcode` | Deep-link migration |

Gate: `python3 scripts/rebrand/rg_gate.py` — debt meter until mop finishes.

---

## Recommended mechanical order

```
1. Tooling ready
   - scripts/rebrand/rewrite_env_tokens.py  (this mop)
   - dry-run report for A/B before any write

2. Bucket A (product correctness) — high density first
   a. env_overrides.rs  (bulk product_env("…"))
   b. provider_init + provider runtimes + auth/lifecycle
   c. remaining src/cli/* and app-core server paths
   d. cargo check -p next-code -p next-code-base -p next-code-app-core

3. Bucket B (tests + prod writers)
   a. Mechanical set_var/remove_var JCODE_ → NEXT_CODE_ in *test* modules
   b. KEEP dual-read tests in next-code-core/env.rs and storage
   c. Prod writers: dual-set or NEXT_CODE-only if readers use product_env
   d. cargo test -p next-code-storage --lib
   e. targeted cargo test for provider_init / config / socket

4. Bucket C (user-facing) — small, high signal
   a. panic / crash_log / update asset names / desktop bail strings
   b. help paths ~/.jcode → ~/.next-code
   c. harness bin name, team session prefix, log prefixes
   d. DEFER provider id "jcode" product decision

5. Bucket E ship path
   a. install.sh / install.ps1 / uninstall / release
   b. rename scripts/jcode_*.py or leave filename + rewrite contents
   c. CI/bench scripts JCODE_ → NEXT_CODE_

6. Bucket G iOS display (bundle id untouched)
   a. strings + dual scheme verify
   b. optional module rename JCode* → NextCode* in follow-up

7. Bucket D comments + Bucket F docs
   a. mechanical comment pass on .rs
   b. user-facing docs (CONFIG_REFERENCE, WINDOWS, HOOKS, …)
   c. leave plans/changelog/REBRAND_* alone

8. Measure
   python3 scripts/rebrand/rg_gate.py --max-print 50
   Update REBRAND_STATUS.md hit counts
```

### Safety rules for every pass

1. Never rewrite `com.jcode.mobile` or `*.jcode.sh`.
2. Never strip dual-read from `product_env` / `next_code_dir` / keyring / URL schemes.
3. Prefer `product_env("X")` over raw `NEXT_CODE_X`-only reads in production.
4. Tests that only *inject* env should set `NEXT_CODE_*` once product code dual-reads.
5. Do not expand allowlist to silence product strings — fix the string.
6. After each bulk pass: `cargo check -p next-code` + relevant `cargo test`.

---

## Tooling

| Script | Role |
|---|---|
| `scripts/rebrand/rewrite_strings.py` | Broad string/path rebrand (already used) |
| `scripts/rebrand/rewrite_env_tokens.py` | **Safer env token rewrite / report** (A/B) |
| `scripts/rebrand/rg_gate.py` | Residual gate / debt meter |
| `scripts/rebrand/rewrite_cargo.py` / `rewrite_rust_idents.py` | Done for structure |

### `rewrite_env_tokens.py` modes

```bash
# Report only (default)
python3 scripts/rebrand/rewrite_env_tokens.py --report

# Apply test-module set_var/remove_var JCODE_ → NEXT_CODE_
python3 scripts/rebrand/rewrite_env_tokens.py --apply-test-setters

# Convert simple production reads to product_env (suffix pattern only)
python3 scripts/rebrand/rewrite_env_tokens.py --apply-product-env --dry-run
python3 scripts/rebrand/rewrite_env_tokens.py --apply-product-env
```

Never touches: `com.jcode.mobile`, `*.jcode.sh`, dual-read impl in `next-code-core/src/env.rs` (except reporting).

---

## Top 10 highest-value file targets

Ordered by user-visible impact + env density + ship risk:

1. `crates/next-code-base/src/config/env_overrides.rs` — 143 raw `JCODE_*` reads; one-file bulk `product_env`
2. `src/cli/provider_init.rs` — runtime provider activation env (read + set)
3. `crates/next-code-update-core/src/lib.rs` — release asset names must match GitHub releases
4. `src/main.rs` — panic banner / thread names
5. `scripts/install.sh` (+ `install.ps1`, `uninstall.sh`) — user install path / binary name
6. `crates/next-code-desktop/src/session_launch/server_io.rs` — desktop error surface
7. `crates/next-code-base/src/auth/lifecycle.rs` — auth env dual-read
8. `src/crash_log.rs` — crash resume instructions
9. `crates/next-code-provider-openrouter-runtime/src/lib.rs` — high env density provider path
10. `ios/Sources/JCodeMobile/Info.plist` + display strings — dual scheme verify; **keep** `com.jcode.mobile`

Honorable mentions: `src/cli/provider_init_tests.rs` (B density), `crates/next-code-telemetry-core/src/lib.rs` (onboarding copy), `docs/CONFIG_REFERENCE.md` (user docs).

---

## Out of scope / deferred decisions

| Item | Owner decision |
|---|---|
| Rename first-party provider id `jcode` → `next-code` | Auth state compatibility |
| App Store bundle id migration | Phase 6 |
| DNS off `*.jcode.sh` | Infra |
| Compat dual-read removal | Contract §2.5 TODO version |
| Filename renames `scripts/jcode_*.py` | Optional; content first |

---

## Re-measure commands

```bash
# A
rg -n 'env::var(_os)?\("JCODE_' -g '*.rs' -g '!target/**' | wc -l

# B
rg -n 'set_var\("JCODE_|remove_var\("JCODE_' -g '*.rs' -g '!target/**' | wc -l

# product_env adoption
rg -n 'product_env(_os)?' -g '*.rs' -g '!target/**' -c

# Gate
python3 scripts/rebrand/rg_gate.py --max-print 30

# Env rewrite report
python3 scripts/rebrand/rewrite_env_tokens.py --report
```
