# Pre-merge residual surface scan

Branch: `orch/premerge/check-residual`  
Date: 2026-07-18  
Scope: jcode branding, desktop/mobile product crates, product telemetry phone-home, hosted next-code subscription login.  
Exclusions: `target/`, `.git/`, demo timeline JSON noise.

## Summary

| Surface | Status |
|---|---|
| `jcode` / `.jcode` | **Clean** (0 hits) |
| `next-code-desktop` / `next-code-mobile` product crates | **Clean** (no crates / workspace members) |
| Product telemetry core (`crate::telemetry::`, `telemetry_stub`, `TELEMETRY.md`, `next-code-telemetry-core`) | **Runtime clean**; stale doc + installer funnel residual |
| Hosted next-code subscription login (`LOGIN_PROVIDERS`, `/login next-code`, `/subscription`) | **Runtime clean**; negative tests + stale docs residual |

**Overall residual risk:** `mostly-clean`  
**Merge blocker?** `no`

No code edits made. No residual test still asserts that hosted next-code login *exists*; remaining tests assert removal (`None` / `!ids.contains("next-code")` / `!/subscription`).

---

## Hit table

| Path | Snippet | KEEP / CLEAN | Why |
|---|---|---|---|
| *(repo-wide)* | `jcode` / `.jcode` | **CLEAN** | Zero matches under source/docs/scripts (excluding target/git). |
| `crates/` workspace | no `next-code-desktop` / `next-code-mobile` | **CLEAN** | No product desktop/mobile crates; only unrelated “desktop control” wording for `macos_computer_use`. |
| `assets/demos/*timeline*.json` | historical `next-code-mobile-design-spec` in demo bash output | **KEEP** | Demo timeline noise; excluded by scan intent; not a live product surface. |
| `crates/next-code-provider-metadata/src/catalog.rs` | `LOGIN_PROVIDERS: [LoginProviderDescriptor; 46]` — no `id: "next-code"` | **KEEP** | Hosted provider absent from catalog. |
| `crates/next-code-provider-metadata/src/lib.rs` | `LoginProviderTarget` has no NextCode/Subscription variant | **KEEP** | Runtime identity enum does not expose hosted product login. |
| `crates/next-code-provider-metadata/src/lib.rs` (~503, ~631–670) | `resolve_login_provider("next-code")` → `None`; `!ids.contains(&"next-code")` | **KEEP** | Negative regression tests locking removal. |
| `crates/next-code-base/src/provider_catalog_tests.rs` (~54, ~431–436) | same `None` / must-stay-removed asserts | **KEEP** | Mirrors metadata tests; asserts absence. |
| `crates/next-code-tui/src/tui/app/tests/state_model_poke_02/part_01.rs:799` | `assert!(!… cmd == "/subscription")` | **KEEP** | Confirms hosted `/subscription` slash command is gone. |
| `src/cli/provider_init.rs:1215–1226` | `disable_subscription_runtime_mode()` no-op comment: “Hosted subscription runtime was removed” | **KEEP** | Intentional stub so call sites stay compile-safe after purge. |
| Claude/OpenAI `subscription_type` fields / docs | e.g. `auth/claude_tests.rs`, `AUTH_CREDENTIAL_SOURCES.md` | **KEEP** | Provider OAuth subscription wording, not hosted product. |
| Binary / path name `next-code` | install paths, crate names, `next-code login` CLI strings | **KEEP** | Product binary name; out of residual scope. |
| `crates/next-code-selfdev-types/src/lib.rs:44` | `"tui" \| "next-code" => Ok(Self::Tui)` | **KEEP** | Build-target alias for the TUI binary name, not a login provider. |
| `crates/next-code-tui/src/tui/login_picker.rs:636` | `"next-code" => Color::Rgb(…)` in `provider_style` | **KEEP** (dead arm) | Unreachable color arm after provider removal; cosmetic only, not a login surface. |
| `crates/next-code-app-core/src/server/client_state.rs:70` | `"next-code" => "Next Code".to_string()` display label | **KEEP** (dead arm / display) | Label helper for provider key; no login registration; harmless if key never appears. |
| `scripts/install.sh:38–57` | `curl … https://telemetry.next-code.sh/v1/event` install funnel | **CLEAN?** → residual note | Still product phone-home from installer (opt-out via `NEXT_CODE_NO_TELEMETRY` / `DO_NOT_TRACK`; only when conversion UUID set). Runtime `crate::telemetry` / `telemetry-core` already purged. Not a login merge blocker. |
| `scripts/test_install_conversion.sh` | mocks `telemetry.next-code.sh` | **KEEP** with install.sh | Tests installer funnel; co-travels with install script. |
| `TELEMETRY.md` | file missing | **CLEAN** | Public telemetry contract file gone. |
| `crates/next-code-telemetry-core/` | directory missing | **CLEAN** | Product telemetry crate gone. |
| `src/telemetry.rs` | file missing | **CLEAN** | Runtime telemetry module gone. |
| `crates/next-code-base/src/subscription_api.rs` | file missing | **CLEAN** | Hosted account client gone. |
| `crates/next-code-base/src/subscription_catalog.rs` | file missing | **CLEAN** | Hosted catalog gone. |
| `src/cli/login/next_code_device.rs` | file missing | **CLEAN** | Hosted device-login client gone. |
| `docs/dev/ACCOUNT_CONTRACT_CONFORMANCE_TESTS.md` | cites `subscription_api.rs`, `next_code_device.rs`, `/login next-code`, staging smoke | **CLEAN** (docs residual) | Stale design doc for removed hosted account contract; does not affect runtime. |
| `docs/dev/ACCOUNT_FLOWS_OBSERVABILITY_PRIVACY.md` | cites `next-code-telemetry-core`, `TELEMETRY.md`, `subscription_api`, `telemetry-worker` | **CLEAN** (docs residual) | Partially updated (“pipeline removed”) but still points at deleted paths. |
| `docs/CODE_QUALITY_AUDIT_2026-04-18.md` / `docs/CODE_QUALITY_TODO.md` | historical `src/telemetry.rs` LOC notes | **KEEP** | Point-in-time audit artifacts. |
| `scripts/swallowed_error_budget.json:949–954` | keys for deleted `subscription_api.rs` / `subscription_catalog.rs` | **CLEAN** (stale budget entries) | Dead budget keys; not a runtime surface. |
| `PLAN_PARITY.md:169` | “next-code has no product telemetry pipeline” | **KEEP** | Accurate product statement. |
| Widespread `NEXT_CODE_NO_TELEMETRY=1` in benches/e2e | env kill-switch | **KEEP** | Defensive opt-out; fine after purge. |

---

## Checklist detail

### 1. `jcode` / `.jcode`
- Full-tree content search (case-insensitive, excluding `target/` / `.git/`): **0 hits**.

### 2. Desktop / mobile product crates
- No workspace crate named `next-code-desktop` or `next-code-mobile`.
- No Cargo member / package name hits for those crates.
- Demo timeline JSON only embeds historical shell output mentioning a figma mobile design spec — noise, not a product crate.

### 3. Product telemetry
| Symbol / path | Present? |
|---|---|
| `crate::telemetry::` call sites | No |
| `telemetry_stub` | No |
| `TELEMETRY.md` | No |
| `next-code-telemetry-core` crate | No |
| `src/telemetry.rs` | No |
| Installer `telemetry.next-code.sh` | **Yes** (`scripts/install.sh`) |

Runtime product telemetry surface is purged. Remaining phone-home is install-funnel only (shell installer, gated, best-effort curl).

### 4. Hosted subscription / login identity
| Expectation | Result |
|---|---|
| `LOGIN_PROVIDERS` has no `next-code` entry | Pass |
| `resolve_login_provider("next-code")` is `None` | Pass (tests assert) |
| `resolve_login_provider("subscription")` is `None` | Pass (tests assert) |
| `/login next-code` / `/connect next-code` command registration | Not present as live registration |
| `/subscription` command | Asserted **absent** in TUI suggestion tests |
| `Next Code Subscription Status` string | Not found as live UI copy |
| Runtime provider key `"next-code" \| "next-code subscription"` | Not a login provider; only dead display/color arms + selfdev binary alias |
| Claude/OpenAI `subscription_type` | Present — **in-scope KEEP** |

### 5. `LOGIN_PROVIDERS`
- Defined in `crates/next-code-provider-metadata/src/catalog.rs`.
- `LoginProviderTarget` variants: AutoImport, Claude, ClaudeApiKey, OpenAi, OpenAiApiKey, OpenRouter, Bedrock, Azure, OpenAiCompatible, Cursor, Copilot, Gemini, Antigravity, Google — **no hosted next-code**.

---

## Residual risk assessment

**mostly-clean**

- **Runtime product identity** (login provider, slash commands, subscription API/catalog, telemetry crate/module): clean.
- **Residual non-runtime noise:**
  1. Stale account-contract / observability docs still describe removed hosted subscription + telemetry-core paths.
  2. `scripts/swallowed_error_budget.json` still lists deleted subscription modules.
  3. Dead match arms for `"next-code"` color/label in TUI/server helpers.
  4. Installer install-funnel POST to `telemetry.next-code.sh` (opt-outable product analytics).

None of these re-register hosted login or reintroduce jcode/desktop/mobile crates.

## Merge blocker?

**no**

Rationale: no live hosted next-code login provider, no `/subscription` command registration, no jcode branding, no desktop/mobile product crates, no in-process product telemetry pipeline. Remaining items are docs/stale budget keys/dead arms/installer funnel — follow-ups, not blockers.

## Edits made

None. (No test still expects hosted next-code login to resolve to `Some`.)

## Suggested non-blocking follow-ups (optional)

1. Archive or rewrite `docs/dev/ACCOUNT_CONTRACT_CONFORMANCE_TESTS.md` and trim dead paths from `docs/dev/ACCOUNT_FLOWS_OBSERVABILITY_PRIVACY.md`.
2. Drop deleted paths from `scripts/swallowed_error_budget.json`.
3. Decide whether installer funnel to `telemetry.next-code.sh` stays as intentional install analytics or should be removed with the product-telemetry purge.
4. Optionally delete dead `"next-code"` arms in `login_picker.rs` / `client_state.rs`.

---

## DONE

- **Residual risk:** mostly-clean  
- **Merge blocker:** no  
