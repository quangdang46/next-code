# Safe evaluation profile

A first-run "kick the tires" profile for evaluating next-code safely **before** pointing it at your main machine, primary credentials, or sensitive repositories.

## Quick start

```bash
next-code --safe-eval run "say hello"
```

That's it. The flag layers a conservative sandbox on top of whatever else you pass.

Equivalent without the flag:

```bash
NEXT_CODE_SAFE_EVAL=1 next-code run "say hello"
```

## What `--safe-eval` actually does

`--safe-eval` (and the matching `NEXT_CODE_SAFE_EVAL=1` env var) is translated at startup into a coordinated set of environment overrides:

| Env var set | Effect |
|---|---|
| `NEXT_CODE_HOME=~/.next-code-safe-eval/` (only if `NEXT_CODE_HOME` was not already set) | Isolated config / sessions / memory / auth dir. Your real `~/.next-code/` is **not** touched, read, or written. |
| `NEXT_CODE_OFFLINE=1` | Disables all startup network operations (update check, telemetry, provider model-list refresh). Provider API calls during a session are unaffected. |
| `NEXT_CODE_NO_TELEMETRY=1` | Belt-and-suspenders: even if telemetry was somehow re-enabled, no events are sent. |
| `NEXT_CODE_AMBIENT_DISABLED=1` | Ambient mode does not start a background runner. |
| `NEXT_CODE_NO_SELFDEV=1` | Self-dev auto-detection is suppressed. |
| `NEXT_CODE_REQUIRE_MCP_TRUST=1` | Project-local `.next-code/mcp.json` / `.claude/mcp.json` are skipped unless their content is in the user's trust store. Manage with `next-code mcp trust <path>` / `next-code mcp revoke <path>` / `next-code mcp list`. See issue #62. |

A short banner is printed at startup so you can confirm the profile took effect:

```
Safe-eval profile: isolated NEXT_CODE_HOME, telemetry off, offline, ambient/selfdev gated.
  NEXT_CODE_HOME = /home/<user>/.next-code-safe-eval
```

(Pass `--quiet` to suppress the banner once you're comfortable with what it does.)

## What is **not** disabled

- The provider you choose still talks to its API during the session itself. `--safe-eval` is about next-code's startup behavior + persistent state, not about cutting off the LLM call you actually came to make.
- Built-in tools (`read`, `write`, `edit`, `bash`, …) are unchanged. You still have a powerful agent — just aimed at a sandboxed home dir.
- MCP servers from `~/.next-code-safe-eval/mcp.json` (if you create one inside the isolated home) still run. The isolated home means you won't accidentally pick up MCP configs from your real `~/.next-code/`.

## Recommended workflow for first-run evaluation

1. **Use a disposable repo / worktree.** `cd ~/sandbox && git clone --depth 1 …`.
2. **Run next-code under `--safe-eval`** so nothing from your real `~/.next-code/` is read or written.
3. **Pass an explicit cheap provider.** For instance:
   ```bash
   next-code --safe-eval --provider deepseek run "explain this repo"
   ```
4. **Verify the isolation.** After the run:
   ```bash
   ls ~/.next-code-safe-eval         # session, auth, memory all live here
   diff -r ~/.next-code ~/.next-code-safe-eval 2>/dev/null | head
   ```
   Your real `~/.next-code/` should be unchanged.
5. **When happy, drop the flag and use next-code normally.**

## Cleanup

```bash
rm -rf ~/.next-code-safe-eval
```

That removes every artifact `--safe-eval` produced — sessions, auth tokens stored in the isolated home, transcripts, memory, etc.

## Compose with other flags

`--safe-eval` is layered, so all other flags still work and override or extend the defaults:

```bash
# Want telemetry on for a one-off in safe-eval mode?
NEXT_CODE_NO_TELEMETRY=0 next-code --safe-eval run "..."

# Want a different isolated home?
NEXT_CODE_HOME=/tmp/next-code-test next-code --safe-eval run "..."
```

## See also

- `--offline` / `NEXT_CODE_OFFLINE` — startup network kill switch (issue #24)
- `--no-context-files` / `-c` — skip AGENTS.md / CLAUDE.md context (issue #9)
- `OAUTH.md` — per-provider login flows
- `docs/SAFETY_SYSTEM.md` — the in-session safety / approval system
