# next-code Plugin API Reference

Complete TypeScript type definitions for the next-code plugin system. All types are derived from the actual Rust implementation in `next-code-plugin-core` and `next-code-plugin-runtime`.

---

## Table of Contents

- [Plugin Global (`nextcode`)](#plugin-global-nextcode)
- [Plugin Manifest Types](#plugin-manifest-types)
- [Event Types](#event-types)
- [Handler Result Types](#handler-result-types)
- [Capability Types](#capability-types)
- [Security Types](#security-types)
- [Configuration Types](#configuration-types)
- [Error Types](#error-types)
- [Internal Types](#internal-types)

---

## Plugin Global (`nextcode`)

The `nextcode` object is injected into the QuickJS sandbox as `__nextcode_api` (dual-read: also `jcode` / `__jcode_api`) and provides all plugin APIs.

```typescript
interface PluginApi {
  /** Plugin identifier (e.g., "npm:my-plugin" or "file:/path/to/plugin.ts") */
  readonly id: string;

  /** Plugin display name */
  readonly name: string;

  /** Plugin version string */
  readonly version: string;

  /** Current working directory of the next-code process */
  readonly cwd: string;

  /** Structured logger */
  readonly logger: PluginLogger;

  /** Durable key-value storage */
  readonly kv: PluginKV;

  /**
   * Register an event handler.
   * @param event - Event name (e.g., "TurnStart", "PreToolUse")
   * @param handler - Callback invoked when the event fires
   */
  on(event: string, handler: (event: any) => void | HandlerResult): void;

  /**
   * Register a custom tool.
   * @param tool - Tool definition with name, description, parameters schema, and handler
   */
  registerTool(tool: ToolDefinition): void;

  /**
   * Read a plugin configuration value.
   * @param key - Configuration key (e.g., "my-plugin.apiKey")
   * @returns The configuration value, or empty string if not set
   */
  getConfig(key: string): string;

  /**
   * Block execution for the specified duration.
   * @param ms - Duration in milliseconds
   */
  sleep(ms: number): void;

  /**
   * Generate a new UUID v4.
   * @returns UUID string
   */
  uuid(): string;
}

interface PluginLogger {
  info(message: string): void;
  warn(message: string): void;
  error(message: string): void;
  debug(message: string): void;
}

interface PluginKV {
  get(key: string): string;
  set(key: string, value: string): void;
}
```

---

## Plugin Manifest Types

### PluginManifest

```typescript
interface PluginManifest {
  /** Plugin short name */
  name: string;

  /** npm package name */
  package_name: string;

  /** Semver version string */
  version: string;

  /** Human-readable description */
  description?: string;

  /** Plugin author */
  author?: string;

  /** SPDX license identifier */
  license?: string;

  /** Where the plugin runs */
  kind?: PluginKind;

  /** Entry point paths */
  entry?: PluginEntry;

  /** Required capabilities */
  capabilities?: PluginCapabilities;

  /** Toggleable features */
  features?: Record<string, PluginFeature>;

  /** User-configurable settings */
  settings?: Record<string, SettingSchema>;

  /** Required engine versions */
  engines?: PluginEngines;

  /** Icon path or URL */
  icon?: string;

  /** Project homepage URL */
  homepage?: string;

  /** Source repository URL */
  repository?: string;

  /** Categorization tags */
  tags?: string[];
}
```

### PluginKind

```typescript
type PluginKind = "server" | "tui" | "both";
```

- `"server"` -- Plugin runs in server/headless mode (default).
- `"tui"` -- Plugin runs in TUI mode only.
- `"both"` -- Plugin runs in both modes.

### PluginEntry

```typescript
interface PluginEntry {
  /** Entry point for server mode */
  server?: string;

  /** Entry point for TUI mode */
  tui?: string;

  /** Entry point for both modes */
  both?: string;
}
```

### PluginFeature

```typescript
interface PluginFeature {
  /** Feature description */
  description: string;

  /** Whether the feature is enabled by default */
  default?: boolean;

  /** Entry point for this feature */
  entry?: string;

  /** Additional capabilities required by this feature */
  additional_capabilities?: PluginCapabilities;
}
```

### PluginEngines

```typescript
interface PluginEngines {
  /** Required next-code version range (e.g., ">=0.9.0"). Prefer `nextcode`; dual-read also accepts legacy `jcode`. */
  nextcode?: string;
  /** @deprecated dual-read legacy key — prefer `nextcode` */
  next-code?: string;
}
```

### PluginState

```typescript
type PluginState =
  | "Discovered"
  | "Loading"
  | "Loaded"
  | "Active"
  | "Disabled"
  | "Blocked"
  | { Error: string };
```

### PluginOrigin

```typescript
type PluginOrigin =
  | { NpmPackage: { name: string; version: string } }
  | { LocalFile: { path: string } }
  | { Builtin: { name: string } }
  | { Remote: { url: string } };
```

### PluginId

```typescript
interface PluginId {
  /** Full identifier string (e.g., "npm:my-plugin") */
  as_str(): string;

  /** Short name without prefix (e.g., "my-plugin") */
  short_name(): string;

  /** String representation */
  toString(): string;
}

// Factory methods
declare namespace PluginId {
  function npm(name: string): PluginId;
  function file(path: string): PluginId;
  function bundled(name: string): PluginId;
}
```

### PluginVersion

```typescript
interface PluginVersion {
  /** Plugin version */
  semver: string;

  /** Minimum required next-code version */
  next_code_min_version?: string;

  /** Maximum supported next-code version */
  next_code_max_version?: string;
}
```

---

## Event Types

### PluginEvent

All 28 event types:

```typescript
type PluginEvent =
  // Tool events
  | "PreToolUse"           // 0  - Before tool execution
  | "PostToolUse"          // 1  - After tool execution
  | "PostToolUseFailure"   // 2  - Tool execution failed
  | "ToolExecutionStart"   // 3  - Tool execution begins
  | "ToolExecutionEnd"     // 4  - Tool execution ends

  // Session events
  | "SessionStart"         // 5  - Session begins
  | "SessionEnd"           // 6  - Session ends
  | "SessionSwitch"        // 7  - User switches session
  | "SessionCompact"       // 8  - Session compacted
  | "SessionBeforeCompact" // 9  - Before compaction
  | "SessionShutdown"      // 10 - Session system shutdown

  // Permission events
  | "PermissionRequest"    // 12 - Permission decision needed
  | "PermissionDenied"     // 13 - Permission denied

  // Agent events
  | "AgentStart"           // 14 - Agent starts
  | "AgentEnd"             // 15 - Agent ends

  // Turn events
  | "TurnStart"            // 16 - Turn begins
  | "TurnEnd"              // 17 - Turn ends

  // Message events
  | "MessageStart"         // 18 - Message begins
  | "MessageEnd"           // 19 - Message ends

  // Compact events
  | "PreCompact"           // 20 - Before compaction
  | "PostCompact"          // 21 - After compaction

  // Task events
  | "TaskCreated"          // 22 - Task created
  | "TaskCompleted"        // 23 - Task completed

  // Other events
  | "AutoCompactionStart"  // 24 - Auto-compaction triggered
  | "UserPromptSubmit"     // 25 - User submits prompt
  | "Stop"                 // 26 - Agent stops
  | "Notification";        // 27 - System notification
```

### EventInput Types

Each event has a specific input shape:

```typescript
// Tool events
interface PreToolUseInput {
  event: "PreToolUse";
  tool_name: string;
  tool_input: Record<string, unknown>;
  session_id: string;
}

interface PostToolUseInput {
  event: "PostToolUse";
  tool_name: string;
  tool_input: Record<string, unknown>;
  tool_output: unknown;
  duration_ms: number;
  success: boolean;
  session_id: string;
}

interface PostToolUseFailureInput {
  event: "PostToolUseFailure";
  tool_name: string;
  tool_input: Record<string, unknown>;
  error: string;
  duration_ms: number;
  session_id: string;
}

interface ToolExecutionStartInput {
  event: "ToolExecutionStart";
  tool_name: string;
  tool_input: Record<string, unknown>;
  session_id: string;
}

interface ToolExecutionEndInput {
  event: "ToolExecutionEnd";
  tool_name: string;
  tool_output: unknown;
  duration_ms: number;
  session_id: string;
}

// Session events
interface SessionStartInput {
  event: "SessionStart";
  session_id: string;
  project_dir: string;
  model: string;
  provider: string;
}

interface SessionEndInput {
  event: "SessionEnd";
  session_id: string;
  duration_seconds: number;
  message_count: number;
}

interface SessionSwitchInput {
  event: "SessionSwitch";
  session_id: string;
  target_session_id: string;
}

interface SessionCompactInput {
  event: "SessionCompact";
  session_id: string;
  reason: string;
}

// Permission events
interface PermissionRequestInput {
  event: "PermissionRequest";
  action: string;
  tool_name?: string;
  target?: string;
  session_id: string;
}

// Agent events
interface AgentStartInput {
  event: "AgentStart";
  session_id: string;
  system_prompt: unknown;
  tools: unknown;
}

interface AgentEndInput {
  event: "AgentEnd";
  session_id: string;
  duration_seconds: number;
  message_count: number;
}

// Turn events
interface TurnStartInput {
  event: "TurnStart";
  session_id: string;
  turn_number: number;
  messages: unknown;
}

interface TurnEndInput {
  event: "TurnEnd";
  session_id: string;
  turn_number: number;
  duration_ms: number;
}

// Message events
interface MessageStartInput {
  event: "MessageStart";
  session_id: string;
  role: string; // "user" | "assistant" | "system"
}

interface MessageEndInput {
  event: "MessageEnd";
  session_id: string;
  role: string;
  content: string;
}

// Compact events
interface PreCompactInput {
  event: "PreCompact";
  session_id: string;
  message_count: number;
  token_count: number;
  system_prompt: unknown;
}

interface PostCompactInput {
  event: "PostCompact";
  session_id: string;
  messages_removed: number;
  tokens_saved: number;
}

// Other events
interface UserPromptSubmitInput {
  event: "UserPromptSubmit";
  content: string;
  session_id: string;
}

interface StopInput {
  event: "Stop";
  session_id: string;
  reason: string;
}

interface NotificationInput {
  event: "Notification";
  level: string; // "info" | "warn" | "error"
  message: string;
  session_id?: string;
}

// Union type
type EventInput =
  | PreToolUseInput
  | PostToolUseInput
  | PostToolUseFailureInput
  | ToolExecutionStartInput
  | ToolExecutionEndInput
  | SessionStartInput
  | SessionEndInput
  | SessionSwitchInput
  | SessionCompactInput
  | PermissionRequestInput
  | AgentStartInput
  | AgentEndInput
  | TurnStartInput
  | TurnEndInput
  | MessageStartInput
  | MessageEndInput
  | PreCompactInput
  | PostCompactInput
  | UserPromptSubmitInput
  | StopInput
  | NotificationInput;
```

### EventOutput Types

Events that support modification have output types:

```typescript
interface PreToolUseOutput {
  event: "PreToolUse";
  /** If set, tool is blocked with this reason */
  block?: string;
  /** If set, replaces the tool input */
  modified_input?: Record<string, unknown>;
}

interface PostToolUseOutput {
  event: "PostToolUse";
  /** If set, replaces the tool output */
  modified_output?: unknown;
}

interface PermissionRequestOutput {
  event: "PermissionRequest";
  /** Auto-decision */
  decision?: PermissionDecision;
  /** Explanation message */
  message?: string;
}

interface AgentStartOutput {
  event: "AgentStart";
  /** Lines to append to system prompt */
  additional_system_prompt: string[];
}

interface PreCompactOutput {
  event: "PreCompact";
  /** Modified system prompt */
  system_prompt?: unknown;
  /** Additional instructions */
  instructions?: string;
  /** If true, prevent compaction */
  prevent?: boolean;
}

interface UserPromptSubmitOutput {
  event: "UserPromptSubmit";
  /** If set, replaces the user prompt */
  modified_prompt?: string;
}

interface NotificationOutput {
  event: "Notification";
  /** If true, suppress the notification */
  suppress?: boolean;
  /** If set, replaces the notification message */
  modified_message?: string;
}

interface StopOutput {
  event: "Stop";
  /** Stop reason (can be modified) */
  reason: string;
}

// Union type
type EventOutput =
  | PreToolUseOutput
  | PostToolUseOutput
  | PermissionRequestOutput
  | AgentStartOutput
  | PreCompactOutput
  | UserPromptSubmitOutput
  | NotificationOutput
  | StopOutput;
```

---

## Handler Result Types

### HandlerResult

Returned by event handlers to control event outcome:

```typescript
interface HandlerResult {
  /** Action to take */
  action: HandlerAction;

  /** Optional output data */
  output?: unknown;

  /** Optional error message */
  error?: string;
}
```

### HandlerAction

```typescript
type HandlerAction =
  | "continue"                    // Proceed normally (default)
  | { block: string }             // Block with reason
  | "allow"                       // Explicitly allow
  | "deny"                        // Explicitly deny
  | "error";                      // Signal an error
```

### PermissionDecision

Used in `PermissionRequest` output:

```typescript
type PermissionDecision = "allow" | "deny" | "ask";
```

### ToolDefinition

Used with `nextcode.registerTool()`:

```typescript
interface ToolDefinition {
  /** Tool name (must be unique across all plugins) */
  name: string;

  /** Description shown to the model */
  description: string;

  /** JSON Schema for parameters */
  parameters: JSONSchema;

  /** Handler function invoked when the tool is called */
  handler: (params: Record<string, unknown>) => unknown;
}

// JSON Schema (simplified)
interface JSONSchema {
  type: "object";
  properties: Record<string, JSONSchemaProperty>;
  required?: string[];
}

interface JSONSchemaProperty {
  type: "string" | "number" | "boolean" | "object" | "array";
  description?: string;
  default?: unknown;
  enum?: unknown[];
  items?: JSONSchemaProperty;
  properties?: Record<string, JSONSchemaProperty>;
}
```

---

## Capability Types

### PluginCapabilities

```typescript
interface PluginCapabilities {
  /** Allowed read paths (e.g., ["$HOME/.next-code/data"]) */
  fs_read?: string[];

  /** Allowed write paths */
  fs_write?: string[];

  /** Allowed network hosts (e.g., ["anextcode.github.com"]) */
  network?: string[];

  /** Allow shell command execution */
  shell?: boolean;

  /** Allow registering custom tools */
  register_tools?: boolean;

  /** Allow registering CLI commands */
  register_commands?: boolean;

  /** Allow registering LLM providers */
  register_providers?: boolean;

  /** Allow reading next-code config */
  read_config?: boolean;

  /** Allow writing next-code config */
  write_config?: boolean;

  /** Allowed environment variables */
  env_vars?: string[];

  /** Events the plugin can subscribe to */
  events?: string[];

  /** Allow direct LLM access */
  llm_access?: boolean;

  /** Allow session manipulation */
  session_access?: boolean;
}
```

### CapabilitySet

Used in security chain evaluation:

```typescript
interface CapabilitySet {
  /** Filesystem paths (matched by prefix) */
  fs_paths: string[];

  /** Network hosts (matched by substring) */
  hosts: string[];

  /** Tool names (matched exactly) */
  tools: string[];

  /** Environment variable names (matched exactly) */
  env_vars: string[];

  /** Shell commands (matched exactly) */
  shell_commands: string[];

  /** Config keys (matched exactly) */
  config_keys: string[];

  /** Provider names (matched exactly) */
  providers: string[];
}
```

### CapabilityAction

```typescript
type CapabilityAction =
  | "read"      // Filesystem read
  | "write"     // Filesystem write
  | "execute"   // Shell execution
  | "network"   // Network access
  | "config"    // Config access
  | "session"   // Session access
  | "provider"; // Provider access
```

---

## Security Types

### CapabilityChain

The security evaluation chain:

```typescript
interface CapabilityChain {
  /** Plugin-specific deny rules (highest priority) */
  deny_list: CapabilitySet;

  /** System-wide deny rules */
  global_deny: CapabilitySet;

  /** Plugin-specific allow rules */
  allow_list: CapabilitySet;

  /** Fallback when no rules match */
  global_default: AccessDefault;

  /** Access mode */
  mode: AccessMode;
}
```

### AccessDefault

```typescript
type AccessDefault = "deny" | "allow" | "ask";
```

- `"deny"` -- Deny access by default (most secure).
- `"allow"` -- Allow access by default (least secure).
- `"ask"` -- Prompt user for each access request.

### AccessMode

```typescript
type AccessMode = "all" | "trusted" | "none" | "interactive";
```

- `"all"` -- Normal evaluation through the chain.
- `"trusted"` -- Only explicit deny rules block access.
- `"none"` -- All access denied (kill switch).
- `"interactive"` -- Requires user approval for each access.

### AccessDecision

Result of capability check:

```typescript
type AccessDecision =
  | { Allowed: string }       // Access granted with reason
  | { Denied: string }        // Access denied with reason
  | { NeedsApproval: string }; // Requires user approval
```

---

## Configuration Types

### PluginConfig

Configuration from `config.toml` `[plugin]` section:

```typescript
interface PluginConfig {
  /** Plugins to explicitly enable */
  enable: string[];

  /** Plugins to explicitly disable */
  disable: string[];

  /** Access mode override */
  mode?: string;

  /** If true, fail on any plugin load error */
  fail_closed?: boolean;

  /** Plugin sources */
  sources?: PluginSource[];

  /** Per-plugin settings */
  settings: Record<string, Record<string, unknown>>;

  /** Feature toggles */
  features: Record<string, string[]>;

  /** Per-plugin overrides */
  plugins: Record<string, PluginPerPluginConfig>;

  /** Skip all plugin hooks */
  skip_hooks: boolean;

  /** Force deny all plugin actions */
  force_deny: boolean;
}
```

### PluginSource

```typescript
type PluginSource =
  | { type: "npm"; package: string; version?: string }
  | { type: "file"; path: string }
  | { type: "directory"; path: string };
```

### PluginPerPluginConfig

```typescript
interface PluginPerPluginConfig {
  /** Enable/disable this plugin */
  enable?: boolean;

  /** Handler timeout in milliseconds */
  timeout_ms?: number;
}
```

### SettingSchema

User-configurable setting definitions:

```typescript
type SettingSchema =
  | StringSetting
  | NumberSetting
  | BooleanSetting
  | EnumSetting
  | ArraySetting
  | ObjectSetting;

interface StringSetting {
  type: "string";
  description: string;
  default?: string;
  /** If true, value is masked in output */
  secret?: boolean;
  /** Environment variable to read from */
  env?: string;
  /** Regex pattern for validation */
  pattern?: string;
  /** Maximum string length */
  max_length?: number;
}

interface NumberSetting {
  type: "number";
  description: string;
  default?: number;
  min?: number;
  max?: number;
}

interface BooleanSetting {
  type: "boolean";
  description: string;
  default?: boolean;
}

interface EnumSetting {
  type: "enum";
  description: string;
  default?: string;
  values: string[];
}

interface ArraySetting {
  type: "array";
  description: string;
  default?: unknown[];
  items: SettingSchema;
  max_items?: number;
}

interface ObjectSetting {
  type: "object";
  description: string;
  default?: unknown;
  properties: Record<string, SettingSchema>;
}
```

### DiscoveryPaths

Plugin discovery directories:

```typescript
interface DiscoveryPaths {
  /** Directories to scan for plugin files */
  plugin_dirs: string[];

  /** npm cache directory */
  npm_cache: string;

  /** Tool directories */
  tool_dirs: string[];
}

// Default paths:
// plugin_dirs: ["~/.next-code/plugins"]
// npm_cache:   "~/.next-code/cache/packages"
// tool_dirs:   ["~/.next-code/tools"]
```

---

## Error Types

### PluginError

All possible plugin errors:

```typescript
type PluginError =
  | { InvalidManifest: string }    // Invalid manifest format
  | { NotFound: string }           // Plugin not found
  | { Load: string }               // Failed to load plugin
  | { Runtime: string }            // Runtime error
  | { Eval: string }               // QuickJS evaluation error
  | { QuickJs: string }            // QuickJS runtime error
  | { Transpile: string }          // SWC transpilation error
  | { Timeout: Duration }          // Handler timed out
  | { Capability: string }         // Capability denied
  | { Npm: string }                // npm error
  | { Io: string }                 // I/O error
  | { Serde: string }              // Serialization error
  | { Other: string };             // Other error
```

---

## Internal Types

These types are used internally by the runtime but are useful for understanding the system.

### PreflightResult

Result of static analysis before plugin loading:

```typescript
interface PreflightResult {
  /** Whether the plugin passed all checks (no blocks) */
  passed: boolean;

  /** Non-fatal warnings (logged but plugin still loads) */
  warnings: string[];

  /** Fatal blocks (prevent loading) */
  blocks: string[];

  /** Capabilities declared in the plugin manifest */
  declared_capabilities: PluginCapabilities;

  /** Patterns detected during analysis */
  detected_patterns: string[];

  /** Detailed static analysis breakdown */
  static_analysis: StaticAnalysis;
}
```

### StaticAnalysis

Detailed preflight analysis:

```typescript
interface StaticAnalysis {
  /** Code uses eval() */
  has_eval: boolean;

  /** Code uses dynamic import() */
  has_dynamic_import: boolean;

  /** Code uses fetch() */
  has_fetch: boolean;

  /** Code references process.* */
  has_process_access: boolean;

  /** Detected filesystem access patterns */
  has_fs_access: string[];

  /** Detected network access patterns */
  has_network_access: string[];

  /** Suspicious string literals found */
  suspicious_strings: string[];
}
```

### HandlerSlot

Internal handler registration:

```typescript
// Handlers are registered as async functions that receive
// EventInput and optional EventOutput, returning HandlerResult
type HandlerSlot = (
  input: EventInput,
  output?: EventOutput
) => Promise<HandlerResult>;
```

### DualTimeout

Timeout configuration for sandbox execution:

```typescript
interface DualTimeout {
  /** Timeout for informational events (default: 500ms) */
  info: number;

  /** Timeout for actionable events (default: 5000ms) */
  actionable: number;

  /** Timeout for permission events (default: 3600000ms / 1 hour) */
  permission?: number;
}
```

### RuntimeConfig

QuickJS runtime configuration:

```typescript
interface RuntimeConfig {
  /** Maximum concurrent plugin executions (default: 4) */
  max_concurrent: number;

  /** Maximum runtime instances in pool (default: 8) */
  max_runtimes: number;

  /** Maximum stack size in bytes (default: 512KB) */
  max_stack_size: number;

  /** Memory limit in bytes (default: 50MB) */
  memory_limit: number;

  /** GC threshold in bytes (default: 10MB) */
  gc_threshold: number;
}
```

---

## Type Mapping: Rust to TypeScript

For reference, here is how Rust types map to TypeScript:

| Rust Type | TypeScript Type |
|-----------|----------------|
| `PluginId` | `string` (e.g., `"npm:my-plugin"`) |
| `PluginEvent` | `string` union (e.g., `"PreToolUse" \| "PostToolUse"`) |
| `EventInput` | Tagged union with `event` discriminant |
| `EventOutput` | Tagged union with `event` discriminant |
| `HandlerResult` | `{ action: HandlerAction; output?: unknown; error?: string }` |
| `HandlerAction` | `"continue" \| { block: string } \| "allow" \| "deny" \| "error"` |
| `PermissionDecision` | `"allow" \| "deny" \| "ask"` |
| `AccessDecision` | `{ Allowed: string } \| { Denied: string } \| { NeedsApproval: string }` |
| `AccessMode` | `"all" \| "trusted" \| "none" \| "interactive"` |
| `AccessDefault` | `"deny" \| "allow" \| "ask"` |
| `CapabilityAction` | `"read" \| "write" \| "execute" \| "network" \| "config" \| "session" \| "provider"` |
| `PluginKind` | `"server" \| "tui" \| "both"` |
| `PluginState` | `"Discovered" \| "Loading" \| "Loaded" \| "Active" \| "Disabled" \| "Blocked" \| { Error: string }` |
| `PluginSource` | `{ type: "npm"; package: string } \| { type: "file"; path: string } \| { type: "directory"; path: string }` |
| `SettingSchema` | Tagged union with `type` discriminant |
| `PluginCapabilities` | Object with optional fields |
| `CapabilitySet` | Object with string array fields |
| `serde_json::Value` | `unknown` or `any` |
| `Option<T>` | `T \| undefined` |
| `Vec<T>` | `T[]` |
| `HashMap<K, V>` | `Record<K, V>` |
| `u32`, `u64`, `f64` | `number` |
| `bool` | `boolean` |
| `String`, `&str` | `string` |
| `Duration` | `number` (milliseconds) |
