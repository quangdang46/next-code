# Plan Report

## Summary (read this first)
- **You asked:** After merging PR2 to `dev`, continue with PR3.
- **What is going on:** PR2 already landed empty `xai-grok-config` / `xai-grok-telemetry` stubs so pager-render compiles. Original SUMMARY “create stubs” is done — PR3 should **deepen** toward next-code, not recreate crates.
- **We recommend:** **Option A** — map `grok_home` → `NEXT_CODE_HOME` / `~/.next-code`; keep empty TOML load; keep telemetry no-op. Do **not** bridge Grok `[ui].*` ↔ next-code `[display].theme` yet (schemas differ).
- **Risk:** Low
- **Status:** Waiting for your OK — reply **go ahead** to implement

## Feature planning
- **Recommended approach:** Thin path-only deepen of `xai-grok-config` to use next-code home (prefer leaf copy of `next_code_dir` logic or depend on `next-code-storage` / `next-code-core` if already light). Accept `NEXT_CODE_HOME`; keep `GROK_HOME` as optional override for migration. Update display strings in pager-render `util.rs` (`~/.grok` → `~/.next-code`). Telemetry stays no-op.
- **Prior art:** Upstream `grok-build/.../xai-grok-config` path helpers + managed config merge (we skip merge). next-code: `next-code-storage::next_code_dir`, `NEXT_CODE_HOME`.
- **Integration points:** `crates/xai-grok-config`, `crates/xai-grok-pager-render/src/util.rs` labels, docs SUMMARY
- **Sub-agents used:** yes (explore)
- **Option B (defer):** load real `[ui]` from disk — blocked until theme/schema map exists
- **Option C (defer):** stub more pager symbols — wait for PR7 compile errors
- **Open questions (defaults if you just say go ahead):**
  1. Env precedence: `GROK_HOME` > `NEXT_CODE_HOME` > `~/.next-code` (compat)
  2. UI labels switch to `~/.next-code` / `$NEXT_CODE_HOME` immediately
  3. No migrate of existing `~/.grok/pager.toml` in PR3 (start clean under next-code home)

## Evidence
1. pager-render uses only: `grok_home`, `default_grok_home`, `user_grok_home`, `load_effective_config_disk_only` + telemetry event stubs gated by `is_enabled()==false`
2. next-code home: `crates/next-code-storage` / `NEXT_CODE_HOME` → `~/.next-code`
3. Theme mismatch: Grok `[ui].theme` vs next-code `[display].theme` (`auto|dark|light`)

## Steps (simple checklist)
1. [ ] Align home helpers in `xai-grok-config` to next-code dir
2. [ ] Update pager-render display prefixes for home
3. [ ] Keep empty config load + no-op telemetry
4. [ ] `cargo check -p xai-grok-pager-render`; targeted config tests
5. [ ] Update SUMMARY; open PR → base `dev`, Refs #35

## Files to touch
- `crates/xai-grok-config/src/lib.rs` (+ Cargo.toml deps if any)
- `crates/xai-grok-pager-render/src/util.rs` (labels)
- `docs/grok-migration-SUMMARY.md`
- `docs/plans/PLAN-20260720-grok-pr3-config-telemetry.md` (this file)

## If you want more detail
PR3 is still foundation: Face files go under next-code home instead of `~/.grok`. Real settings bridge and pager wiring stay later PRs.
