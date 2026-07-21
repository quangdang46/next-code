# Lifecycle Hooks

next-code can run external commands (and HTTP / plugin handlers) at well-defined
lifecycle points so other programs can observe or gate agent behavior without
forking next-code.

The canonical runtime is the **`next-code-hooks`** crate (v2). Face `/hooks`
lists and enable/disables those same handlers — it does **not** use OpenCode’s
in-process JS plugin `Hooks` model.

## Canonical config (v2)

Layers (lowest → highest priority; handlers **append**, settings override):

| Layer | Path |
|-------|------|
| User | `~/.next-code/hooks.toml` (or `$NEXT_CODE_HOME/hooks.toml`) |
| Project | `<cwd>/.next-code/hooks.toml` |
| Env | `$NEXT_CODE_HOOKS_CONFIG` (path to a TOML file) |

Kill-switch: `DISABLE_NEXT_CODE_HOOKS` (any value) disables all layers.

```toml
# ~/.next-code/hooks.toml
[settings]
timeout_secs = 30
max_concurrency = 10

[[events.PreToolUse]]
type = "command"
enabled = true
command = "~/bin/next-code-tool-policy"
timeout_secs = 5

[[events.SessionStart]]
type = "command"
command = "~/bin/next-code-session-notify"

[[events.TurnEnd]]
type = "http"
method = "POST"
url = "http://127.0.0.1:9999/hooks/turn-end"
```

Handler types: `command` | `http` | `agent` | `plugin`.  
CLI: `next-code hooks list|enable|disable|test|metrics`.

Face Extensions → Hooks tab talks ACP `x.ai/hooks/list` and
`x.ai/hooks/action` (reload, enable, disable). Hook ids look like
`user/PreToolUse[0]`.

### OpenCode event name aliases (docs only)

OpenCode uses dotted in-process plugin callbacks. Map them to next-code
`HookEvent` names when reading OpenCode docs:

| OpenCode-style | next-code `HookEvent` |
|----------------|------------------------|
| `tool.execute.before` | `PreToolUse` |
| `tool.execute.after` | `PostToolUse` |
| `session.created` / start | `SessionStart` |
| `session.idle` | `SessionIdle` |
| `session.error` | `SessionError` |
| `permission.ask` / asked | `PermissionAsked` / `PermissionRequest` |
| compaction before/after | `PreCompact` / `PostCompact` |

OpenCode authors mutate args in TypeScript; next-code command hooks use stdin
JSON and exit `0` / `2` (allow / deny) for blocking events.

## Legacy v1 (`config.toml [hooks]`)

Still merged at runtime into v2 handlers via `legacy_v1_to_v2_handlers`:

```toml
# ~/.next-code/config.toml
[hooks]
turn_end      = "~/bin/next-code-turn-notify"     # observer
session_start = ""                            # observer
session_end   = ""                            # observer
pre_tool      = "~/bin/next-code-tool-policy"     # gate
post_tool     = ""                            # observer
pre_tool_timeout_ms = 5000
```

Env overrides (always win; empty value disables a config hook):
`NEXT_CODE_HOOK_TURN_END`, `NEXT_CODE_HOOK_SESSION_START`, `NEXT_CODE_HOOK_SESSION_END`,
`NEXT_CODE_HOOK_PRE_TOOL`, `NEXT_CODE_HOOK_POST_TOOL`, `NEXT_CODE_HOOK_PRE_TOOL_TIMEOUT_MS`.

Prefer v2 `hooks.toml` for new automation.

## Common contract (command handlers)

- The hook command line is parsed shell-style (quotes and backslash escapes
  work) but executed **directly**, not through a shell. A leading `~/` in the
  program path is expanded.
- The hook runs in the session working directory when known.
- Every hook receives env such as `NEXT_CODE_HOOK_EVENT`,
  `NEXT_CODE_HOOK_SESSION_ID`, `NEXT_CODE_HOOK_CWD`, plus JSON on stdin for
  tool gates. Nested next-code sees `NEXT_CODE_HOOKS_DISABLED=1` (recursion
  guard).

## Gate hook: `PreToolUse` / v1 `pre_tool`

Runs **synchronously before matching tool calls** and can block:

- **Exit 0**: allow.
- **Exit 2**: deny (stderr returned to the model when configured).
- Other failures default to fail-open unless `[settings].fail_closed = true`.

### Example policy script

```bash
#!/usr/bin/env bash
# ~/bin/next-code-tool-policy
input=$(cat)
case "$NEXT_CODE_HOOK_TOOL_NAME" in
  bash)
    if grep -qE 'rm -rf /([^a-zA-Z]|$)|mkfs|dd if=' <<<"$input"; then
      echo "blocked: destructive shell command" >&2
      exit 2
    fi
    ;;
esac
exit 0
```

## Design notes

- Face `/hooks` manages **next-code** lifecycle hooks, not OpenCode/Bun plugins
  and not `~/.grok/hooks` JSON files.
- Bundle plugins (`/plugins`, `~/.next-code/plugins`) are a sibling Extensions
  tab; optional import of plugin-bundled `hooks/` is a later phase.
- Hot paths check whether handlers exist before building large payloads.
