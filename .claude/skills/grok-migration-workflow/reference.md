# Lessons from Face cutover (reference)

Read only when debugging migration regressions. Prefer SKILL.md for the workflow.

## Proven incidents

### Black screen on quit (`q` / Ctrl+C)

- **Symptom:** Alt cleared to black; process zombie; no Leave.
- **Diag stop:** `alt_buffer_cleared` — never `writer_drained` / `LeaveAlternateScreen`.
- **Root cause:** ratatui 0.28 `SharedTermWriter::activate` TLS clone kept `mpsc::Sender` alive → `WriterThread::join` hung.
- **Fix:** `deactivate()` before `drop(terminal)` + bounded `join_timeout`; always run Leave if drain fails.
- **Not the cause:** ACP auth wire, dual-Leave sprays, `no_alt_screen`.

### Resume hint said `grok`

- Stock Face hardcodes `grok --resume`.
- Embed must use argv0 / `XAI_PAGER_RESUME_CLI` → `nextcode` / `next-code`.
- next-code CLI has `--resume` but not Face `--minimal` / `--fullscreen` flags.

### Quit replayed last error

- Fullscreen `ExitSummary` reprinted title/prompt/error after Leave.
- Product choice: resume-only tail (no transcript replay).

### Stale `nextcode` binary

- PATH may hit `%LOCALAPPDATA%\next-code\bin` or `~\.local\bin`.
- Install must update **both** `next-code.exe` and `nextcode.exe`.

## Stock vs embed checklist

| Stock grok-bin | next-code Face embed |
|----------------|----------------------|
| Own main + crash handler | `startup` + `pager_launch` |
| Default MvpAgent / leader | `install_agent_factory(NextCodeFaceAgent)` + `no_leader` |
| Hint: `grok --resume` | Hint: `nextcode --resume` |
| Same `app::run` restore | Must not skip deactivate / Leave |
