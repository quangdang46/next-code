# Spike D: Permissions, Tool Exec, File I/O — Grok vs Next-Code

## Date: 2026-07-17
## Status: ✅ Complete

---

## 1. Permission / Approval Flow

### Grok (via ACP)

```
Agent Leader (separate process)
  ├── Agent decides to execute a tool
  ├── Sends acp::RequestPermissionRequest to Pager (client side)
  │     ├── request.session_id
  │     ├── request.tool_name + args + options
  │     └── request.meta (bash highlights, MCP scope, etc.)
  │
  ▼
Pager (AppView)
  ├── handle_permission_request() → enqueue on agent's permission_queue
  ├── User sees modal: "Allow this tool execution?"
  │     ├── AllowOnce 
  │     ├── AllowAlways (YOLO mode)
  │     └── RejectAlways (managed policy)
  │
  └── permission response sent back via ACP
        └── acp::RequestPermissionOutcome::Selected(option_id)
             → agent leader executes or blocks the tool
```

**YOLO mode:** Pager auto-approves `AllowOnce` when YOLO is on → no user interaction needed.  
**Key dependency:** `xai_grok_workspace::permission` — shared code for permission scoping (bash highlights, default_always_allow_scope, MCP tool permissions).

### Next-Code

Next-code's permission model is unclear from the crate structure. The **tool execution** is in-process:
```
Agent runtime → tool registry → tool execution → result
```

There is likely no "ask user permission" modal in next-code's server — tools run automatically or are configured via allowed tool lists.

### Adapter Strategy

| Permission Aspect | Grok | Next-Code | Strategy |
|------------------|------|-----------|----------|
| **Permission modal** | `PermissionRequest` UI in pager | Not present | ✅ **Keep** — pager's permission UI stays |
| **YOLO mode** | `~/.grok/config.toml` → auto-approve | Not present | ✅ **Keep** — pager's yolo setting |
| **Permission source** | ACP leader sends permission requests | next-code tool exec is in-process | 🔴 **Shim** — replace ACP request with local tool approval check |
| **Always-allow scope** | `xai_grok_workspace::permission` | Not present | 🟡 Stub or simplify |
| **MCP permission scope** | Same as above | Not present | 🟡 Stub |
| **Managed policy** | xAI OTA policy override | Not present | 🟢 Remove |

---

## 2. Tool Execution Flow

### Grok (via ACP)

```
Pager sends user input → ACP to agent leader
Agent leader processes → generates tool calls → sends ACP events back:
  ├── ToolCallStart (tool_name, args) → pager creates ToolCallBlock
  ├── ToolCallOutput (stream) → pager appends to ToolCallBlock
  └── ToolCallEnd → pager marks block complete

For tool results:
  ├── ACP RequestPermission → pager user approval
  ├── Pager sends approval → ACP to leader
  └── Leader executes tool → ACP ToolCallResult back to pager
```

**Grok leader** is a separate binary (`grok serve` or `grok pager --with-server`) that manages:
- Agent (Claude Code, custom xAI models, etc.)
- Tool execution (shell, edit, read, search, file IO)
- Session state
- Provider/API key management
- Sandbox (containerized execution)

### Next-Code (in-process)

```
ServerRuntime
  ├── agent::Agent (handles input, generates responses via providers)
  ├── tool::Registry (defines available tools)
  ├── provider::Provider (model API calls)
  └── client handles everything in-process
```

Tool execution in next-code:
- Agent decides to use a tool → calls `tool::Registry::execute(name, args)`  
- Result returned directly to agent → agent continues → sends back Text response
- Everything is in the same process, no ACP needed

### Tool Exec Comparison (ACP vs in-process)

| Tool | Grok ACP Method | next-code equivalent |
|------|----------------|---------------------|
| **Shell command** | `terminal/create` + `terminal/output` + `terminal/wait` | Tool execute "bash" |
| **Read file** | `fs/read_text_file` | Tool execute "read" |
| **Write file** | `fs/write_text_file` | Tool execute "edit" / "write" |
| **Search** | ACP tool_call + result | Tool execute "search" |
| **List directory** | ACP tool_call + result | Tool execute "list_dir" |

**Key difference:**
- Grok separates permission (ask user) from execution (run after approval)
- Next-code does everything in one flow
- The shim needs to **intercept** next-code's tool execution before it runs → check with pager permission system → execute if approved

### Shim approach:

```rust
// Instead of: next_code_server.execute_tool(name, args)
// The shim does:
// 1. Create acp::RequestPermissionRequest from the tool call
// 2. Send to pager's permission queue
// 3. Wait for user approval (or YOLO auto-approve)
// 4. If approved → actually run tool
// 5. Return result as ACP event
```

This means the shim needs to block the tool execution pipeline until the pager's permission system responds. This requires either:
- An async channel (pager sends approval → shim receives → executes)
- Or skip permission entirely (YOLO mode) and just execute directly

---

## 3. File I/O

### Grok

- Via ACP: `fs/read_text_file`, `fs/write_text_file` are permission-gated
- Read is typically auto-approved (YOLO)
- Write is gated (user must approve)
- File paths are resolved relative to the workspace root
- Bash command `bash_selection_count` determines scope

### Next-Code

- Tool executions happen directly on the filesystem
- File read/write is un-gated (or via tool registry config)
- Same filesystem, same workspace

### Shim

The shim just needs to:
- Translate file paths (same workspace, no change)
- Route through permission (if not YOLO) → then execute using next-code's tool registry
- Subagent/shared session file I/O → route path correctly

---

## 4. Summary — What to Keep vs Shim vs Delete

### ✅ Keep as-is from pager:
- Permission modal UI (`acp_handler/permissions.rs`, `dispatch/permissions.rs`)  
- Permission queue + stash prompt
- YOLO mode auto-approve
- Permission display (bash highlights, MCP scope, options)

### 🟡 Shim / Adapt:
- `acp::RequestPermissionRequest` → local tool approval check
- `acp::PermissionOptionId` response → execute tool in next-code
- `xai_grok_workspace::permission` → inline or simplify

### 🟢 Delete (xAI-specific):
- Managed policy override
- SuperGrok upsell / credit-limit checks
- Remote campaign config permissions
