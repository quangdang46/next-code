# Plan Report

## Summary (read this first)
- **You asked:** After PR4 merge, start the next brick (PR5: agent / shell / ACP).
- **What is going on:** Face (PR1–4) has render + config + tools/workspace compile stubs. Pager still needs ACP channels and huge `xai_grok_shell` / small `xai_grok_agent` import surfaces. SUMMARY’s `xai-shim-*` names would force mass renames; PR2–4 kept Cargo `xai-*`.
- **We recommend:** **Option C (narrow)** — vendor small `xai-acp-lib` (~8 rs), stub `xai-grok-agent` (modal/plugin types), façade-stub `xai-grok-shell` high-frequency modules (full `RemoteSettings`, clipboard re-export, config/session/auth DTOs). **Do not** wire `AcpAgentTx` → next-code runtime yet (that is PR8 / `GrokHost`). Keep Cargo names `xai-acp-lib` / `xai-grok-agent` / `xai-grok-shell`. Keep ACP id `enable-always-approve` (YOLO bridge later).
- **Risk:** Medium (shell stub surface is large; incomplete stubs → PR7 compile churn)
- **Status:** Implemented on `pr-5-grok-agent-shell-acp` (PR #39) — merge-ready; review: APPROVE WITH NITS

## Feature planning
- **Recommended approach:** Treat “shim” as a doc label only; keep package names matching pager `use xai_grok_*` / `xai_acp_lib`. Vendor ACP channel crate almost wholesale. Stub agent discovery/plugins. Grow shell as a compile façade (empty/no-op functions + Default DTOs), not a 14MB vendor.
- **Prior art (GitHub / local):** Local `grok-build` `SOURCE_REV` ba69d70 — `xai-acp-lib` 8 files ~88KB; `xai-grok-agent` ~30 rs; `xai-grok-shell` ~434 rs / ~14MB. next-code already has `agent-client-protocol` 0.10.4, `xai-grok-shared` clipboard, `xai-grok-config` home, `next-code-agent-runtime` (different schema — not drop-in).
- **Integration points:** new crates under `crates/`; workspace `Cargo.toml`; leave Face packages green
- **Sub-agents used:** yes (explore inventory)
- **Option A (skip):** docs-only — blocks PR7
- **Option B (avoid):** rename to `xai-shim-*` — forces pager import rewrite
- **Option D (defer):** vendor entire shell — too large for one PR
- **Open questions (defaults if you say go ahead):**
  1. Keep Cargo names `xai-*` (not `xai-shim-*`) ✅
  2. Vendor `xai-acp-lib` wholesale (adapt deps only) ✅
  3. Shell = frequency-ordered stubs, not full copy ✅
  4. Expand `RemoteSettings` under **shell** `util::config` (full Default fields); leave tiny PR3 config stub as-is or re-export carefully ✅
  5. No YOLO → `PermissionMode` remap in PR5 ✅
  6. No live next-code agent channel wiring ✅

## Evidence
1. **Pager imports:** `xai_acp_lib` = HEART (`AcpAgentTx`/`Rx`, message enums) across `acp/`, `app/`, `event_loop`. `xai_grok_shell` ≈653 hits / ≈115 files. `xai_grok_agent` ≈6 production sites (agents_modal + plugins).
2. **Installed ACP:** `agent-client-protocol` already in Face crates at 0.10.4 unstable.
3. **RemoteSettings risk:** pager types `xai_grok_shell::util::config::RemoteSettings`; PR3/4 only added `{ folder_trust_enabled }` on `xai-grok-config` — must not force pager onto that tiny type.
4. **PR4 review (merged):** ToolOutput variants 1:1, serde `tag=type`, permission ids, AskUserQuestion outcomes, KillOutcome, RestoreDegree match upstream. Deliberate gap: fuzzy `path: String` (upstream `Utf32String`) — OK until PR7 file-search.

## Steps
1. [x] Branch `pr-5-grok-agent-shell-acp` from `dev`
2. [x] Vendor `xai-acp-lib` (+ workspace member); `cargo check -p xai-acp-lib`
3. [x] Stub `xai-grok-agent` for pager modal/plugin symbols
4. [x] Façade `xai-grok-shell` for top import prefixes (util/config/clipboard/home, sampling, agent::config, auth DTOs, extensions, session types, active_sessions)
5. [x] `cargo check -p xai-acp-lib -p xai-grok-agent -p xai-grok-shell -p xai-grok-pager-render`
6. [x] Update SUMMARY; open PR → `dev`, Refs #35 (merge pending)

## Files to touch
- `crates/xai-acp-lib/**` (new, vendor)
- `crates/xai-grok-agent/**` (new, stubs)
- `crates/xai-grok-shell/**` (new, stubs)
- `Cargo.toml` (workspace members)
- `docs/grok-migration-SUMMARY.md`
- `docs/plans/PLAN-20260720-grok-pr5-agent-shell-acp.md` (this file)

## If you want more detail
Runtime wiring (`AcpAgentTx` → `next-code-app-core` / `GrokHost`) stays **PR8**. PR6 remains small stubs (voice / announcements / file-util). PR7 copies pager against these façades.
