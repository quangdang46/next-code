# Plan Report

## Summary (read this first)
- **You asked:** Why `nextcode --resume session_*` prints `Error: Session does not exist` after a normal quit hint.
- **What is going on:** next-code CLI correctly resolves the id under `NEXT_CODE_HOME` / sessions, then Face `materialize_startup` re-checks a **stubbed** Grok on-disk store (`resolve_local_session` → always `None`) and bails before ACP `session/load` / daemon `ResumeSession`.
- **We recommend:** In next-code embed, defer existence to the ACP agent (daemon authority). Keep stock Grok disk/GCS checks unchanged.
- **Risk:** Low
- **Status:** Implementing (parent asked investigate + fix)

## Bug investigation
- **Verified root cause:** Face `resolve_existing_session` + stub `xai_grok_shell::session::persistence::resolve_local_session` + `allow_remote_restore: false` → hard `"Session does not exist"` before `NextCodeFaceAgent::load_session`.
- **Ruled out:** LOCALAPPDATA vs USERPROFILE mismatch as primary (dispatch already found the session — user saw `Connecting to server...` then Face’s exact bail string). Ephemeral-only id is secondary; quit hint id matched a findable next-code session.
- **Hypotheses ranked:** 1) Face stub preflight (verified) 2) never persisted (would be `No session found matching` from dispatch) 3) serve wiped memory (ACP would fail later with daemon message)

## Evidence
1. `crates/xai-grok-shell/src/session/persistence.rs` — stub always `None` / `false`
2. `crates/xai-grok-pager/src/app/session_startup.rs` — bail `"Session does not exist"`
3. `src/cli/pager_launch.rs` — installs next-code agent; passes `--resume` into Face
4. `src/cli/dispatch.rs` — `find_session_by_name_or_id` then `Connecting to server...` then Face

## Steps
1. [x] Add `defer_existence_to_agent` on `MaterializeCtx` when `is_nextcode_embed()`
2. [x] Pass resume id through to ACP load
3. [x] Unit test + clearer connect/resume errors
4. [ ] Rebuild/install + smoke

## Files to touch
- `crates/xai-grok-pager/src/app/session_startup.rs`
- `crates/xai-grok-pager/src/headless.rs` (struct field)
- `src/cli/pager_agent.rs` (clearer errors)
