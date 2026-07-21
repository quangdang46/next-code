# Plan Report

## Summary (read this first)
- **You asked:** Merge PR3 and continue brick 4 (tools + workspace).
- **What is going on:** PR3 is on `dev`. pager-render already compiles with PR2 stubs (`detach_std_command`, `image_validate`, `enable-always-approve`). SUMMARY’s “map to next-code Registry / worktree” overshoots what Face/pager actually imports.
- **We recommend:** **Option C (narrow)** — grow `xai-grok-tools` / `xai-grok-workspace` with **compile stubs** for the top symbols full pager (PR7) will need. Do **not** wire `next_code_app_core::tool::Registry` or implement Grok worktrees in PR4. Keep ACP id `"enable-always-approve"` unchanged (YOLO bridge = PR5).
- **Risk:** Low–Medium (stub surface can grow; avoid pulling ripgrep/registry runtime)
- **Status:** Implemented — awaiting merge of PR #38

## Feature planning
- **Recommended approach:** Inventory upstream pager imports of `xai_grok_tools` / `xai_grok_workspace`; add minimal type/module stubs so a future pager crate can compile against shims. Leave existing Face symbols intact. Foreign sessions = empty stubs (SUMMARY skip).
- **Prior art:** Local `grok-build` (`SOURCE_REV` ba69d70) tools (~630 rs) + workspace (~261 rs). next-code: `PermissionMode` / `always_allow_tools` — different shape than ACP option id.
- **Integration points:** `crates/xai-grok-tools`, `crates/xai-grok-workspace`; keep `xai-grok-pager-render` green
- **Sub-agents used:** yes (explore) + parent verification against pager `use` lines
- **Option A (skip):** docs-only — almost no-op, doesn’t help PR7
- **Option B (avoid):** remap always-approve id → next-code mode string — breaks Face cursor matching
- **Open questions (defaults applied):**
  1. Type stubs only (no Registry execution) ✅
  2. Keep `"enable-always-approve"` wire id ✅
  3. foreign_sessions = empty / skip ✅
  4. No real `worktree` module ✅
  5. Fuzzy matcher = empty (no nucleo) ✅
  6. folder_trust `feature_enabled` always false (local-inert) ✅

## Evidence
1. pager-render imports only detach + image_validate + permission id helpers (verified)
2. Upstream pager has ~100 `use xai_grok_tools|workspace` sites across ~50 files — ToolOutput / ask_user_question / permission / folder_trust dominate
3. Upstream tools/workspace are huge runtime crates — wrong to vendor wholesale in PR4
4. next-code has no ACP option id equal to `enable-always-approve`
5. Verified shapes: `ToolOutput` `#[serde(tag = "type")]`, `SessionMode` strum snake_case, `ENABLE_ALWAYS_APPROVE_OPTION_ID = "enable-always-approve"`, `MCP_TOOL_NAME_DELIMITER = "__"`

## Steps
1. [x] Freeze import list from upstream `xai-grok-pager` → tools/workspace
2. [x] Add stub modules/types for that list only
3. [x] `cargo check -p xai-grok-tools -p xai-grok-workspace -p xai-grok-pager-render`
4. [x] Update SUMMARY; PR → `dev`, Refs #35

## Files touched
- `crates/xai-grok-tools/**` — types/output, SessionMode, skills, ask_user_question, slash constants, TemplateRenderer stub
- `crates/xai-grok-workspace/**` — permission helpers, folder_trust, trust, foreign_sessions empty, fuzzy empty, RestoreDegree
- `crates/xai-grok-config` — `RemoteSettings { folder_trust_enabled }`
- `docs/grok-migration-SUMMARY.md`
- `docs/plans/PLAN-20260720-grok-pr4-tools-workspace.md`
