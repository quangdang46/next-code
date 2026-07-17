# next-code Plugin Author Guide

A comprehensive guide to building plugins for next-code. Plugins extend next-code with custom event handlers, tools, configuration, and integrations -- all running inside a secure QuickJS sandbox.

---

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Plugin Lifecycle](#plugin-lifecycle)
- [Manifest Format](#manifest-format)
- [TypeScript API Reference](#typescript-api-reference)
- [Event Reference](#event-reference)
- [Tool Registration](#tool-registration)
- [Capability Security Model](#capability-security-model)
- [Configuration](#configuration)
- [Environment Variables](#environment-variables)
- [CLI Commands](#cli-commands)
- [Testing Plugins](#testing-plugins)
- [Publishing to npm](#publishing-to-npm)
- [FAQ](#faq)

---

## Overview

next-code plugins are TypeScript or JavaScript modules that run inside a QuickJS sandbox. They can:

- **Listen to events** -- react to tool calls, session lifecycle, messages, and more.
- **Register custom tools** -- add new tools the model can invoke.
- **Modify behavior** -- block or modify tool inputs/outputs, inject system prompts, suppress notifications.
- **Persist state** -- use `pi.kv` for durable cross-session key-value storage.
- **Read configuration** -- access plugin-specific settings from `config.toml`.

Plugins have **no access** to Node.js built-ins, DOM, `require()`, or `process`. All host interaction goes through the `pi` global object injected by the runtime.

### What You Can Build

| Use Case | Example |
|----------|---------|
| Tool gatekeeper | Block dangerous tool calls based on custom rules |
| Telemetry | Log tool usage, turn durations, token counts |
| Custom tools | Add domain-specific tools (e.g., `jira_create_ticket`) |
| Prompt injection | Append instructions to system prompts per session |
| Notification filter | Suppress or rewrite noisy notifications |
| Session analytics | Track conversation metrics across sessions |

---

## Quick Start

### 1. Create a plugin file

Create `~/.next-code/plugins/hello-plugin.ts`:

```typescript
// Declare the plugin identity
const manifest = {
  name: 'hello-plugin',
  version: '1.0.0',
  description: 'A minimal next-code plugin',
  capabilities: {
    events: ['TurnStart'],
  },
};

// Register an event handler
pi.on('TurnStart', (event) => {
  pi.logger.info(`[hello-plugin] Turn started in session ${event.session_id}`);
});

// Register a custom tool
pi.registerTool({
  name: 'hello_greet',
  description: 'Greet someone by name',
  parameters: {
    type: 'object',
    properties: {
      name: { type: 'string', description: 'Name to greet' },
    },
    required: ['name'],
  },
  handler: (params) => {
    return `Hello, ${params.name}!`;
  },
});

// Export plugin metadata
export default {
  name: manifest.name,
  version: manifest.version,
  description: manifest.description,
};
```

### 2. Start next-code

```bash
next-code
```

next-code automatically discovers plugins in `~/.next-code/plugins/` on startup.

### 3. Verify it loaded

```bash
next-code plugin list
```

You should see `file:hello-plugin.ts` in the output.

---

## Plugin Lifecycle

Plugins go through these stages:

```
Discovery  -->  Preflight  -->  Load  -->  Activate  -->  Runtime  -->  Unload
```

1. **Discovery** -- next-code scans plugin directories, npm cache, and config sources.
2. **Preflight** -- Static analysis checks for dangerous patterns, undeclared capabilities, and suspicious constructs. Warnings are logged; blocks prevent loading.
3. **Load** -- TypeScript is transpiled to JavaScript via SWC, then evaluated in a QuickJS sandbox.
4. **Activate** -- Event handlers and tools registered during module evaluation become active.
5. **Runtime** -- Events are dispatched to registered handlers. Tools are callable by the model.
6. **Unload** -- Cleanup on session end or plugin disable. `SessionEnd` event fires before teardown.

### Preflight Analysis

Before a plugin loads, next-code runs static analysis to detect:

| Pattern | Severity | Effect |
|---------|----------|--------|
| `eval()` | Warning | Logged, plugin still loads |
| `new Function()` | Warning | Logged, plugin still loads |
| `process.*` | Warning | Not available in sandbox |
| `require()` | Warning | Use ES imports instead |
| `fetch()` without network capability | Warning | Declare `network` capability |
| `exec()`/`spawn()` without shell capability | Warning | Declare `shell` capability |
| `rm -rf`, `sudo`, `chmod 777` | **Block** | Plugin loading is prevented |

---

## Manifest Format

Plugins declare their identity and capabilities via a manifest. For npm packages, the manifest lives in `package.json` under the `"next-code"` (or `"pi"`) key. For local files, the manifest is the exported default object.

### package.json (npm plugins)

```json
{
  "name": "next-code-plugin-analytics",
  "version": "1.0.0",
  "main": "dist/server.js",
  "next-code": {
    "name": "analytics",
    "package_name": "next-code-plugin-analytics",
    "version": "1.0.0",
    "description": "Track tool usage analytics",
    "author": "Your Name",
    "license": "MIT",
    "kind": "server",
    "entry": {
      "server": "dist/server.js",
      "tui": "dist/tui.js"
    },
    "capabilities": {
      "fs_write": ["$HOME/.next-code/data/analytics"],
      "events": ["PreToolUse", "PostToolUse", "TurnStart", "TurnEnd"],
      "register_tools": true,
      "read_config": true
    },
    "engines": {
      "next-code": ">=0.9.0"
    },
    "tags": ["analytics", "telemetry"]
  }
}
```

### Manifest Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | Yes | Plugin identifier (short name) |
| `package_name` | `string` | Yes | npm package name |
| `version` | `string` | Yes | Semver version string |
| `description` | `string` | No | Human-readable description |
| `author` | `string` | No | Plugin author |
| `license` | `string` | No | SPDX license identifier |
| `kind` | `"server" \| "tui" \| "both"` | No | Where the plugin runs (default: `"server"`) |
| `entry` | `PluginEntry` | No | Entry point paths |
| `capabilities` | `PluginCapabilities` | No | Required capabilities |
| `features` | `Record<string, PluginFeature>` | No | Toggleable features |
| `settings` | `Record<string, SettingSchema>` | No | User-configurable settings |
| `engines` | `{ next-code?: string }` | No | Required next-code version range |
| `icon` | `string` | No | Icon path or URL |
| `homepage` | `string` | No | Project homepage URL |
| `repository` | `string` | No | Source repository URL |
| `tags` | `string[]` | No | Categorization tags |

### PluginEntry

```typescript
interface PluginEntry {
  server?: string;   // Entry point for server mode
  tui?: string;      // Entry point for TUI mode
  both?: string;     // Entry point for both modes
}
```

### PluginKind

- `"server"` -- Plugin runs in server/headless mode (default).
- `"tui"` -- Plugin runs in TUI mode only.
- `"both"` -- Plugin runs in both modes.

---

## TypeScript API Reference

All plugin APIs are available through the global `pi` object (injected as `__jcode_pi`).

### `pi.id`

```typescript
pi.id: string
```

The plugin's unique identifier. Format: `npm:package-name` or `file:/path/to/plugin.ts`.

### `pi.name`

```typescript
pi.name: string
```

The plugin's display name (same as `pi.id` for most plugins).

### `pi.version`

```typescript
pi.version: string
```

The plugin's version string.

### `pi.on(eventName, handler)`

```typescript
pi.on(event: string, handler: (event: any) => void | HandlerResult): void
```

Register an event handler. The handler receives an event object whose shape depends on the event type. See [Event Reference](#event-reference) for all events and their fields.

**Parameters:**
- `event` -- Event name (e.g., `"TurnStart"`, `"PreToolUse"`)
- `handler` -- Callback function. May return a `HandlerResult` for events that support modification.

**Example:**

```typescript
pi.on('TurnStart', (event) => {
  pi.logger.info(`Turn ${event.turn_number} started`);
});

pi.on('PreToolUse', (event) => {
  if (event.tool_name === 'rm') {
    return { action: 'block', output: 'Blocked by policy' };
  }
  return { action: 'continue' };
});
```

### `pi.registerTool(toolDefinition)`

```typescript
pi.registerTool(tool: {
  name: string;
  description: string;
  parameters: JSONSchema;
  handler: (params: any) => any;
}): void
```

Register a custom tool that the model can invoke. See [Tool Registration](#tool-registration) for details.

### `pi.getConfig(key)`

```typescript
pi.getConfig(key: string): string
```

Read a plugin configuration value from the global next-code config. Returns an empty string if not set.

**Parameters:**
- `key` -- Configuration key (e.g., `"my-plugin.apiKey"`)

**Example:**

```typescript
const apiKey = pi.getConfig('my-plugin.apiKey');
const maxResults = parseInt(pi.getConfig('my-plugin.maxResults') || '10', 10);
```

### `pi.logger`

```typescript
pi.logger: {
  info(message: string): void;
  warn(message: string): void;
  error(message: string): void;
  debug(message: string): void;
}
```

Structured logger that writes to next-code's tracing system. Messages appear in debug logs and can be filtered by level.

**Example:**

```typescript
pi.logger.info('[my-plugin] Starting up');
pi.logger.warn('[my-plugin] Deprecated config key used');
pi.logger.error('[my-plugin] Failed to parse input');
pi.logger.debug('[my-plugin] Internal state: ' + JSON.stringify(state));
```

### `pi.kv`

```typescript
pi.kv: {
  get(key: string): string;
  set(key: string, value: string): void;
}
```

Durable key-value storage that persists across sessions. Backed by the runtime's storage layer. Values are strings; serialize complex data with `JSON.stringify`/`JSON.parse`.

**Example:**

```typescript
// Save state
pi.kv.set('my-plugin.counter', JSON.stringify({ count: 42 }));

// Restore state
const saved = pi.kv.get('my-plugin.counter');
const counter = saved ? JSON.parse(saved) : { count: 0 };
```

### `pi.sleep(ms)`

```typescript
pi.sleep(ms: number): void
```

Block the current execution for the specified number of milliseconds. Use sparingly -- this blocks the sandbox thread.

**Example:**

```typescript
pi.sleep(100); // Wait 100ms
```

### `pi.uuid()`

```typescript
pi.uuid(): string
```

Generate a new UUID v4 string.

**Example:**

```typescript
const requestId = pi.uuid();
```

### `pi.cwd`

```typescript
pi.cwd: string
```

The current working directory of the next-code process.

**Example:**

```typescript
pi.logger.info(`Working directory: ${pi.cwd}`);
```

---

## Event Reference

Events are dispatched to handlers registered via `pi.on()`. Each event has specific input fields and optional output fields that allow modification.

### Event Categories

| Category | Events | Description |
|----------|--------|-------------|
| **Tool** | `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `ToolExecutionStart`, `ToolExecutionEnd` | Tool execution lifecycle |
| **Session** | `SessionStart`, `SessionEnd`, `SessionSwitch`, `SessionCompact`, `SessionBeforeCompact`, `SessionShutdown` | Session lifecycle |
| **Agent** | `AgentStart`, `AgentEnd` | Agent lifecycle |
| **Turn** | `TurnStart`, `TurnEnd` | Conversation turn lifecycle |
| **Message** | `MessageStart`, `MessageEnd` | Message production lifecycle |
| **Compact** | `PreCompact`, `PostCompact`, `AutoCompactionStart` | Context compaction |
| **Permission** | `PermissionRequest`, `PermissionDenied` | Permission system |
| **Task** | `TaskCreated`, `TaskCompleted` | Task management |
| **Other** | `UserPromptSubmit`, `Stop`, `Notification` | Misc events |

### Tool Events

#### `PreToolUse`

Fired **before** a tool is executed. Can block or modify the tool input.

**Input:**
```typescript
{
  tool_name: string;        // Name of the tool about to run
  tool_input: any;          // Tool parameters
  session_id: string;       // Current session ID
}
```

**Output (return value):**
```typescript
{
  block?: string;           // If set, tool is blocked with this reason
  modified_input?: any;     // If set, replaces the tool input
}
```

**Example:**
```typescript
pi.on('PreToolUse', (event) => {
  // Block dangerous tools
  if (event.tool_name === 'Bash' && event.tool_input.command?.includes('rm -rf')) {
    return { block: 'Blocked dangerous command' };
  }

  // Modify Read tool to add a limit
  if (event.tool_name === 'Read' && !event.tool_input.limit) {
    return {
      modified_input: { ...event.tool_input, limit: 200 }
    };
  }

  return {}; // Continue without modification
});
```

#### `PostToolUse`

Fired **after** a tool returns successfully. Can modify the output.

**Input:**
```typescript
{
  tool_name: string;        // Tool that was executed
  tool_input: any;          // Tool parameters that were used
  tool_output: any;         // Tool's return value
  duration_ms: number;      // Execution time in milliseconds
  success: boolean;         // Whether the tool succeeded
  session_id: string;       // Current session ID
}
```

**Output:**
```typescript
{
  modified_output?: any;    // If set, replaces the tool output
}
```

**Example:**
```typescript
pi.on('PostToolUse', (event) => {
  pi.logger.info(`Tool ${event.tool_name} took ${event.duration_ms}ms`);

  // Track metrics
  totalToolDuration += event.duration_ms;
});
```

#### `PostToolUseFailure`

Fired when a tool execution fails.

**Input:**
```typescript
{
  tool_name: string;        // Tool that failed
  tool_input: any;          // Tool parameters
  error: string;            // Error message
  duration_ms: number;      // Execution time before failure
  session_id: string;       // Current session ID
}
```

**Output:** None (read-only event).

#### `ToolExecutionStart`

Fired when a tool begins execution (before `PreToolUse`).

**Input:**
```typescript
{
  tool_name: string;
  tool_input: any;
  session_id: string;
}
```

**Output:** None (read-only event).

#### `ToolExecutionEnd`

Fired when a tool finishes execution.

**Input:**
```typescript
{
  tool_name: string;
  tool_output: any;
  duration_ms: number;
  session_id: string;
}
```

**Output:** None (read-only event).

### Session Events

#### `SessionStart`

Fired when a new session begins.

**Input:**
```typescript
{
  session_id: string;       // New session's ID
  project_dir: string;      // Project directory path
  model: string;            // Model being used (e.g., "claude-4")
  provider: string;         // Provider name (e.g., "anthropic")
}
```

**Output:** None (read-only event).

#### `SessionEnd`

Fired when a session ends. Use for cleanup and state persistence.

**Input:**
```typescript
{
  session_id: string;       // Session that ended
  duration_seconds: number; // Total session duration
  message_count: number;    // Messages in the session
}
```

**Output:** None (read-only event).

**Example:**
```typescript
pi.on('SessionEnd', () => {
  // Persist state before shutdown
  pi.kv.set('my-plugin.data', JSON.stringify(collectedData));
  pi.logger.info('State saved, shutting down');
});
```

#### `SessionSwitch`

Fired when the user switches to a different session.

**Input:**
```typescript
{
  session_id: string;           // Current session
  target_session_id: string;    // Session being switched to
}
```

**Output:** None (read-only event).

#### `SessionCompact`

Fired when a session is compacted.

**Input:**
```typescript
{
  session_id: string;
  reason: string;               // Why compaction happened
}
```

**Output:** None (read-only event).

#### `SessionBeforeCompact`

Fired before compaction begins. Same input as `SessionCompact`.

#### `SessionShutdown`

Fired when the session system is shutting down entirely.

**Input:** Same as `SessionEnd`.

### Agent Events

#### `AgentStart`

Fired when an agent starts. Can inject additional system prompt text.

**Input:**
```typescript
{
  session_id: string;
  system_prompt: any;           // Current system prompt
  tools: any;                   // Available tools
}
```

**Output:**
```typescript
{
  additional_system_prompt: string[];  // Lines to append to system prompt
}
```

**Example:**
```typescript
pi.on('AgentStart', (event) => {
  return {
    additional_system_prompt: [
      'Always use the example_hello tool for greetings.',
      'Prefer concise responses.',
    ],
  };
});
```

#### `AgentEnd`

Fired when an agent finishes.

**Input:**
```typescript
{
  session_id: string;
  duration_seconds: number;
  message_count: number;
}
```

**Output:** None (read-only event).

### Turn Events

#### `TurnStart`

Fired when a conversation turn begins.

**Input:**
```typescript
{
  session_id: string;
  turn_number: number;          // Sequential turn number
  messages: any;                // Current message history
}
```

**Output:** None (read-only event).

#### `TurnEnd`

Fired when a turn completes.

**Input:**
```typescript
{
  session_id: string;
  turn_number: number;
  duration_ms: number;          // Turn duration
}
```

**Output:** None (read-only event).

### Message Events

#### `MessageStart`

Fired when a message begins (user, assistant, or system).

**Input:**
```typescript
{
  session_id: string;
  role: string;                 // "user" | "assistant" | "system"
}
```

**Output:** None (read-only event).

#### `MessageEnd`

Fired when a message is fully produced.

**Input:**
```typescript
{
  session_id: string;
  role: string;
  content: string;              // Full message content
}
```

**Output:** None (read-only event).

### Compact Events

#### `PreCompact`

Fired before context compaction. Can modify the system prompt or prevent compaction.

**Input:**
```typescript
{
  session_id: string;
  message_count: number;        // Messages in context
  token_count: number;          // Current token count
  system_prompt: any;           // Current system prompt
}
```

**Output:**
```typescript
{
  system_prompt?: any;          // Modified system prompt
  instructions?: string;        // Additional instructions
  prevent?: boolean;            // If true, prevent compaction
}
```

#### `PostCompact`

Fired after compaction completes.

**Input:**
```typescript
{
  session_id: string;
  messages_removed: number;     // Messages removed
  tokens_saved: number;         // Tokens freed
}
```

**Output:** None (read-only event).

#### `AutoCompactionStart`

Fired when automatic compaction triggers.

**Input:** Same as `PreCompact`.

### Permission Events

#### `PermissionRequest`

Fired when a permission decision is needed. Can approve, deny, or defer to user.

**Input:**
```typescript
{
  action: string;               // Action being requested
  tool_name?: string;           // Tool requesting permission
  target?: string;              // Target resource
  session_id: string;
}
```

**Output:**
```typescript
{
  decision?: "allow" | "deny" | "ask";  // Auto-decision
  message?: string;                      // Explanation
}
```

**Example:**
```typescript
pi.on('PermissionRequest', (event) => {
  // Auto-approve Read tool
  if (event.tool_name === 'Read') {
    return { decision: 'allow', message: 'Auto-approved by plugin' };
  }
  // Block shell commands
  if (event.tool_name === 'Bash') {
    return { decision: 'deny', message: 'Shell access denied by policy' };
  }
  return { decision: 'ask' }; // Let user decide
});
```

#### `PermissionDenied`

Fired when a permission is denied.

**Input:**
```typescript
{
  action: string;
  tool_name?: string;
  target?: string;
  session_id: string;
}
```

**Output:** None (read-only event).

### Task Events

#### `TaskCreated`

Fired when a new task is created.

**Input:**
```typescript
{
  session_id: string;
  task_id: string;
  subject: string;
}
```

**Output:** None (read-only event).

#### `TaskCompleted`

Fired when a task is marked complete.

**Input:**
```typescript
{
  session_id: string;
  task_id: string;
}
```

**Output:** None (read-only event).

### Other Events

#### `UserPromptSubmit`

Fired when the user submits a prompt. Can modify the prompt.

**Input:**
```typescript
{
  content: string;              // User's prompt text
  session_id: string;
}
```

**Output:**
```typescript
{
  modified_prompt?: string;     // If set, replaces the prompt
}
```

**Example:**
```typescript
pi.on('UserPromptSubmit', (event) => {
  // Auto-prepend context
  if (event.content.startsWith('/code ')) {
    return {
      modified_prompt: `Please write code: ${event.content.slice(6)}`
    };
  }
});
```

#### `Stop`

Fired when the agent stops.

**Input:**
```typescript
{
  session_id: string;
  reason: string;               // Why the agent stopped
}
```

**Output:**
```typescript
{
  reason: string;               // Stop reason (can be modified)
}
```

#### `Notification`

Fired for system notifications. Can suppress or modify.

**Input:**
```typescript
{
  level: string;                // "info" | "warn" | "error"
  message: string;              // Notification text
  session_id?: string;          // Optional session context
}
```

**Output:**
```typescript
{
  suppress?: boolean;           // If true, notification is suppressed
  modified_message?: string;    // If set, replaces the message
}
```

**Example:**
```typescript
pi.on('Notification', (event) => {
  // Suppress noisy notifications
  if (event.message.includes('cache hit')) {
    return { suppress: true };
  }
});
```

---

## Tool Registration

Register custom tools that the model can invoke via `pi.registerTool()`.

### Tool Definition

```typescript
interface ToolDefinition {
  name: string;                 // Tool name (must be unique)
  description: string;          // What the tool does (shown to model)
  parameters: JSONSchema;       // JSON Schema for parameters
  handler: (params: any) => any; // Implementation
}
```

### JSON Schema for Parameters

Parameters follow JSON Schema format:

```typescript
{
  type: 'object',
  properties: {
    name: {
      type: 'string',
      description: 'The user name'
    },
    count: {
      type: 'number',
      description: 'Number of items',
      default: 10
    },
    mode: {
      type: 'string',
      enum: ['fast', 'slow'],
      description: 'Processing mode'
    }
  },
  required: ['name']
}
```

### Handler Return Values

The handler can return any JSON-serializable value:

```typescript
// Return a string
pi.registerTool({
  name: 'greet',
  description: 'Say hello',
  parameters: { type: 'object', properties: { name: { type: 'string' } }, required: ['name'] },
  handler: (params) => `Hello, ${params.name}!`,
});

// Return an object
pi.registerTool({
  name: 'stats',
  description: 'Get statistics',
  parameters: { type: 'object', properties: {} },
  handler: () => ({ turns: turnCount, duration: totalDuration }),
});

// Return a number
pi.registerTool({
  name: 'count',
  description: 'Get the count',
  parameters: { type: 'object', properties: {} },
  handler: () => 42,
});
```

### Tool Naming Convention

Prefix your tools to avoid conflicts:

```typescript
// Good: prefixed with plugin name
pi.registerTool({ name: 'analytics_report', ... });
pi.registerTool({ name: 'analytics_reset', ... });

// Bad: generic name likely to conflict
pi.registerTool({ name: 'report', ... });
```

### Tool Execution Flow

When the model invokes a plugin tool:

1. next-code looks up the tool by name in the plugin registry.
2. The tool handler runs inside the QuickJS sandbox.
3. The return value is serialized and returned to the model.
4. If the handler throws, the error is reported to the model.

---

## Capability Security Model

Plugins declare required capabilities in their manifest. The runtime enforces these through a multi-layer security chain.

### Capability Fields

| Capability | Type | Description |
|------------|------|-------------|
| `fs_read` | `string[]` | Allowed read paths (e.g., `["$HOME/.next-code/data"]`) |
| `fs_write` | `string[]` | Allowed write paths |
| `network` | `string[]` | Allowed hosts (e.g., `["api.github.com"]`) |
| `shell` | `boolean` | Allow shell command execution |
| `register_tools` | `boolean` | Allow registering custom tools |
| `register_commands` | `boolean` | Allow registering CLI commands |
| `register_providers` | `boolean` | Allow registering LLM providers |
| `read_config` | `boolean` | Allow reading next-code config |
| `write_config` | `boolean` | Allow writing next-code config |
| `env_vars` | `string[]` | Allowed environment variables |
| `events` | `string[]` | Events the plugin can subscribe to |
| `llm_access` | `boolean` | Allow direct LLM access |
| `session_access` | `boolean` | Allow session manipulation |

### Evaluation Order

The security chain evaluates access in this order:

```
Mode check --> Deny list --> Global deny --> Allow list --> Global default
```

1. **Mode** -- If mode is `"none"`, all access is denied immediately.
2. **Deny list** -- Plugin-specific deny rules (highest priority).
3. **Global deny** -- System-wide deny rules.
4. **Allow list** -- Plugin-specific allow rules.
5. **Global default** -- Fallback: `"deny"`, `"allow"`, or `"ask"`.

### Access Modes

| Mode | Behavior |
|------|----------|
| `"all"` | Normal evaluation through the chain |
| `"trusted"` | Only explicit deny rules block access |
| `"none"` | All access denied (kill switch) |
| `"interactive"` | Requires user approval for each access |

### Declaring Capabilities

```typescript
const manifest = {
  name: 'my-plugin',
  version: '1.0.0',
  capabilities: {
    // Read access to specific directories
    fs_read: ['$HOME/.next-code/data', '$HOME/.config/my-plugin'],

    // Write access to a specific directory
    fs_write: ['$HOME/.next-code/data/my-plugin'],

    // Network access to specific hosts
    network: ['api.github.com', 'api.openai.com'],

    // Tool registration
    register_tools: true,

    // Config access
    read_config: true,

    // Events to subscribe to
    events: ['PreToolUse', 'PostToolUse', 'TurnStart'],

    // Shell access (use with caution)
    shell: false,
  },
};
```

### Capability Patterns

**Path patterns:** Use `$HOME` for the user's home directory. Paths are matched by prefix.

```typescript
capabilities: {
  fs_read: ['$HOME/.next-code/data'],  // Matches $HOME/.next-code/data/anything
}
```

**Host patterns:** Matched by substring containment.

```typescript
capabilities: {
  network: ['api.github.com'],  // Matches https://api.github.com/v1/...
}
```

### Security Best Practices

1. **Minimal capabilities** -- Only declare what you need.
2. **Specific paths** -- Use narrow path prefixes, not `$HOME`.
3. **Specific hosts** -- List exact API hosts, not wildcards.
4. **Avoid shell** -- Shell access is powerful and dangerous.
5. **No `eval()`** -- The preflight analyzer will warn on `eval()` usage.

---

## Configuration

### config.toml `[plugin]` Section

Plugin configuration lives in next-code's `config.toml`:

```toml
[plugin]
# Enable specific plugins
enable = ["my-plugin", "analytics"]

# Disable specific plugins
disable = ["broken-plugin"]

# Access mode: "all", "trusted", "none", "interactive"
mode = "all"

# If true, fail on any plugin load error
fail_closed = false

# Skip all plugin hooks
skip_hooks = false

# Force deny all plugin actions
force_deny = false

# Plugin sources
[[plugin.sources]]
type = "npm"
package = "next-code-plugin-analytics"
version = "1.0.0"

[[plugin.sources]]
type = "file"
path = "/home/user/my-plugin.ts"

[[plugin.sources]]
type = "directory"
path = "/home/user/plugins"

# Per-plugin settings
[plugin.settings.my-plugin]
api_key = "sk-..."
max_results = 50

# Per-plugin overrides
[plugin.plugins.my-plugin]
enable = true
timeout_ms = 5000

# Feature toggles
[plugin.features.my-plugin]
enable = ["advanced-analytics", "export"]
```

### SettingSchema

Plugins can declare user-configurable settings in their manifest:

```json
{
  "settings": {
    "apiKey": {
      "type": "string",
      "description": "API key for the service",
      "secret": true,
      "env": "MY_PLUGIN_API_KEY"
    },
    "maxResults": {
      "type": "number",
      "description": "Maximum results per query",
      "default": 10,
      "min": 1,
      "max": 100
    },
    "mode": {
      "type": "enum",
      "description": "Processing mode",
      "default": "fast",
      "values": ["fast", "slow", "auto"]
    },
    "enabled": {
      "type": "boolean",
      "description": "Enable the plugin",
      "default": true
    },
    "tags": {
      "type": "array",
      "description": "Filter tags",
      "items": { "type": "string" }
    },
    "advanced": {
      "type": "object",
      "description": "Advanced settings",
      "properties": {
        "retryCount": { "type": "number", "description": "Retry count", "default": 3 }
      }
    }
  }
}
```

### SettingSchema Types

| Type | Fields |
|------|--------|
| `string` | `description`, `default?`, `secret?`, `env?`, `pattern?`, `max_length?` |
| `number` | `description`, `default?`, `min?`, `max?` |
| `boolean` | `description`, `default?` |
| `enum` | `description`, `default?`, `values: string[]` |
| `array` | `description`, `default?`, `items: SettingSchema`, `max_items?` |
| `object` | `description`, `default?`, `properties: Record<string, SettingSchema>` |

---

## Environment Variables

next-code checks these environment variables for plugin system control:

| Variable | Effect |
|----------|--------|
| `NEXT_CODE_DISABLE_PLUGINS=1` | Disables all plugins (sets mode to `"none"`) |
| `NEXT_CODE_SKIP_PLUGINS=1` | Skips all plugin hooks (plugins load but don't fire) |
| `NEXT_CODE_PLUGIN_MODE=<mode>` | Override plugin access mode (`"all"`, `"trusted"`, `"none"`, `"interactive"`) |
| `NEXT_CODE_TEAM_WORKER=1` | Force-deny all plugin actions (for automated/team environments) |

These are **kill switches** -- they take effect immediately and override config.toml settings.

### Checking Kill Switches

```bash
next-code plugin doctor
```

This command reports active kill switches and can clear them with `--fix`.

---

## CLI Commands

### `next-code plugin list`

List all installed plugins and their states.

```bash
next-code plugin list
```

Output:
```
Installed plugins:
  npm:next-code-plugin-analytics   active
  file:hello-plugin.ts         active
```

### `next-code plugin install <source>`

Install a plugin from npm or local path.

```bash
# Install from npm
next-code plugin install next-code-plugin-analytics

# Install from local file
next-code plugin install /path/to/my-plugin.ts

# Install from local directory
next-code plugin install /path/to/plugins/
```

### `next-code plugin uninstall <id>`

Remove a plugin by its ID.

```bash
next-code plugin uninstall npm:next-code-plugin-analytics
next-code plugin uninstall file:hello-plugin.ts
```

### `next-code plugin info <id>`

Show detailed information about a plugin.

```bash
next-code plugin info npm:next-code-plugin-analytics
```

### `next-code plugin enable <id>`

Enable a previously disabled plugin.

```bash
next-code plugin enable npm:next-code-plugin-analytics
```

### `next-code plugin disable <id>`

Disable an active plugin (keeps it installed but inactive).

```bash
next-code plugin disable npm:next-code-plugin-analytics
```

### `next-code plugin audit`

Show the plugin audit trail (event dispatch history).

```bash
# Show last 20 entries
next-code plugin audit

# Show last 50 entries as JSON
next-code plugin audit --recent 50 --json
```

### `next-code plugin doctor`

Diagnose plugin system issues. Can automatically fix problems with `--fix`.

```bash
# Check for issues
next-code plugin doctor

# Check and fix
next-code plugin doctor --fix
```

Output:
```
Plugin system status:
  Active plugins: 3
  Registered handlers: 12
  Audit trail entries: 47

✅ Plugin system is healthy
```

---

## Testing Plugins

### Local Testing

1. Place your plugin in `~/.next-code/plugins/`:

```bash
cp my-plugin.ts ~/.next-code/plugins/
```

2. Start next-code and verify it loads:

```bash
next-code plugin list
next-code plugin info file:my-plugin.ts
```

3. Check the audit trail to see events:

```bash
next-code plugin audit
```

### Debug Logging

Enable debug logging to see plugin activity:

```bash
RUST_LOG=next_code_plugin_runtime=debug next-code
```

This shows:
- Plugin discovery and loading
- Preflight analysis results
- Event dispatch to handlers
- Tool registration
- Capability checks

### Preflight Validation

Test your plugin's preflight analysis locally:

```bash
# The preflight analyzer checks for:
# - eval() usage (warning)
# - new Function() (warning)
# - process.* references (warning)
# - require() usage (warning)
# - fetch() without network capability (warning)
# - exec()/spawn() without shell capability (warning)
# - rm -rf, sudo, chmod 777 (block)
```

### Common Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| Plugin not loading | Preflight block | Remove suspicious patterns |
| Plugin not loading | Invalid manifest | Check `package.json` `next-code` field |
| Events not firing | Wrong event names | Check event name casing |
| Tools not registered | Missing `register_tools` capability | Add to manifest capabilities |
| Config returns empty | Wrong key format | Use `plugin-name.key` format |
| Plugin timed out | Handler took too long | Optimize handler or increase timeout |

### Sandbox Limitations

The QuickJS sandbox does **not** provide:

- `require()` or CommonJS modules
- `process`, `__dirname`, `__filename`
- Node.js built-ins (`fs`, `path`, `http`, etc.)
- DOM APIs
- `setTimeout`/`setInterval` (use `pi.sleep()` instead)
- Dynamic `import()`

All host interaction must go through `pi.*` methods.

---

## Publishing to npm

### Package Structure

```
next-code-plugin-my-feature/
  package.json
  tsconfig.json
  src/
    server.ts          # Server entry point
    tui.ts             # TUI entry point (optional)
  dist/
    server.js          # Compiled output
    tui.js             # Compiled output
  README.md
  LICENSE
```

### package.json

```json
{
  "name": "next-code-plugin-my-feature",
  "version": "1.0.0",
  "description": "My awesome next-code plugin",
  "main": "dist/server.js",
  "scripts": {
    "build": "tsc",
    "prepublishOnly": "npm run build"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  },
  "next-code": {
    "name": "my-feature",
    "package_name": "next-code-plugin-my-feature",
    "version": "1.0.0",
    "description": "My awesome next-code plugin",
    "author": "Your Name",
    "license": "MIT",
    "kind": "server",
    "entry": {
      "server": "dist/server.js"
    },
    "capabilities": {
      "events": ["TurnStart", "TurnEnd"],
      "register_tools": true
    },
    "engines": {
      "next-code": ">=0.9.0"
    }
  }
}
```

### tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "node",
    "outDir": "dist",
    "rootDir": "src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "declaration": true
  },
  "include": ["src/**/*"]
}
```

### Publishing Steps

```bash
# 1. Build
npm run build

# 2. Test locally
cp -r . ~/.next-code/cache/packages/next-code-plugin-my-feature/
next-code plugin install next-code-plugin-my-feature

# 3. Publish to npm
npm publish
```

### Naming Convention

Use the prefix `next-code-plugin-` for discoverability:

```
next-code-plugin-analytics
next-code-plugin-github-integration
next-code-plugin-custom-tools
```

### Versioning

Follow semver. The `engines.next-code` field specifies the minimum next-code version:

```json
{
  "engines": {
    "next-code": ">=0.9.0"
  }
}
```

---

## FAQ

### Q: Can I use npm packages in my plugin?

**A:** No. The QuickJS sandbox does not support `require()` or dynamic `import()`. All functionality must be implemented using the built-in `pi.*` APIs or plain JavaScript.

### Q: How do I persist data across sessions?

**A:** Use `pi.kv.set(key, value)` and `pi.kv.get(key)`. Values are strings, so serialize objects with `JSON.stringify()`.

```typescript
// Save
pi.kv.set('my-plugin.data', JSON.stringify({ count: 42 }));

// Load
const data = JSON.parse(pi.kv.get('my-plugin.data') || '{}');
```

### Q: Can my plugin make HTTP requests?

**A:** Not directly. The sandbox does not provide `fetch()` or `XMLHttpRequest`. If you need network access, declare the `network` capability and use the runtime's bridge (if available). For most use cases, register a custom tool and let the model handle the request.

### Q: How do I debug my plugin?

**A:** Use `pi.logger.debug()` for detailed logging, and run next-code with `RUST_LOG=next_code_plugin_runtime=debug` to see all plugin activity. Check `next-code plugin audit` for event dispatch history.

### Q: Can multiple plugins handle the same event?

**A:** Yes. All registered handlers for an event are called concurrently via `join_all`. Each handler receives a clone of the event input. Multiple `PreToolUse` handlers can block a tool -- if any handler returns `{ action: 'block' }`, the tool is blocked.

### Q: What happens if my plugin throws an error?

**A:** The error is caught by the sandbox, logged as a warning, and the event dispatch continues with other handlers. The plugin does not crash next-code.

### Q: How do I test my plugin without publishing?

**A:** Place the `.ts` or `.js` file in `~/.next-code/plugins/` or configure a local path in `config.toml`:

```toml
[[plugin.sources]]
type = "file"
path = "/path/to/my-plugin.ts"
```

### Q: Can I access environment variables?

**A:** Not directly from the sandbox. Declare required env vars in your manifest's `env_vars` capability and access them through the runtime bridge (if available).

### Q: What is the difference between `PreToolUse` and `ToolExecutionStart`?

**A:** `ToolExecutionStart` fires first (read-only), then `PreToolUse` fires and can modify or block the tool. Use `ToolExecutionStart` for logging/monitoring and `PreToolUse` for policy enforcement.

### Q: How do I disable my plugin temporarily?

**A:** Use the CLI:

```bash
next-code plugin disable npm:my-plugin
```

Or add to `config.toml`:

```toml
[plugin.plugins.my-plugin]
enable = false
```

### Q: Can I register CLI commands?

**A:** Yes, if you declare `register_commands: true` in your capabilities. The exact API for command registration is still evolving.

### Q: What is the timeout for plugin handlers?

**A:** Default timeouts:
- **Informational events** (SessionEnd, TurnEnd, PostCompact, AutoCompactionStart): 500ms
- **Actionable events** (PreToolUse, PostToolUse, etc.): 5000ms
- **Permission events**: 3600ms (1 hour, to allow user interaction)

Per-plugin timeouts can be configured in `config.toml`:

```toml
[plugin.plugins.my-plugin]
timeout_ms = 10000
```

---

## Further Reading

- [API Reference](./api-reference.md) -- Complete TypeScript type definitions
- [Example Plugin](../../examples/plugins/example-plugin.ts) -- Full working example
- [Security Model](../SAFETY_SYSTEM.md) -- next-code security architecture
- [Config Reference](../CONFIG_REFERENCE.md) -- Full config.toml documentation
