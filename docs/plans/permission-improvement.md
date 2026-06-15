# Permission System Implementation Plan

> Gap analysis + implementation roadmap for jcode's permission system vs Claude Code.
> Based on thorough research of existing code.

---

## Current Architecture Map

```
Tool call
  │
  ▼
validate_tool_allowed(tool_name)     ← turn_loops.rs, turn_streaming_mpsc.rs, turn_execution.rs
  │
  ▼
dcg_bridge::classify_for_session()   ← dcg_bridge.rs
  │
  ├─ session allow-list check        → Allow (user approved earlier)
  ├─ execution_policy::evaluate_tool → Allow/Deny/Prompt (config rules)
  └─ classify_with_mode()
       ├─ Mode::Default/Mode::Auto → is_legacy_auto_allowed() → Allow
       │                              └─ not in list → Prompt (dialog)
       ├─ Mode::Plan → Engine::evaluate(Read+Fs) → Allow (reads) / Prompt (writes)
       ├─ Mode::AcceptEdits → Engine::evaluate(Write+Fs) → Allow (edits) / Prompt (spawn/network)
       └─ Mode::DontAsk → Engine::evaluate() → Allow (allow-listed) / Prompt (not listed)
  │
  ▼
BridgeDecision::Prompt
  │
  ├─ BusEvent::PermissionRequested    → TUI dialog renders
  └─ ToolResult(is_error=true)       → transcript shows tool failed
       User approves → session allow-list set → model retries → auto-allows
```

### Key Components

| Component | File | Role |
|-----------|------|------|
| `PermissionMode` enum | `jcode-agent-runtime/src/permission.rs` | 6 modes mirroring dcg_core::Mode |
| `dcg_bridge.rs` | `jcode-app-core/src/dcg_bridge.rs` (1326 lines) | Bridge between jcode actions and dcg-core engine |
| `SafetySystem` | `jcode-base/src/safety.rs` | Legacy system for ambient permission requests |
| `execution_policy.rs` | `jcode-app-core/src/execution_policy.rs` (847 lines) | Configurable per-command rules |
| `permission dialog` | `ui_overlays.rs:650-699` | TUI permission dialog rendering |
| `dialog keyboard` | `input.rs:1786-1887` | Arrow/Enter/Esc handling |
| `dialog bus handler` | `local.rs:253-278` | BusEvent → pending_permission_* fields |
| `validate_tool_allowed` | `turn_execution.rs:548-634` | Entry point that publishes bus event + returns error |

### State Variables

| Variable | Type | File | Purpose |
|----------|------|------|---------|
| `GLOBAL_MODE` | `Mutex<Mode>` | `dcg_bridge.rs` | Current permission mode |
| `ENGINE` | `Engine` | `dcg_bridge.rs` | dcg-core evaluation engine |
| `SESSION` | `Mutex<Session>` | `dcg_bridge.rs` | dcg-core session (deny counts, allow-once) |
| `SESSION_MODES` | `HashMap<String, Mode>` | `dcg_bridge.rs` | Per-session mode overrides |
| `SESSION_ALLOWED_ACTIONS` | `HashMap<String, HashSet<String>>` | `dcg_bridge.rs` | Per-session allow list |
| `pending_permission_*` | 6 fields on App | `app.rs` | TUI dialog state |

---

## Gap 1: Tool-Specific Permission Dialogs

### Current (jcode)

1 generic dialog for ALL tools:
```
╭─ Permission request: bash ──╮
│  bash (git status)           │
│                              │
│  ❯ ✔ Approve  ◯ Approve all  ◯ Always allow  ◯ Deny
└──────────────────────────────┘
```

### Target (Claude Code)

Tool-specific dialogs with contextual information:

| Tool | CCB Component | Shows |
|------|---------------|-------|
| `bash` | `BashPermissionRequest.tsx` | Full command, cwd, OS, sandbox suggestion |
| `file_edit` | `FileEditPermissionRequest.tsx` | Diff (old vs new), file path, line numbers |
| `file_write` | `FileWritePermissionRequest.tsx` | File path, old content, new content |
| `web_fetch` | `WebFetchPermissionRequest.tsx` | URL, content preview |
| `powershell` | `PowerShellPermissionRequest.tsx` | Full command |
| `sed_edit` | `SedEditPermissionRequest.tsx` | Diff (rendered same as file edit) |
| `notebook_edit` | `NotebookEditPermissionRequest.tsx` | Cell changes |
| `skill` | `SkillPermissionRequest.tsx` | Skill name + args |
| `sandbox` | `SandboxPermissionRequest.tsx` | Sandbox details |
| `plan_mode` | `EnterPlanModePermissionRequest.tsx` | Scope + reason |
| `fallback` | `FallbackPermissionRequest.tsx` | Generic |

### Implementation Plan

**Step 1:** Refactor `draw_permission_dialog_overlay` to dispatch to tool-specific renderers

**Files:**
- `ui_overlays.rs` — add tool-specific draw functions
- `mod.rs` (TuiState trait) — add `pending_permission_input()` to pass tool input
- `app.rs` — store `pending_permission_input: Option<Value>`

**Code Structure:**
```rust
pub(super) fn draw_permission_dialog_overlay(frame, area, app) {
    let tool = app.pending_permission_tool();
    match tool {
        Some("bash") => draw_bash_permission_dialog(frame, area, app),
        Some("edit") | Some("hashline_edit") => draw_edit_permission_dialog(frame, area, app),
        Some("write") => draw_write_permission_dialog(frame, area, app),
        Some("webfetch") => draw_webfetch_permission_dialog(frame, area, app),
        _ => draw_generic_permission_dialog(frame, area, app), // fallback
    }
}
```

**Step 2:** Bash dialog — show command with prefix truncation

```rust
fn draw_bash_permission_dialog(frame, area, app) {
    // Command from pending_permission_input
    let command = app.pending_permission_input()
        .and_then(|v| v.get("command"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    // Show simplified command prefix (like CCB's getSimpleCommandPrefix)
    let prefix = get_first_word_prefix(command);
    // Render:
    // ╭─ Permission: bash ────────────────────╮
    // │  git status                            │
    // │  Working dir: ~/Projects/jcode         │
    // │  OS: darwin                            │
    // │                                       │
    // │  ❯ ✔ Approve  ◯ Approve all  ◯ Always allow  ◯ Deny
    // └───────────────────────────────────────┘
}
```

**Step 3:** Edit dialog — show diff (reuse existing `render_diff` or `draw_file_diff`)

```rust
fn draw_edit_permission_dialog(frame, area, app) {
    // File path, old_string, new_string from pending_permission_input
    // Render inline diff:
    // ╭─ Permission: edit ────────────────────╮
    // │  File: src/main.rs                    │
    // │  ─ a/src/main.rs                      │
    // │  ─ old line                           │
    // │  + new line                           │
    // │                                       │
    // │  ❯ ✔ Approve  ◯ Approve all  ◯ Always allow  ◯ Deny
    // └───────────────────────────────────────┘
}
```

**Step 4:** Write dialog — show file path + content preview

```rust
fn draw_write_permission_dialog(frame, area, app) {
    // File path, content from pending_permission_input
    // Show path + first N lines of content
}
```

**Step 5:** WebFetch dialog — show URL

```rust
fn draw_webfetch_permission_dialog(frame, area, app) {
    // URL from pending_permission_input
}
```

---

## Gap 2: Worker Badge (Subagent Attribution)

### Current (jcode)

Dialog shows tool name + reason only — no indication of WHICH agent requested.

### Target (Claude Code)

WorkerBadge shows `session_id` or friendly name of the requesting agent:
```
╭─ Permission: bash [subagent: Code Reviewer] ─╮
```

### Implementation Plan

**Step 1:** Add `worker_id` / `worker_name` to `PermissionRequested` bus event

**File:** `jcode-base/src/bus.rs`
```rust
pub struct PermissionRequested {
    pub session_id: String,
    pub worker_session_id: Option<String>,  // NEW: subagent's session
    pub worker_name: Option<String>,        // NEW: subagent's display name
    pub tool_name: String,
    pub reason: String,
    pub allow_once_code: String,
    pub alternatives: Vec<String>,
}
```

**Step 2:** Populate in `validate_tool_allowed`

**File:** `turn_execution.rs`
```rust
// Add worker info if this is a subagent call
let worker_session_id = self.session.is_subagent.then(|| self.session.id.clone());
let worker_name = self.session.subagent_name.clone();
```

**Step 3:** Store + render in TUI

**Files:**
- `app.rs` — `pending_permission_worker_name: Option<String>`
- `local.rs` — set from bus event
- `ui_overlays.rs` — render in dialog title (append `[agent: name]`)

---

## Gap 3: Denial Tracking

### Current (jcode)

No tracking. User can deny infinitely without any change in behavior.

### Target (Claude Code)

Track consecutive + total denials. After 3 consecutive or 20 total, fallback to prompting mode.

**References:** CCB `denialTracking.ts`:
```typescript
const DENIAL_LIMITS = { maxConsecutive: 3, maxTotal: 20 };
```

### Implementation Plan

**Step 1:** Add denial tracking state to dcg_bridge

**File:** `dcg_bridge.rs`
```rust
static DENIAL_TRACKING: LazyLock<Mutex<DenialTrackingState>> = ...;

struct DenialTrackingState {
    consecutive_denials: HashMap<String, u32>,  // per-session
    total_denials: HashMap<String, u32>,         // per-session
}

const MAX_CONSECUTIVE_DENIALS: u32 = 3;
const MAX_TOTAL_DENIALS: u32 = 20;
```

**Step 2:** Record denial on Deny press

**File:** `input.rs` (dialog keyboard handler)
```rust
KeyCode::Enter if selected == 3 => {
    // Deny
    crate::dcg_bridge::record_denial(&session_id, &tool_name);
    if crate::dcg_bridge::should_fallback_to_prompt(&session_id) {
        // Show warning: "You've denied this tool many times..."
        // Fall back to prompting mode
    }
    app.reset_permission_dialog();
}
```

**Step 3:** Check in `classify_for_session`

**File:** `dcg_bridge.rs`
```rust
pub fn classify_for_session(action, session_id) -> BridgeDecision {
    // If denial limit exceeded for this session, show ask (always prompt)
    if denial_limit_exceeded(session_id, action) {
        return BridgeDecision::Prompt {
            reason: "You've denied this tool before. Review carefully.".into(),
            ...
        };
    }
    // ... normal flow
}
```

---

## Gap 4: Permission Explainer

### Current (jcode)

Generic reason: `"Tool action 'bash' requires permission in current mode Default"`

### Target (Claude Code)

AI-generated explanation with risk level LOW/MEDIUM/HIGH:
```
╭─ Permission: bash ─────────────────╮
│  git push origin main              │
│  Risk: HIGH — pushes to remote     │
│  You are about to push code to     │
│  the remote repository. This will  │
│  modify the project's history.     │
│                                    │
│  ❯ ✔ Approve  ...  ◯ Deny
└────────────────────────────────────┘
```

**References:** CCB `permissionExplainer.ts`:
```typescript
generatePermissionExplanation(action, command, input) → { 
    riskLevel: "LOW" | "MEDIUM" | "HIGH", 
    reasoning: string 
}
```

### Implementation Plan

**Step 1:** Add `generate_permission_explanation` in agent runtime

**File:** New file `jcode-agent-runtime/src/permission_explainer.rs`

```rust
pub fn explain_action(tool_name: &str, input: &Value) -> PermissionExplanation {
    // Rule-based: classify by tool type + input patterns
    match tool_name {
        "bash" => explain_bash(input),
        "edit" | "write" => explain_file_edit(input),
        "webfetch" => explain_webfetch(input),
        _ => explain_generic(tool_name),
    }
}

struct PermissionExplanation {
    risk: RiskLevel,        // Low | Medium | High
    summary: String,        // 1-line summary
    details: Vec<String>,   // bullet points
}

enum RiskLevel { Low, Medium, High }
```

**Step 2:** Wire into `BridgeDecision::Prompt` — add explanation fields

**File:** `dcg_bridge.rs`
```rust
pub enum BridgeDecision {
    Allow,
    Prompt {
        reason: String,
        allow_once_code: String,
        alternatives: Vec<String>,
        explanation: Option<PermissionExplanation>,  // NEW
    },
    Deny { ... },
}
```

**Step 3:** Render in dialog

**File:** `ui_overlays.rs` — show risk badge + explanation text

---

## Gap 5: Mode Transition Safety

### Current (jcode)

`cycle_mode()` blindly switches to next mode. Switching from BypassPermissions to Auto keeps all dangerous permissions active.

### Target (Claude Code)

`permissionSetup.ts`: `stripDangerousPermissionsForAutoMode()` — when entering Auto mode, remove dangerously broad allow rules.

### Implementation Plan

**Step 1:** Add dangerous permission detection

**File:** `dcg_bridge.rs`
```rust
fn is_dangerous_allow_rule(tool: &str) -> bool {
    matches!(tool, "bash" | "write" | "edit" | "patch" | "webfetch" | "subagent")
}

fn strip_dangerous_permissions_for_mode(session_id: &str, target_mode: Mode) {
    if target_mode == Mode::Auto {
        if let Ok(mut guard) = SESSION_ALLOWED_ACTIONS.lock() {
            if let Some(actions) = guard.get_mut(session_id) {
                actions.retain(|a| !is_dangerous_allow_rule(a));
            }
        }
    }
}
```

**Step 2:** Call before mode switch

**File:** `dcg_bridge.rs` — in `cycle_mode()`:
```rust
pub fn cycle_mode() -> Mode {
    let next = ...;
    strip_dangerous_permissions_for_all_sessions(next);
    *guard = next;
    next
}
```

---

## Gap 6: Plan Mode Dialog

### Current (jcode)

No dialog when entering/exiting Plan mode.

### Target (Claude Code)

- EnterPlanMode: shows scope + reason for entering plan mode
- ExitPlanMode: shows changes made during plan mode

### Implementation Plan

**Step 1:** Add plan mode dialog rendering

**File:** `ui_overlays.rs`
```rust
fn draw_enter_plan_mode_dialog(frame, area, app) {
    // ╭─ Entering Plan Mode ───────────────╮
    // │  Plan mode is read-only. Writes will│
    // │  be denied.                         │
    // │                                     │
    // │  ❯ Enter Plan Mode    ◯ Cancel
    // └─────────────────────────────────────┘
}
```

**Step 2:** Wire into Alt+P cycle or `/permissions plan`

**File:** `input.rs` — when cycling TO Plan mode:
```rust
KeyCode::Char('p') if modifiers == KeyCode::ALT => {
    let next = crate::dcg_bridge::peek_next_mode();
    if next == Mode::Plan {
        // Show plan mode dialog first
        app.show_plan_mode_confirmation = true;
    }
}
```

---

## Gap 7: Permission Request Timeout

### Current (jcode)

Pending permission dialog stays open indefinitely.

### Target

Auto-deny after timeout (e.g., 30 seconds for automatic/headless agents).

### Implementation Plan

**Step 1:** Add timestamp to `pending_permission_tool`

**File:** `app.rs`
```rust
pub pending_permission_at: Option<Instant>,
```

**Step 2:** Check expiry before rendering

**File:** `local.rs` (bus handler) or `ui.rs` (render check):
```rust
if let Some(at) = app.pending_permission_at {
    if at.elapsed() > Duration::from_secs(30) {
        // Auto-deny
        app.reset_permission_dialog();
        // Optionally notify: "Permission request timed out"
    }
}
```

---

## Summary

### Implementation Priority

| # | Feature | Effort | Complexity | Impact | Dependencies |
|---|---------|--------|------------|--------|--------------|
| **P0** | Tool-specific dialogs | 2-3 days | Medium | High - user sees contextual info per tool | `pending_permission_input` pass-through |
| **P0** | Worker badge | 0.5 day | Low | Medium - know WHICH agent asked | Bus event field + TUI render |
| **P1** | Denial tracking | 1 day | Low | Medium - prevents endless deny loop | Static state + dialog warning |
| **P1** | Permission explainer | 2 days | Medium | Medium - explains WHY | BridgeDecision + dialog rendering |
| **P2** | Plan mode dialog | 0.5 day | Low | Low | UI overlay + mode detection |
| **P2** | Mode transition safety | 0.5 day | Low | Low | `cycle_mode` + strip logic |
| **P3** | Permission timeout | 0.5 day | Low | Low | Timestamp + auto-deny |

### Files That Need Changes

| File | P0 | P1 | P2 | P3 |
|------|----|----|----|----|
| `ui_overlays.rs` | ✅ | ✅ | ✅ | — |
| `input.rs` | — | ✅ | ✅ | — |
| `app.rs` | ✅ | — | ✅ | ✅ |
| `local.rs` | ✅ | — | — | ✅ |
| `tui_state.rs` | ✅ | — | — | — |
| `mod.rs` (TuiState) | ✅ | — | — | — |
| `turn_execution.rs` | ✅ | — | — | — |
| `bus.rs` | ✅ | — | — | — |
| `dcg_bridge.rs` | — | ✅ | ✅ | — |
| `permission_explainer.rs` (NEW) | — | ✅ | — | — |
| `permission.rs` | — | — | — | — |

### First Step (P0)

**1.** Add `pending_permission_input: Option<Value>` to App and TuiState trait  
**2.** Populate from tool arguments in `validate_tool_allowed`  
**3.** Dispatch to tool-specific `draw_*_permission_dialog()`  
**4.** Implement `draw_bash_permission_dialog()` with command display  
**5.** Implement `draw_edit_permission_dialog()` with inline diff  
**6.** Implement `draw_write_permission_dialog()` with file path  
**7.** Implement `draw_webfetch_permission_dialog()` with URL  
**8.** Add worker badge to dialog title  
