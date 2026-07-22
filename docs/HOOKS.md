# Lifecycle Hooks

next-code can run external commands (and HTTP / plugin handlers) at well-defined
lifecycle points so other programs can observe or gate agent behavior without
forking next-code.

The canonical runtime is the **`next-code-hooks`** crate (v2). Face `/hooks`
lists, enables/disables, adds (merge TOML), and removes those same handlers —
it does **not** use OpenCode’s in-process JS plugin `Hooks` model.

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

### CLI

```bash
next-code hooks list
next-code hooks list --event PreToolUse
next-code hooks list --json
next-code hooks enable PreToolUse 0
next-code hooks disable SessionStart 0
next-code hooks test SessionStart          # dry-run
next-code hooks test PreToolUse --execute  # actually run
next-code hooks metrics
```

### Face `/hooks`

Extensions → Hooks tab uses ACP `x.ai/hooks/list` and `x.ai/hooks/action`:

| Key | Action |
|-----|--------|
| `r` | Reload list from disk |
| `Space` | Enable / disable selected hook (or whole source group) |
| `a` | Merge another `hooks.toml` into `~/.next-code/hooks.toml` |
| `x` | Remove selected handler by id (`user/PreToolUse[0]`) |

Trust / untrust return Unsupported (next-code always loads project `hooks.toml`
like other project config). Hook ids look like `user/PreToolUse[0]`.

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

## Runtime dispatch (what actually fires)

| Event | Fired? | Where |
|-------|--------|--------|
| `Setup` | Yes | Agent create |
| `SessionStart` | Yes | Agent create / attach / restore |
| `AgentStart` | Yes | Agent create / attach (primary) |
| `SessionEnd` / `AgentEnd` | Yes | `mark_closed` |
| `SessionUpdated` / `SessionError` | Yes | Session state paths |
| `SessionDiff` | Yes | After file-modifying tools |
| `SessionIdle` | Yes | After turn completes (streaming + headless) |
| `Stop` | Yes | After turn completes (fire-and-forget; deny cannot cancel finished turn) |
| `TurnEnd` | Yes | Streaming chat path (`fire_turn_end_hook`) |
| `UserPromptSubmit` / `UserPromptExpansion` | Yes | Turn start |
| `PreToolUse` / `PostToolUse` / `PostToolUseFailure` / `ToolError` | Yes | Tool registry |
| `FileChanged` | Yes | Write tool |
| `PermissionRequest` / `PermissionDenied` / `PermissionAsked` / `PermissionReplied` | Yes | DCG bridge |
| `PreCompact` / `PostCompact` | Yes | Manual compact, auto context-limit compact, compact completion poll |
| `SubagentStart` / `SubagentStop` | **Not yet** | Swarm/subagent spawn sites not wired — configure handlers for future use |
| `TaskCreated` / `TaskCompleted` | **Not yet** | No dedicated dispatch site |
| `AutoCompactionControl` | **Not yet** | Control plane not exposed as a hook seam |
| `Custom(...)` | Only if you call dispatch with that name | No built-in emitter |

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
  tab. Import of plugin-bundled `hooks/` JSON into the registry is **deferred**
  (follow-up); enable the plugin for skills/agents today.
- Hot paths check whether handlers exist before building large payloads.
- Phase 3 (OpenCode-like in-process TypeScript hooks) is **deferred** — large
  optional work; shell/HTTP hooks remain the product.
