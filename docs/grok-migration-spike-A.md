# Spike A: AppState & Event Loop Architecture

## Date: 2026-07-17
## Status: ✅ Complete (pre-code research)

---

## 1. Grok Architecture — Pager as Thin ACP Client

```
grok CLI
  ├── serve         → ACP leader daemon (agent runtime, providers, tools)
  ├── pager/serve   → TUI client (thin, communicates via ACP over Unix socket)
  └── agent         → headless mode (also ACP client)
```

**Pager process:**
- Starts, connects to an ACP leader via Unix socket
- `xai_acp_lib` provides channels: `AcpAgentTx` (send to agent), `AcpAgentRx` (receive from agent)
- **No agent/providers/tools in-process** — pure UI rendering + input routing
- Event loop: `tokio::select!` over: input (crossterm), ACP messages, animation ticks, task results
- State: `AppView` (10,348 LOC) — owns everything: agents, sessions, scrollback, input, etc.
- ACP messages: `agent_client_protocol` crate (external, from crates.io)
- Pager IS the `acp::ClientSide` of the connection

**Key files:**
| File | LOC | Purpose |
|------|:---:|---------|
| `app/mod.rs` | 1,793 | Terminal management, screen mode, static flags |
| `app/app_view.rs` | 10,348 | Root state: agents, sessions, scrollback, input, layout |
| `app/event_loop.rs` | 4,118 | tokio::select! loop, suspend_for_child, restore |
| `app/dispatch.rs` | ~58 files | Action → Effect mapping (sync state mutations) |
| `app/effects.rs` | ~5 files | Effect → async tasks (ACP sends, timers, I/O) |
| `app/agent_view/` | ~10 files | Per-agent view-model: input state, render, paste |

**State structure:**
```
AppView
  ├── agents: IndexMap<AgentId, AgentView>
  │     └── AgentView
  │           ├── session: SessionModel (cwd, id, yolo, auto, ...)
  │           ├── scrollback: ScrollbackState (49k LOC)
  │           ├── input_state
  │           ├── queue: pending prompts
  │           └── modals (agents_modal, settings_modal, ...)
  ├── active_view: ActiveView (welcome | agent(id) | ...)
  ├── screen_mode: ScreenMode (fullscreen | minimal)
  ├── model_state: ModelState (provider/model info from ACP)
  └── agent_tx: AcpAgentTx  ← channel to ACP leader
```

---

## 2. Next-Code Architecture — Monolithic Server

```
next-code CLI
  ├── serve          → starts Unix socket server
  ├── (TUI mode)     → built-in TUI, connects to local server
  └── various cmds   → agent, session, memory, ...
```

**Key difference:** next-code is NOT thin-client. The `server/` module (2,222 LOC in `server.rs` alone, many sub-modules) runs everything:
- `client_lifecycle.rs` (3,128 LOC) — handles Request → agent runtime
- `client_session.rs` — session management
- `client_state.rs` — state queries
- `server/agent` — in-process agent runtime
- `server/provider` — provider management
- `server/tool` — tool registry/execution

**Server-side protocol:** `next-code-protocol` defines `Request`/`ServerEvent` enums — NOT the same as ACP.

**State structure:** Not in a single `AppView`. Instead:
- `ServerRuntime` manages agent sessions
- Clients connect via tokio socket, send `Request`, receive `ServerEvent`
- TUI mode is one such client (but in-process)

---

## 3. Architecture Gap — The Core Challenge

| Dimension | Grok Pager | Next-Code TUI |
|-----------|-----------|---------------|
| **Architecture** | Thin ACP client | Monolithic (server + TUI in process) |
| **Protocol** | ACP (`agent_client_protocol`) | `next-code-protocol` (`Request`/`ServerEvent`) |
| **Agent runtime** | Separate ACP leader process | In-process `agent::Agent` + `server/` |
| **TUI state** | `AppView` (single struct, 10k LOC) | Distributed |
| **Transport** | Unix socket to leader | Unix socket OR in-process channel |
| **Screen suspend** | `suspend_for_child()` (reader park, writer drain, alt screen) | ? (need to check) |

**✅ Compatible:**
- Both use **crossterm** + **ratatui** for TUI
- Both use **tokio** async runtime
- Both have similar terminal lifecycle (raw mode, alt screen)

**⚠️ Needs adapter:**
- ACP messages must map to next-code `Request`/`ServerEvent`
- `AppView` sends user input via `AcpAgentTx` → needs to call next-code agent instead
- Pager receives session events via ACP → needs translation from next-code server events
- `AppView` expects ACP leader capabilities (initialize, reconnect, foreign sessions, etc.)

**🔴 No equivalent / skip:**
- xAI auth (OAuth via auth.x.ai) → use next-code's own auth
- Grok telemetry → stub
- ACP leader cluster / foreign sessions → stub
- xAI-specific permissions (plan approval, SuperGrok upsell) → stub
- Voice (xAI-specific) → skip

---

## 4. Adapter Strategy — In-Process ACP Shim

### Approach: Wrap next-code agent runtime as ACP Agent

```
┌─────────────────────────────────────────────┐
│ next-code process                             │
│                                                │
│  ┌──────────────┐     ┌──────────────────┐   │
│  │ Grok Pager   │ ←→ │ ACP Shim         │   │
│  │ AppView      │     │ (local channels) │   │
│  │ event_loop   │     │                  │   │
│  │ scrollback   │     │ AcpAgentTx/Rx   │   │
│  │ input, views │     │                  │   │
│  └──────────────┘     └───────┬──────────┘   │
│                               │               │
│                        ┌──────▼──────────┐   │
│                        │ next-code server │   │
│                        │ (agent runtime)  │   │
│                        │ providers/tools  │   │
│                        │ sessions/memory  │   │
│                        └─────────────────┘   │
└─────────────────────────────────────────────┘
```

### Shim interface (1 trait):

```rust
// xai-shim-acp provides this trait
// next-code-app-core implements it
pub trait GrokHost {
    // Session lifecycle
    fn initialize(&mut self) -> Result<()>;
    fn send_message(&mut self, text: &str, images: &[ImageData]) -> Result<()>;
    fn subscribe(&mut self, cwd: &Path) -> Result<()>;
    
    // Tool execution
    fn execute_tool(&mut self, name: &str, args: Value) -> Result<ToolResult>;
    fn read_file(&mut self, path: &Path) -> Result<String>;
    fn write_file(&mut self, path: &Path, content: &str) -> Result<()>;
    
    // State queries
    fn get_history(&self) -> Vec<Turn>;
    fn get_model_catalog(&self) -> Vec<ModelInfo>;
    fn get_token_usage(&self) -> TokenUsage;
    
    // Events (streamed from agent to pager)
    fn poll_events(&mut self) -> Vec<ServerEvent>;
}
```

### Messages to translate:

| ACP Method (what pager calls) | next-code equivalent |
|---|---|
| `session/initialize` | Start agent session with model/provider |
| `session/message` | `agent::handle_input()` |
| `session/subscribe` | Subscribe to session events |
| `session/restore` | `session/load` / `reload` |
| `session/load` | `server/client_session::handle_resume_session` |
| `fs/read_text_file` | Tool execution (read tool) |
| `fs/write_text_file` | Tool execution (write tool) |
| `terminal/create` | Tool execution (terminal) |
| `terminal/output` | Poll terminal output |
| `terminal/wait_for_exit` | Wait + stream |
| `terminal/kill` | Kill terminal process |
| `request/permission` | Permission dialog UI → tool approval |
| `session/update` → notification | ServerEvent::Session / ToolActivity |

---

## 5. Event Loop Differences

### Grok event loop (`event_loop.rs`, 4,118 LOC)
```rust
tokio::select! {
    // Biased: input first, then ACP, then ticks
    Some(event) = event::read() => handle_input(event),  // crossterm input
    Some(msg) = acp_rx.recv() => handle_acp_message(msg), // agent response  
    _ = animation_tick() => draw(),
    Some(task) = task_set.join_next() => handle_task_result(task),
    _ = config_watcher.changed() => reload_config(),
    // suspend_for_child (editor/pager spawn)
}
```

### Next-code event loop (implicit, in `server/`)
- Accepts client connections on Unix socket
- Reads `Request` from each client
- Processes via `client_lifecycle::handle_agent_task`, etc.
- Streams `ServerEvent` back to client

### Adapter approach:
1. Replace `acp_rx.recv()` with next-code agent runtime response
2. Instead of ACP messages, call next-code's Sender/Receiver pair
3. Keep Grok's input handling, animation ticks, suspend logic as-is
4. Add a `select!` arm for next-code server events

---

## 6. Summary — Feasibility

| Component | Strategy | Difficulty |
|-----------|----------|:----------:|
| `app/event_loop.rs` | Keep as-is, replace ACP channel with shim | 🟡 |
| `app/app_view.rs` | Keep as-is (10k LOC, zero edit) | ✅ |
| `app/dispatch/` | Modify: replace ACP `dispatch` with next-code calls | 🔴 |
| `app/effects/` | Replace ACP sends with shim calls | 🟡 |
| `app/agent/` | Keep AgentId/AgentView types, remove ACP model_state | 🟡 |
| `app/agent_view/` | No changes needed (pure UI) | ✅ |
| `input/` | No changes needed (crossterm + key parsing) | ✅ |
| `scrollback/` | No changes needed (pure UI state) | ✅ |
| `views/` | Permission dialog: needs next-code approval bridge | 🟡 |
| `settings/` | Replace config xAI → next-code config types | 🟡 |
| `acp/` module | Replace entirely with shim | 🟡 |
| `headless.rs` | No change needed (not used in TUI mode) | ✅ |
| `notifications/` | Proxy shell → next-code notification | 🟡 |
| `slash/` commands | Most are xAI-specific → skip | 🟡 |

**Key insight: The pager code itself (app_view, scrollback, input, views, settings) does NOT need to change.** Only the ACP transport and config need shimming. The pager's `AppView` just draws + routes input — everything goes through `AcpAgentTx`.
