# Plan: telemetry-purge-c
Date: 2026-07-18
Status: done
Problem: Hosted/product telemetry is already no-op stubbed; Option C removes all call sites, stub modules, dead state, and stale docs so next-code has zero telemetry surface.

## Work packages
1. **impl-appcore** (owner: implementer-appcore)
   - Paths: `crates/next-code-app-core/**`
   - Remove every `crate::telemetry::...` call site
   - Do NOT delete stub modules (owned by cleanup package)
2. **impl-tui** (owner: implementer-tui)
   - Paths: `crates/next-code-tui/**`
   - Remove every `crate::telemetry::...` / telemetry import call site
3. **impl-cli-base** (owner: implementer-cli-base)
   - Paths: `src/cli/**`, `crates/next-code-base/src/memory/**` (call sites only), not telemetry_stub.rs
4. **impl-cleanup** (owner: implementer-cleanup) — after 1–3 land or on same tree if solo
   - Delete `telemetry_stub.rs`, `telemetry_state.rs`, `telemetry_tests.rs`
   - Remove `pub mod telemetry` re-exports
   - Docs: TELEMETRY.md, docs refs, scripts NO_TELEMETRY dual junk, panic_budget/swallowed refs
   - Keep `ErrorCategory`/`SessionEndReason` via `next_code_usage_types` if still needed elsewhere

## Constraints
- Do not reintroduce network telemetry
- Keep CLI/auth/agent behavior identical
- Prefer deleting call lines over wrapping in `if false`
- Package-local `cargo check` before claiming done
- No commit/push unless asked

## Verification
- `cargo check -p next-code-app-core -p next-code-tui -p next-code --bin next-code` (or workspace as feasible)
- `rg 'crate::telemetry::|telemetry_stub|TELEMETRY\.md' ` should be empty (except historical changelog if left)

## Decision log
- User chose Option C (full purge of call sites + stub)
- macOS .app launcher kept (unrelated)
