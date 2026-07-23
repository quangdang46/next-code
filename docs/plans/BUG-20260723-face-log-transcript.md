# BUG-20260723 — Face `/log` ≡ `/transcript` broken (Windows `$PAGER` / silent spawn)

- **Status:** LOOK done — waiting for OK before BUILD
- **Risk:** Low–medium (Face-local; no ACP/daemon)
- **Same command?** **Yes.** In Face, `/log` is an alias of `/transcript`.

## Summary

Face `/transcript` and `/log` are **one slash command**: both dispatch `Action::OpenTranscriptPager`, write a temp Markdown (or ANSI in minimal), then suspend the TUI into `$PAGER` (default **`less`**). They are **not** the legacy next-code pair (`/log mark` vs open session file). On this Windows host `$PAGER` is unset and `less` is missing; spawn failure is ignored (`let _ = Command…status()`), so both feel “broken” (flash / no-op).

## Evidence (verified)

| Claim | Where |
|-------|--------|
| `/log` alias of `transcript` | `crates/xai-grok-pager/src/slash/commands/transcript.rs` — `aliases() -> &["log"]`, `Action::OpenTranscriptPager` |
| Router → dispatch | `dispatch/router.rs` `OpenTranscriptPager` → `dispatch_open_transcript_pager` |
| Non-minimal: temp `grok-transcript-*.md` + `pending_pager_path` | `dispatch/transcript.rs` ~244–275 |
| Event loop: `$PAGER` else `"less"`; **status ignored** | `app/event_loop.rs` ~613–663 |
| No ACP/daemon handler | Slash → Face Action only; no pager_agent method for transcript view |
| Stock Grok same shape | DeepWiki `xai-org/grok-build`: alias + `$PAGER` suspend |
| Host check (2026-07-23) | `$env:PAGER` empty; `where less` empty |
| Alias hazard documented | `transcript.rs` header + `PLAN-20260721-slash-commands-grok-vs-nextcode.md` |

## Legacy vs Face (not the same product meaning)

| Token | Face (current embed) | Legacy next-code TUI |
|-------|----------------------|----------------------|
| `/transcript` | Render scrollback → `$PAGER` | Open session transcript path (OS app) / `path` subcmd |
| `/log` | **Alias of `/transcript`** | `/log mark [note]` → `NEXT_CODE_LOG_MARK` in `~/.next-code/logs/` |

Embed already chose Face meaning (`transcript.rs` comments). Log-mark is **not** ported.

## What users hit

Likely symptoms (no toast on spawn fail):

1. **Windows / no `less`:** brief suspend, pager never opens, TUI restores — looks like a no-op.
2. **Empty scrollback:** system line `No conversation transcript to view yet` / `No active session to view`.
3. **Minimal mode without minimal crate installed:** embed has no `xai-grok-pager-minimal`; if `screen_mode` were Minimal, incremental pump may never finish — **secondary**; default path is non-minimal MD export.

Not a stub / “unsupported” slash — command is registered and runs; **child pager fails closed silently**.

## Ranked hypotheses

1. **Primary:** Default `less` + empty `$PAGER` on Windows; spawn error discarded.  
2. Empty session / no active agent.  
3. Minimal mode without pump (only if user is in Minimal).  

## Ruled out

- Missing slash registration (command exists; alias wired).
- Daemon/ACP gap (feature is Face-local).
- `/log` vs `/transcript` being two different Face features (they are not).

## Fix path (PLAN — do not BUILD until OK)

| Option | Idea |
|--------|------|
| **A (recommended)** | Windows fallback pager when `less` missing / spawn fails: e.g. `more.com`, or open temp file with OS default (`cmd /c start`, etc.), and **surface an error toast** if spawn fails |
| **B** | Document `set PAGER=…` (e.g. `bat`, `less` from Git, `notepad`) — insufficient alone |
| **C** | Optional: `/export` already copies/writes MD without pager — point users there as workaround |

Touch: `crates/xai-grok-pager/src/app/event_loop.rs` (pager spawn + error report); optionally brand temp name `nextcode-transcript-*.md`. No ACP changes.

## Open questions

1. Preferred Windows fallback: OS default app vs `more` vs require `$PAGER`?
2. Keep Face `$PAGER` UX on Unix; only harden Windows + error toast?
3. Revive next-code `/log mark` as `/log-mark` later, or drop?

## Copy / wire / delete

| Kind | Action |
|------|--------|
| **Keep** | Face `/transcript` + `/log` alias |
| **Wire** | Reliable pager open + fail-visible on Windows |
| **Delete** | Silent `let _ = status()` for pager spawn (at least report Err) |
| **Do not** | Reintroduce legacy `/log mark` as `/log` without explicit product OK |
