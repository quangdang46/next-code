# Plan Report

## Summary (read this first)
- **You asked:** Merge PR6 and continue the next brick (PR7).
- **What is going on:** SUMMARY PR7 says “copy entire `xai-grok-pager` + delete old TUI”. Upstream pager is **~433 `.rs` / ~374k LOC** and still depends on **10 crates we do not have** (plus ratatui package renames and deeper stubs). Deleting `next-code-tui*` in the same PR leaves next-code without a working UI until PR8 wiring lands.
- **We recommend:** **Option C (vendor Face, keep brain UI until PR8)** — add workspace crate `xai-grok-pager` by copying upstream sources; add **narrow compile stubs** for the 10 missing deps; point ratatui deps at `next-code-ratatui-*` via `package =`; voice `default-features = false` / no `audio`. **Do not** delete `next-code-tui*` or switch `src/main.rs` in PR7. Success bar: `cargo check -p xai-grok-pager`.
- **Risk:** **High** (size + stub churn; expect several compile-fix loops)
- **Status:** Waiting for your OK — reply **go ahead** to implement

## Feature planning
- **Recommended approach (Option C):**
  1. Branch `pr-7-grok-pager-vendor` from `dev` (PR6 already merged @ `d71369f22`).
  2. Copy `grok-build/crates/codegen/xai-grok-pager` → `crates/xai-grok-pager` (keep Cargo name `xai-grok-pager`; Apache headers + NOTICE).
  3. Adapt `Cargo.toml`: workspace path deps to existing Face crates; `xai-ratatui-*` → `next-code-ratatui-*` with `package =` (same pattern as pager-render); `xai-grok-voice` **without** `audio` feature; drop or soft-gate bins/tests that need `xai-grok-pager-bin` / pty-harness if they block lib check.
  4. Add **10 missing stub/façade crates** (pager import sites only — not full vendor of mermaid/update/fast-worktree):
     | Crate | Upstream size | PR7 strategy |
     |-------|---------------|--------------|
     | `xai-grok-version` | ~66 LOC | vendor/adapt tiny |
     | `xai-prompt-queue` | ~164 LOC | vendor/adapt tiny |
     | `xai-token-estimation` | ~228 LOC | vendor/adapt tiny |
     | `xai-hooks-plugins-types` | ~1.1k LOC | vendor types or stub used enums/structs |
     | `xai-crash-handler` | ~1.5k | narrow façade for wrap_restore sites |
     | `xai-grok-mermaid` | ~1.8k | stub / no-op render (or thin vendor if cheap) |
     | `xai-grok-sandbox` | ~3.5k | stub types pager render needs |
     | `xai-grok-plugin-marketplace` | ~5k | stub CTA/types |
     | `xai-grok-update` | ~5.4k | stub welcome/update checks |
     | `xai-fast-worktree` | ~16k / 35 rs | **façade only** for `worktree_cmd` symbols |
  5. Grow existing stubs (`xai-grok-shell`, `tools`, `workspace`, `telemetry`, …) only as `cargo check -p xai-grok-pager` fails — iterative, smallest symbols.
  6. **Out of PR7:** delete `next-code-tui*`; change `src/main.rs` / CLI; implement `GrokHost` (PR8); real mic/STT/GCS/upload.
- **Why not SUMMARY literal “delete old TUI” in PR7:** product would ship neither Face nor brain UI until PR8. Keep dual crates; cut over in PR8 after check + smoke.
- **Option A (avoid):** wholesale vendor all 10 missing crates + delete TUI in one PR — weeks of noise, high break risk.
- **Option B:** only copy pager sources without missing stubs — will not compile.
- **Prior art:** Local `grok-build` @ ba69d70; PR2–6 Face pattern (narrow stubs, `xai-*` names, NOTICE/LICENSE).
- **Integration points:** new `crates/xai-grok-pager` + stub crates + workspace members; **no** `next-code-agent-runtime` / entrypoint wiring.
- **Sub-agents used:** skipped (inventory verified directly against grok-build + workspace).
- **Open questions (defaults if go ahead):**
  1. Keep Cargo name `xai-grok-pager` (not `next-code-tui-pager`) ✅
  2. Do **not** delete old TUI in PR7 ✅
  3. Stub missing deps narrowly; grow on compile errors ✅
  4. Success = `cargo check -p xai-grok-pager` (tests optional / later) ✅

## Evidence
1. **Pager size:** `grok-build/.../xai-grok-pager/src` → **433** `.rs`, **~374k** lines (Measured 2026-07-20).
2. **Missing workspace deps (HAVE/NEED):** version, prompt-queue, mermaid, update, token-estimation, fast-worktree, plugin-marketplace, hooks-plugins-types, sandbox, crash-handler — **MISSING**. Ratatui = **ALIAS** via `next-code-ratatui-*`. PR2–6 Face crates **HAVE**.
3. **Pager import pressure (file counts):** e.g. `xai_hooks_plugins_types` ~20 files, `xai_ratatui_textarea` ~22, `xai_grok_update` ~5, `xai_fast_worktree` ~4.
4. **Old TUI still live:** `crates/next-code-tui` (~248 rs) + many `next-code-tui-*`; binary still `run_main` path — deleting in PR7 would orphan the product UI.
5. **SUMMARY §PR7 / mermaid chart** still says “Pager crate + old TUI delete” — plan updates that to “vendor pager; TUI delete deferred to PR8”.

## Steps
1. [ ] Branch from `dev`
2. [ ] Add stub crates for the 10 missing deps (tiny first; façades for large ones)
3. [ ] Copy pager → `crates/xai-grok-pager`; adapt Cargo.toml + NOTICE
4. [ ] Iterate `cargo check -p xai-grok-pager` until green (stub growth)
5. [ ] Update `docs/grok-migration-SUMMARY.md` PR7 wording (no TUI delete here)
6. [ ] PR → `dev`, Refs #35 — **do not** switch binary yet

## Files to touch (expected)
- `crates/xai-grok-pager/**` (large)
- `crates/xai-grok-version/**`, `xai-prompt-queue/**`, `xai-token-estimation/**`, `xai-hooks-plugins-types/**`, `xai-crash-handler/**`, `xai-grok-mermaid/**`, `xai-grok-sandbox/**`, `xai-grok-plugin-marketplace/**`, `xai-grok-update/**`, `xai-fast-worktree/**`
- Existing Face stubs as needed (`xai-grok-shell`, `telemetry`, …)
- `Cargo.toml` / `Cargo.lock`
- `docs/grok-migration-SUMMARY.md`
- `docs/plans/PLAN-20260720-grok-pr7-pager-vendor.md` (this file)

## If you want more detail
- Expect **multiple commits** on the branch (stubs → copy → compile fixes).
- `xai-grok-telemetry` already exists but is a thin no-op; pager will force many more symbols — grow in place, still no real OTLP.
- PR8 remains: entrypoint + `GrokHost` + then remove / stop shipping old `next-code-tui` path.
