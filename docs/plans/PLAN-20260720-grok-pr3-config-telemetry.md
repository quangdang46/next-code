# Plan Report

## Summary (read this first)
- **You asked:** Research grok-build carefully and implement PR3.
- **What is going on:** PR2 stubs pointed at `~/.grok`. Upstream grok-build uses `$GROK_HOME` / `~/.grok`; next-code uses `$NEXT_CODE_HOME` / `~/.next-code`.
- **We recommend / did:** Map Face home with precedence `GROK_HOME` > `NEXT_CODE_HOME` > `~/.next-code`; keep empty TOML + no-op telemetry.
- **Risk:** Low
- **Status:** Implemented — verify + merge PR #37

## Evidence (LOOK)
1. Upstream `grok-build/.../xai-grok-config/src/paths.rs` — `default_grok_home` → `~/.grok`, `GROK_HOME` override, `dunce::canonicalize`, `OnceLock`
2. Upstream user guide — `~/.grok/pager.toml`, `GROK_HOME` override ([05-configuration.md](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md))
3. next-code `next_code_dir()` — `$NEXT_CODE_HOME` then `~/.next-code` (`crates/next-code-storage/src/lib.rs`)
4. Theme schema mismatch — defer real config load

## Steps
1. [x] Align `xai-grok-config` home helpers
2. [x] Update pager-render display / clipboard labels + tests
3. [x] Keep empty config load + no-op telemetry
4. [ ] `cargo test -p xai-grok-config` + `cargo check -p xai-grok-pager-render`
5. [ ] Push / merge PR #37

## Files touched
- `crates/xai-grok-config/src/lib.rs`
- `crates/xai-grok-pager-render/src/util.rs`
- `crates/xai-grok-pager-render/src/clipboard/mod.rs`
- `docs/grok-migration-SUMMARY.md`
