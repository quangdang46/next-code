# Deepen — Merge bundle hooks/MCP into registries (2026-07-22)

**ID:** D4 · **Priority:** P1 · Phase 1.5 / Phase 2  
**Status:** Design contract (docs only — **not** implement approval)  
**Parent:** Master plan Phase 1 §3  
**Evidence:** [inventory](../20260722-nextcode-extension-inventory.md) GAP rows for `hooks/hooks.json` / `.mcp.json`  
**Paired honesty:** [`20260722-bundle-counts-honesty.md`](./20260722-bundle-counts-honesty.md) (D3 — Decision B until this lands)  
**Reload:** [`20260722-hook-registry-reload.md`](./20260722-hook-registry-reload.md) (D7)  
**Trust:** [`20260722-trust-gate-design.md`](./20260722-trust-gate-design.md) (D1)  
**Enable gate:** [`20260722-plugins-state-skill-gate.md`](./20260722-plugins-state-skill-gate.md) (D2)  
**Manifest (later):** [`20260722-plugin-manifest-abi-v1.md`](./20260722-plugin-manifest-abi-v1.md) (D8)  
**Security:** [`20260722-argv-plugin-security.md`](./20260722-argv-plugin-security.md) (D13)  
**Prior deferral:** `docs/plans/PLAN-20260721-hooks-follow-opencode.md` Phase 2 item 2  

---

## Summary (read first)

Bundle plugins declare `hooks/hooks.json` and `.mcp.json`. Face **counts** them as Active/Blocked via `src/cli/face_plugins.rs` → `to_info`. Runtime **does not** merge into `next-code-hooks` / MCP client → UI lie ([inventory](../20260722-nextcode-extension-inventory.md)).

| Phase | Name | Scope |
|-------|------|-------|
| **1** | Honesty demote (D3 B) | Statuses/strings honest; no merge |
| **1.5** | Merge ABI + importer | Parse Grok-shaped JSON; compile into existing registries; enable+trust; provenance |
| **2** | Package manifest | `next-code-plugin.toml` `[[hooks]]` / mcp resources — same backend, new front-end parser (D8) |

**This file specifies Phase 1.5 (Option A wiring).** Land D3 B first if 1.5 slips. Do **not** invent a third parallel runtime.

---

## 1. Goal

When a bundle plugin is **enabled** (and **trusted** if project-scoped executable), its declared hooks/MCP become **real** runtime entries:

| Surface | Target runtime | Must not invent |
|---------|----------------|-----------------|
| Hooks | `crates/next-code-hooks` registry + dispatcher | Second hook runtime / OpenCode JS in-process |
| MCP | `crates/next-code-base/src/mcp` (`McpConfig` / `McpManager`) | Face-only fake servers |

---

## 2. Today’s gaps (verified)

### 2.1 Discovery without load

| Step | Code | Behavior |
|------|------|----------|
| Recognize plugin | `src/cli/face_plugins.rs` → `looks_like_plugin_dir` | `hooks/hooks.json` or `.mcp.json` enough |
| Count | `has_hooks`, `mcp_server_count ∈ {0,1}` | Existence only (boolean-as-1) |
| List ACP | `plugins_list_payload` → `to_info` | Fake `HookStatus::Active` / `McpStatus::Active` when enabled |
| Hooks runtime | `docs/HOOKS.md` layers | User/project/env `hooks.toml` only |
| MCP runtime | `McpConfig::load_for_dir` / `load_project_locals` | `~/.next-code/mcp.json` + project `.next-code/mcp.json`, `.mcp.json`, `.claude/mcp.json` — **not** plugin roots |

Inventory: “Bundle hooks / MCP → UI counts only.”

### 2.2 What Face already does for real hooks/MCP

| Tab | ACP | Backend |
|-----|-----|---------|
| `/hooks` | `x.ai/hooks/list`, `x.ai/hooks/action` | Real `hooks.toml` → `HookInfo` |
| `/mcp` | `x.ai/mcp/list` | Real `McpConfig` catalog |
| `/plugins` | `x.ai/plugins/list` | Discovery metadata only for hooks/MCP files |

Merge must make Plugins-declared contributions appear (with provenance) on `/hooks` and `/mcp`.

### 2.3 Type comments vs code

`crates/xai-hooks-plugins-types/src/lib.rs` says `HookStatus` / `McpStatus` derive from trust + file flags. `to_info` uses **enable** only. After merge, statuses must mean **loaded ∧ gated**, aligning D3 Decision A.

---

## 3. Phase 1.5 — Hooks merge ABI

### 3.1 Input path

```text
<plugin-root>/hooks/hooks.json
```

Already recognized by `looks_like_plugin_dir` / `discover_in_parent`.

### 3.2 Input shape (Grok-compatible)

Documented for Grok in `crates/xai-grok-pager/docs/user-guide/10-hooks.md`. Canonical example for next-code compile:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "python3 hooks/pre_tool_use.py",
            "timeout": 5000
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "hooks": [
          { "type": "command", "command": "notify-session.sh" }
        ]
      }
    ]
  }
}
```

Rules:

- Outer key `hooks` → event name → array of matcher groups.
- Each group: optional `matcher`, nested `hooks[]` of `{ type, command, timeout?, … }`.
- Phase 1.5 imports `type: "command"` (required). `type: "http"` maps to v2 `type = "http"` when URL fields present; else skip + log.
- Relative `command` paths resolve against **plugin root**, then become absolute before spawn.
- Reject path segments that escape plugin root (`..`) — D13.

### 3.3 Event name mapping

Use `docs/HOOKS.md` OpenCode alias table + Grok camelCase names. Parse via `HookEvent::parse` patterns (`crates/next-code-hooks` tests: snake/kebab/space).

| Incoming (JSON key) | next-code `HookEvent` | Action |
|---------------------|------------------------|--------|
| `PreToolUse` | `PreToolUse` | Import |
| `PostToolUse` | `PostToolUse` | Import |
| `SessionStart` / `session.created` | `SessionStart` | Import |
| `tool.execute.before` | `PreToolUse` | Import (alias) |
| `tool.execute.after` | `PostToolUse` | Import (alias) |
| `SubagentStop` / `SubagentEnd` | `SubagentStop` | Import (config OK; dispatch may still be unwired — same as `hooks.toml`) |
| Unknown | — | **Skip + log**; do not crash session |

### 3.4 Compile target (v2 handler)

Same logical shape as `hooks.toml`:

```toml
[[events.PreToolUse]]
type = "command"
enabled = true
command = ["python3", "/abs/plugin/hooks/pre_tool_use.py"]
matcher = "Bash"
timeout_secs = 5
```

**Preferred persistence model:** **in-memory compile** at registry load / reload — do **not** require writing into `~/.next-code/hooks.toml` as source of truth (optional debug dump OK). Disable then removes contributions without mutating user TOML.

Face “Add” (`hooks_add` merge TOML into user hooks) remains a **user** import path — distinct from automatic plugin compile-in.

### 3.5 Provenance (required)

```text
provenance = "plugin:<plugin_id>"
source_path = <abs path to hooks.json>
source_dir  = <plugin-root> or <plugin-root>/hooks
face_id     = "plugin/<plugin_id>/PreToolUse[<i>]"
```

`plugin_id` = existing `PluginInfo.id` (`"<scope>/<hex8>/<name>"` from `face_plugins.rs` → `plugin_id`).

Face `/hooks` grouping should show plugin source like user/project TOML groups (`HookInfo.source_dir`).

### 3.6 Gates before spawn (order)

1. Kill-switch `DISABLE_NEXT_CODE_HOOKS` (existing).
2. Plugin **enable** — `~/.next-code/plugins-state.json` via `is_enabled`; disabled → strip all `plugin:<id>` handlers.
3. **Trust** — project-scoped plugin roots require trust before command spawn (D1). User-global `~/.next-code/plugins/**` treated as user-chosen (trusted with install).
4. Per-handler enabled flag if JSON supports it; else default true.

### 3.7 Load / reload triggers

| Trigger | Behavior |
|---------|----------|
| Session start / hooks config load | Discover enabled plugins; parse JSON; append handlers |
| `x.ai/plugins/action` enable/disable | Reload registry (atomic swap — D7) |
| `x.ai/hooks/action` Reload | Re-read TOML layers **and** recompile plugin JSON |
| Plugin uninstall | Drop provenance |

In-flight PreToolUse: finish or timeout; next event sees new set (D7).

### 3.8 Worked compile example

Plugin root: `~/.next-code/plugins/policy-demo/`  
`hooks/hooks.json` contains one PreToolUse → `python3 hooks/pre_tool_use.py` matcher `Bash`.

After enable:

1. Resolve command → `["python3", "/home/u/.next-code/plugins/policy-demo/hooks/pre_tool_use.py"]` (or Windows equivalent).
2. Register under `HookEvent::PreToolUse` with provenance `plugin:user/<hex>/policy-demo`.
3. `next-code hooks list` / Face `/hooks` shows the handler.
4. Tool Bash with `rm -rf /` → runner exit 2 → `HookResult::Blocked` (`execute.rs` → `interpret_exit_code`).

Disable plugin → handler absent before next PreToolUse (D7 exit).

### 3.9 Failure modes (hooks)

| Issue | Behavior |
|-------|----------|
| Missing file | No hooks (`has_hooks=false`) |
| Invalid JSON | Skip plugin hooks + structured log; session continues |
| Unknown event / type | Skip entry + log |
| Empty command / path escape | Reject entry (D13) |
| Re-enable twice | Idempotent — no duplicate handlers (D7) |

---

## 4. Phase 1.5 — MCP merge ABI

### 4.1 Input path

```text
<plugin-root>/.mcp.json
```

### 4.2 Input shape

Same family as project `.mcp.json` / `~/.next-code/mcp.json`:

```json
{
  "mcpServers": {
    "demo-fs": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "demo-http": {
      "url": "https://example.com/mcp",
      "headers": { "Authorization": "Bearer …" }
    }
  }
}
```

Reuse `McpConfig` / `McpServerConfig` in `crates/next-code-base/src/mcp/protocol.rs`. Drop unconnectable rows with existing `is_connectable` log.

### 4.3 Merge layers and precedence

**Today (no plugins)** — `load_project_locals` / `load_catalog_for_dir`:

```text
user ~/.next-code/mcp.json
  → project .next-code/mcp.json
  → project .mcp.json
  → project .claude/mcp.json
(later same-name overrides earlier within project locals)
```

**Phase 1.5 addition:**

```text
… existing layers …
  → enabled plugin .mcp.json contributions
```

**Frozen duplicate-name precedence:**

```text
user > project (.next-code / .mcp.json / .claude) > plugin
```

Multiple plugins: sort by `plugin_id` ascending; **last wins** (match “later override” project style); log each override.

`McpServerInfo.config_source` (`xai-hooks-plugins-types`):

```text
Some(format!("plugin: {}", plugin.name))
```

### 4.4 Gates

1. Plugin enabled — else strip.
2. Optional MCP trust (`--require-mcp-trust` / `mcp/trust.rs`) — project plugin MCP as project-local executables when required.
3. Connectable filter unchanged.

### 4.5 Catalog vs connect

- Face `/mcp`: `load_catalog_for_dir` includes plugin rows.
- Connect: `McpManager` `load_for_dir` uses same merged set.
- Reload after plugin toggle.

### 4.6 Failure modes (MCP)

| Issue | Behavior |
|-------|----------|
| Invalid JSON | Skip + log |
| Duplicate name | Precedence §4.3; log |
| Disabled plugin | Strip all contributed servers |
| Secrets in file | Document risk; prefer env indirection |

---

## 5. Internal compile API sketch (not public package ABI)

```text
fn compile_plugin_hooks(plugin: &Discovered) -> Result<Vec<CompiledHandler>, CompileError>
fn compile_plugin_mcp(plugin: &Discovered) -> Result<McpConfig, CompileError>

struct CompiledHandler {
  event: HookEvent,
  handler: HandlerConfig,   // existing command/http shape
  provenance: String,       // plugin:<id>
  source_path: PathBuf,
}
```

Call sites:

- Hooks registry build after TOML layers (append).
- MCP `load_catalog_for_dir` / `load_for_dir` after project locals (append with precedence).

Phase 2 (D8): `[[hooks]]` / `[[resources]] mcp` feed the **same** functions after a different parse front-end.

---

## 6. Interaction with honesty (D3)

| Stage | Counts / Face |
|-------|----------------|
| Before 1.5 | Decision B — declared not loaded; never emit Active for file-only |
| After 1.5 | Decision A — `hooksLoaded` / `mcpLoaded`; Active only if loaded ∧ enabled ∧ trust OK |
| Disabled after 1.5 | Strip → `None` (preferred default) |

`hook_count` / `mcp_server_count` after merge = **loaded** entry counts (parsed), not boolean file existence.

`build_plugin_fields` (`extensions_modal.rs`) may again show `"{n} hooks"` / `"{n} MCP servers"` **only** when loaded.

---

## 7. Security boundaries

| Rule | Why |
|------|-----|
| Resolve commands under plugin root or absolute allowlist | Path traversal (D13) |
| Project plugins need trust before argv | RCE (D1) |
| Disabled plugin = zero spawn surface | D2 |
| No new prompt-inject channel from hook stdout | D0 bare-host |

---

## 8. Docs updates (implement PR)

| Doc | Update |
|-----|--------|
| `docs/plugins.md` | Declared → merged when enabled (+ trust) |
| `docs/HOOKS.md` | Remove “deferred”; provenance + reload |
| `docs/CONFIG_REFERENCE.md` if MCP layers listed | Add plugin layer |
| Face empty states | `/hooks` may show `plugin:…` groups |

---

## 9. Acceptance tests (design)

| ID | Scenario | Pass |
|----|----------|------|
| MH-01 | Enabled plugin + hooks.json PreToolUse deny | Fires via next-code-hooks; tool blocked |
| MH-02 | Enabled plugin + .mcp.json | Tools visible (or explicit merge error) |
| MH-03 | Disable plugin | Neither present before next event |
| MH-04 | Invalid JSON | Session alive; skip logged |
| MH-05 | Face counts | Match registry (D3 A) |
| MH-06 | Duplicate MCP server name | User/project wins over plugin |
| MH-07 | Re-enable twice | No duplicate handlers |
| MH-08 | Untrusted project plugin | No spawn (D1) |

Unit: parser fixtures. Integration: temp plugin dir + list/dispatch.

---

## 10. Files to touch (when approved)

| Area | Paths |
|------|-------|
| Discovery / gates | `src/cli/face_plugins.rs` |
| Hook compile | new module under `crates/next-code-hooks/` or `src/cli/plugin_hooks_compile.rs` |
| Registry merge | `crates/next-code-hooks/src/registry.rs`, config load used by app-core |
| MCP merge | `crates/next-code-base/src/mcp/protocol.rs`, `manager.rs` |
| Face honesty A | `crates/xai-grok-pager/src/views/extensions_modal.rs` → `build_plugin_fields` |
| Types | `crates/xai-hooks-plugins-types/src/lib.rs` |
| Docs | `docs/plugins.md`, `docs/HOOKS.md` |

---

## 11. Exit criteria

- [ ] Enabled plugin with `hooks/hooks.json` → `PreToolUse` fires via `next-code-hooks` (stdin/stdout protocol unchanged).
- [ ] Enabled plugin with `.mcp.json` → servers/tools visible to model (or explicit merge error logged — not silent UI Active).
- [ ] Disabled plugin → neither hooks nor MCP contributions present.
- [ ] Face counts match honesty Decision A after merge.
- [ ] Precedence user > project > plugin documented and tested.
- [ ] Provenance on every compiled entry.
- [ ] No second dispatcher process.

---

## 12. If Phase 1 only ships D3 Option B

Keep this file as the **A / Phase 1.5** spec. Demote UI in D3; do not half-merge without enable+trust. Track merge as explicit follow-up referencing this deepen + D7/D1/D2.

---

## 13. Non-goals / out of scope

- OpenCode in-process `Hooks` TypeScript interface / QuickJS revival.
- Auto-writing plugin JSON into user `hooks.toml` as the only mechanism.
- Face UI for editing `hooks.json` inside plugins.
- Marketplace install trust UX beyond D1.
- Phase 2 `next-code-plugin.toml` body (D8 consumes this backend).
- Auto-exporting all MCP tools without alias policy (D11 Phase 3).

---

## 14. Open questions (≤2 — non-blocking)

1. Persist compiled handlers only in memory, or also write a generated overlay file under `~/.next-code/` for debuggability?
2. For MCP multi-plugin name clashes: last-wins (proposed) vs first-wins vs hard error?

Defaults: **in-memory + structured logs**; **last-wins + log**.

---

## Status

**Design contract.** Pair with D3 for Phase 1 honesty. Waiting master/readiness **go ahead** before production Rust.
