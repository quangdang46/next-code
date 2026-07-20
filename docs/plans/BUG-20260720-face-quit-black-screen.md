# Plan Report — Face quit black screen

## Summary (read this first)
- **You asked:** Why Grok OK but next-code Face black-screen on quit — is wire logic mismatched?
- **Verified root cause:** Yes — **port adaptation mismatch**, not ACP auth wire.
  - Stock Grok (ratatui 0.29) drops the writer `mpsc::Sender` when the terminal drops → writer exits → `LeaveAlternateScreen` runs.
  - next-code Face (ratatui **0.28** shim) keeps a second `SharedTermWriter` in `ACTIVE_SHARED_WRITER` thread-local. That clone holds the **same** `Sender`. `drop(terminal)` does **not** close the channel → `writer_thread.join()` hangs forever → alt already cleared to black, **Leave never runs**, process zombies.
- **Fix shipped:** `SharedTermWriter::deactivate()` before drop/join; `join_timeout(2s)` so Leave still runs if writer sticks; regression test.
- **Risk:** Low
- **Status:** Implemented — rebuild Face / `next-code` and repro quit (`q`). Expect diag lines through `LeaveAlternateScreen_result=Ok` and process exit.

## Bug investigation
- **Verified root cause:** `ACTIVE_SHARED_WRITER` keeps `WriterSender` alive across `drain_writer_thread_before_teardown`.
- **Citations:**
  - `crates/xai-grok-pager-render/src/render/draw.rs` — `deactivate`, `join_timeout`, test `deactivate_closes_writer_channel_after_local_drop`
  - `crates/xai-grok-pager/src/app/mod.rs` — `drain_writer_thread_before_teardown` calls deactivate then bounded join

## Steps
1. [x] Clear TLS before join
2. [x] Bounded join + teardown still runs on drain Err
3. [ ] Operator: rebuild + quit repro; confirm diag + clean exit

## Next step
Rebuild (`scripts/install_release.sh` or your usual Windows install) and quit Face once; check `%LOCALAPPDATA%\next-code\face-quit-diag.log`.
