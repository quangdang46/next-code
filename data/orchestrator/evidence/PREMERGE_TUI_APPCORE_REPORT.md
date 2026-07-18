# Pre-merge focused checks: tui + app-core + provider-metadata

| Field | Value |
| --- | --- |
| Branch | `orch/premerge/check-tui` |
| HEAD | `00428e461` — *refactor: remove residual next-code hosted subscription surface* |
| Date (UTC) | 2026-07-18T05:11:31Z |
| Env | `NEXT_CODE_NO_TELEMETRY=1` |
| Toolchain | `rustc 1.98.0-nightly` / `cargo 1.98.0-nightly` |
| Worktree | read-only checks only (no commit / push / merge) |
| Logs | `check-logs/` |

## Commands and exit codes

| # | Command | Exit | Result |
| --- | --- | --- | --- |
| 1 | `cargo check -p next-code-app-core -p next-code-tui -p next-code-provider-metadata -p next-code --bin next-code` | **0** | `Finished dev` in ~5m 58s |
| 2 | `cargo test -p next-code-provider-metadata --lib -- --test-threads=1` | **0** | **14 passed**, 0 failed |
| 3 | `cargo test -p next-code-tui --lib onboarding_welcome -- --test-threads=2` | **0** | **4 passed**, 0 failed (1815 filtered) |
| 4 | `cargo test -p next-code-tui --lib test_top_level_command_suggestions_include_config -- --test-threads=1` | **0** | **1 passed**, 0 failed (1818 filtered) |

Log files:

- `check-logs/cargo-check.log`
- `check-logs/test-provider-metadata.log`
- `check-logs/test-tui-onboarding-welcome.log`
- `check-logs/test-tui-top-level-config.log`

## Per-command detail

### 1. `cargo check` (app-core, tui, provider-metadata, next-code bin)

- **Exit:** 0
- **Failures:** none
- **Classification:** clean typecheck/link for all four targets
- **Warnings only (non-blocking):**
  - `next-code-tui`: many `dead_code` / unused-import warnings (UI helpers, hotkey feedback, swarm panel, todo panel, etc.) — pre-existing hygiene noise, not compile blockers
  - `next-code-app-core`: 2× deprecated `next_code_base::hooks::{hook_configured,dispatch_observer}` in `turn_execution.rs`
  - `next-code-base`: unused import `ModelRoute`
  - future-incompat note for `proc-macro-error2 v2.0.1` (transitive)

### 2. `next-code-provider-metadata` lib tests

- **Exit:** 0
- **14/14 ok**, including login-matrix uniqueness / alias resolution tests:
  - `matrix_cli_login_selection_preserves_existing_order`
  - `matrix_login_provider_aliases_resolve_to_canonical_ids`
  - `matrix_login_provider_ids_and_aliases_are_unique`
  - `matrix_tui_login_selection_supports_numbers_and_names`
  - `resolve_login_provider_loose_*`
  - provider profile/config tests (ollama, nvidia nim, minimax, cerebras, alibaba, normalize_api_base)
- **Failures:** none
- **Subscription / hosted login:** tests **assert removal**, not presence:
  - `assert!(!ids.contains(&"next-code"), "hosted next-code login provider must stay removed…")`
  - comments: *“Hosted next-code/subscription login was removed”*
  - `resolve_login_provider("subscription")` is exercised as a non-hosted path (must not resurrect hosted product login)

### 3. TUI `onboarding_welcome` filter

- **Exit:** 0
- **4/4 ok:**
  - `onboarding_welcome_centers_within_tall_area`
  - `onboarding_welcome_login_suggestion_shows_typed_command`
  - `onboarding_welcome_renders_on_tiny_area_without_panicking`
  - `onboarding_welcome_shows_title_and_suggestions`
- **Failures:** none
- No residual expectation of `/subscription` or hosted next-code product login in these tests.

### 4. TUI top-level command suggestions (config)

- **Exit:** 0
- **1/1 ok:** `test_top_level_command_suggestions_include_config_and_alignment`
- **Failures:** none
- Nearby comment in suite: *“Hosted /subscription command was removed with the next-code login provider.”* — aligned with cleanup; test does not require `/subscription`.

## Residual `/subscription` / hosted-login scan (source)

Scoped to `next-code-tui`, `next-code-app-core`, `next-code-provider-metadata`, `next-code` sources:

| Hit class | Verdict |
| --- | --- |
| Swarm **channel_subscriptions** in app-core server | Unrelated (multi-agent channels), not product billing |
| Provider menu_detail *“requires Claude Pro/Max subscription”* / ChatGPT Plus | Third-party provider copy, expected |
| CLI remediation `next-code login --provider <name>` | Multi-provider CLI login (kept); not hosted product login |
| Metadata/TUI comments + asserts that hosted `next-code` login / `/subscription` **must stay removed** | Cleanup verified, not a regression |
| OpenAI usage JSON field `subscription_type` | External API field, not product surface |

**No failing or still-required test expects `/subscription` or hosted next-code login.**

## Package-level merge recommendation

| Package | Focused check status | Merge-safe for this slice? |
| --- | --- | --- |
| `next-code-provider-metadata` | check ✅ · lib tests 14/14 ✅ | **Yes** |
| `next-code-app-core` | check ✅ (no focused lib tests requested) | **Yes** for compile surface |
| `next-code-tui` | check ✅ · onboarding_welcome 4/4 ✅ · top-level config 1/1 ✅ | **Yes** for these paths |
| `next-code` (bin) | check ✅ | **Yes** for link/typecheck |

### DONE

- All requested commands exited **0**.
- **No test failures** under the rebrand/telemetry/subscription cleanup filters.
- Hosted next-code login and `/subscription` product surface remain intentionally removed (asserted by metadata tests; not demanded by TUI tests).
- **Recommendation: packages look merge-safe** for this focused tui + app-core + provider-metadata pre-merge gate.
- Caveats (non-blocking): large dead_code warning surface in TUI; deprecated hook APIs in app-core; full app-core/tui lib suites and integration/e2e were **not** run here.

No commit, push, or merge performed.
