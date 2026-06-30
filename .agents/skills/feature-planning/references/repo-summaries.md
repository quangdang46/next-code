# Reference Repo Summaries

Static summaries of all 13 repos for quick lookup without cloning.

---

## 1. oh-my-openagent
**URL:** https://github.com/code-yeongyu/oh-my-openagent  
**Stack:** TypeScript, Bun, OpenCode plugin  
**What it is:** A powerful OpenCode plugin that adds named agents (Prometheus, Atlas, Hephaestus, Sisyphus-Junior) with model-variant routing, tmux session management, and multi-agent delegation via `delegate-task`.

**Key Patterns:**
- **Agent factory pattern**: `createAtlasAgent()`, `createHephaestusAgent()` — each agent is a factory with model-variant routing
- **Prompt variants per model**: `default.md` (Claude), `gpt.md`, `gemini.md`, `kimi.md` — same agent, different prompt per provider
- **Delegate-task orchestration**: Atlas spawns Sisyphus-Junior subagents via `task()` calls; never self-reports, always verifies
- **Model resolution pipeline**: `resolveModel(input)` → UI override → agent-specific → fallback chain
- **Tmux integration**: `createTmuxSession()`, `spawnPane()` for multi-pane agent workflows
- **Session management**: `SessionCursor`, `trackInjectedPath()`, `SessionToolsStore`
- **Config migration**: `migrateConfigFile()` with `AGENT_NAME_MAP`, `HOOK_NAME_MAP`, `MODEL_VERSION_MAP`

**Key Files:**
- `src/agents/atlas/agent.ts` — orchestrator agent factory
- `src/agents/prometheus/system-prompt.ts` — strategic planner prompt loader
- `src/agents/hephaestus/agent.ts` — autonomous deep worker
- `src/agents/sisyphus-junior/agent.ts` — category-spawned executor
- `src/shared/index.ts` — barrel export of 297 utility files
- `src/shared/model-availability.ts` — `resolveModel()`, `checkModelAvailability()`
- `packages/prompts-core/` — model-neutral prompt markdown files

---

## 2. opencode
**URL:** https://github.com/anomalyco/opencode  
**Stack:** TypeScript, Bun, SST, monorepo (Turbo)  
**What it is:** The open source AI coding agent. Terminal UI with provider abstraction, extension system, desktop app, and a well-structured monorepo.

**Key Patterns:**
- **Provider abstraction**: Clean separation between LLM provider and agent logic
- **Monorepo layout**: `packages/` with `tui/`, `desktop/`, `web/`, `identity/`
- **SST for infra**: Config-as-code for cloud deployment
- **Desktop + TUI**: Supports both Electron-style desktop and pure terminal modes
- **Zed extension**: `packages/extensions/zed/` for IDE integration

**Key Files:**
- `packages/tui/` — terminal UI implementation
- `packages/desktop/` — Electron-style desktop wrapper
- `sst.config.ts` — infrastructure config
- `turbo.json` — monorepo build pipeline

---

## 3. oh-my-pi
**URL:** https://github.com/can1357/oh-my-pi  
**Stack:** TypeScript + Rust, Bun ≥ 1.3.14  
**What it is:** Fork of Pi by @mariozechner. "A coding agent with the IDE wired in." 40+ providers, 32 built-in tools, 13 LSP ops, 27 DAP ops, ~27k lines of Rust core.

**Key Patterns:**
- **Benchmarked tool use**: Every tool is tuned for first-attempt success rate; `packages/typescript-edit-benchmark/` has full benchmark harness
- **LSP integration**: 13 language server protocol operations built in
- **DAP integration**: 27 debug adapter protocol operations built in
- **Multi-provider**: 40+ providers with provider-neutral abstraction
- **Rust core + TS surface**: Performance-critical code in Rust, developer-facing API in TypeScript
- **Mutation testing**: `src/mutations.ts` for benchmark task generation

**Key Files:**
- `packages/typescript-edit-benchmark/src/` — full benchmark framework
- `packages/typescript-edit-benchmark/src/tasks.ts` — benchmark task definitions
- `packages/typescript-edit-benchmark/src/runner.ts` — benchmark runner
- `packages/typescript-edit-benchmark/src/prompts/` — benchmark prompt templates

---

## 4. codebuff
**URL:** https://github.com/CodebuffAI/codebuff  
**Stack:** TypeScript, multi-agent pipeline  
**What it is:** AI coding assistant that coordinates specialized agents. Beats Claude Code 61% vs 53% on 175+ coding tasks. Has a `freebuff` free tier.

**Key Patterns:**
- **4-agent pipeline**: File Picker → Planner → Editor → Reviewer — each is a specialized agent
- **Tree-sitter code map**: `packages/code-map/` uses tree-sitter for language-aware code parsing across 10+ languages
- **Agent composition**: Multi-agent as a *strategy*, not just concurrency — each agent has a specific role
- **Custom agent builder**: `/init` command generates agent scaffolding
- **Eval-driven development**: `evals/` directory with 175+ tasks across real open-source repos

**Key Files:**
- `packages/code-map/src/index.ts` — code map entry point
- `packages/code-map/src/languages.ts` — language detection
- `packages/code-map/src/tree-sitter-queries/` — per-language AST queries
- `evals/README.md` — eval methodology

---

## 5. codex (OpenAI Codex CLI)
**URL:** https://github.com/openai/codex  
**Stack:** TypeScript, Node.js  
**What it is:** OpenAI's official local coding agent CLI. Single binary, sandboxed execution, ChatGPT plan integration.

**Key Patterns:**
- **Sandbox-first execution**: All tool use is sandboxed; firewall init script at `scripts/init_firewall.sh`
- **Container execution**: `run_in_container.sh` for isolated runs
- **Hardened tool use**: Security-first design, execution policy, network isolation
- **Multiple install paths**: npm, Homebrew, binary releases — portable distribution
- **Bazel build**: `BUILD.bazel`, `MODULE.bazel` for reproducible builds

**Key Files:**
- `codex-cli/bin/codex.js` — CLI entry point
- `codex-cli/scripts/init_firewall.sh` — firewall/sandbox setup
- `codex-cli/scripts/run_in_container.sh` — container execution
- `codex-cli/package.json` — deps and scripts

---

## 6. claude-code (CCB — Claude Code Best)
**URL:** https://github.com/claude-code-best/claude-code  
**Stack:** TypeScript, Bun  
**What it is:** Decompiled/reconstructed Claude Code (CCB = 踩踩背) with many enterprise features: Pipe IPC multi-instance, ACP protocol (Zed/Cursor IDE), Remote Control Docker deployment, Langfuse monitoring, Web Search, Computer Use, Chrome Use, Voice Mode, Sentry, GrowthBook.

**Key Patterns:**
- **Pipe IPC**: `main/sub` auto-orchestration + LAN cross-machine zero-config discovery; `/pipes` panel + `Shift+↓` + message broadcast routing
- **ACP Protocol**: Session resume, Skills, permission bridging for Zed/Cursor
- **Remote Control**: Docker self-hosted remote UI — watch Claude Code from your phone
- **Langfuse monitoring**: Every agent loop step is observable and can be converted to datasets
- **Feature flags**: GrowthBook integration for enterprise feature gating
- **Memory management**: `/dream` command for memory consolidation
- **Poor Mode**: Disable memory extraction + typing suggestions to reduce concurrent requests

**Key Files:**
- `src/types/message.ts` — message types
- `src/types/tools.ts` — tool type definitions
- `src/types/plugin.ts` — plugin system types
- `src/types/hooks.ts` — hook system

---

## 7. pi-agent-rust
**URL:** https://github.com/Dicklesworthstone/pi_agent_rust  
**Stack:** Rust 2024 edition, `asupersync` async runtime, `rich_rust` TUI  
**What it is:** High-performance Rust port of Pi Agent by Jeffrey Emanuel. Single binary, <100ms startup, <50MB idle memory, SQLite sessions, WASM extension security, io_uring fast lane.

**Key Patterns:**
- **SQLite session store**: `src/session_sqlite.rs` — segmented log + offset index, O(index+tail) reopen on large histories
- **Hostcall security model**: Capability-gated hostcalls: `tool`/`exec`/`http`/`session`/`ui`/`events`; two-stage exec guard; trust lifecycle `pending→acknowledged→trusted→killed`
- **io_uring fast lane**: `src/hostcall_io_uring_lane.rs` — deterministic dispatch, typed opcodes, bounded shard queues
- **WASM extension runtime**: `src/pi_wasm.rs` — startup prewarm, warm isolate reuse, DCG/heredoc AST signals for dangerous shell detection
- **SSE streaming parser**: Tracks scanned bytes, handles UTF-8 tails, normalizes chunk boundaries, interns event-type strings
- **Multi-provider**: `src/providers/` — Anthropic, OpenAI, Vertex, Azure, Cohere, GitLab, Copilot
- **Benchmarks**: `benches/` — tools, semantic context, session save, TUI perf, extensions
- **Shadow dual execution**: Automatic backoff on divergence; compatibility-lane kill switches

**Key Files:**
- `src/session_sqlite.rs` — session persistence
- `src/agent_cx.rs` — agent execution context
- `src/extension_dispatcher.rs` — WASM extension dispatch
- `src/hostcall_io_uring_lane.rs` — fast-path hostcall routing
- `src/providers/anthropic.rs` — Anthropic provider
- `src/pi_wasm.rs` — WASM runtime
- `benches/` — full benchmark suite
- `Cargo.toml` — deps: `asupersync`, `rich_rust`, edition 2024

---

## 8. gajae-code
**URL:** https://github.com/Yeachan-Heo/gajae-code  
**Stack:** TypeScript (86.2%) + Rust (7.8%) / Bun  
**What it is:** An external coding-agent harness (`gjc`) that runs alongside Claude Code or Codex CLI. Structured workflow pipeline: `deep-interview` → `ralplan` → `ultragoal`, with optional tmux parallel workers. "Encode intention. Decode software."

**Key Patterns:**
- **Workflow pipeline**: Three-step `deep-interview` → `ralplan` → `ultragoal` with stateful progression
- **Tmux-native sessions**: `--tmux` flag for tmux-backed leader sessions with isolated worktrees (`--worktree`)
- **Notifications SDK**: Bundled Telegram reference daemon with WebSocket-based `action_needed`/`reply` protocol for mobile answers
- **Research/REPL mode**: Optional Jupyter-notebook-style `rlm` with persistent Python kernel
- **Computer-Use mode**: Experimental desktop control with screenshot/input bindings
- **TUI identity**: Dark (`red-claw`) and light (`blue-crab`) themes, plus migration themes mimicking Claude Code, Codex CLI, OpenCode
- **External & non-invasive**: Sits beside existing agents, not as a plugin; supports RPC, MCP, and Bridge/HTTPS surfaces
- **Configuration**: Provider retry budgets in `~/.gjc/config.yml`

**Key Files:**
- `gjc.ts` — CLI entry point
- `src/pipeline/` — workflow pipeline stages
- `src/tmux/` — tmux session management
- `src/notifications/` — Telegram notification SDK
- `src/repl/` — REPL/research mode

---

## 9. kimchi
**URL:** https://github.com/getkimchi/kimchi  
**Stack:** TypeScript (90.9%) / Node.js 22, Bun, pnpm  
**What it is:** Terminal-based AI coding agent that acts as a development assistant inside the CLI, backed by Kimchi's LLM infrastructure. Built on the [pi-mono](https://github.com/badlogic/pi-mono) coding agent SDK.

**Key Patterns:**
- **Multi-model orchestration**: Different LLMs handle different roles (orchestrator, builder, reviewer, explorer, researcher); orchestrator classifies tasks and delegates to role-specific models
- **Ferment system**: Cross-session project management persisting structured plans (goals, phases, steps) as JSON, enabling progressive-refinement work across sessions with a state machine enforcing valid transitions
- **LSP integration**: Built-in Language Server Protocol support for type-aware code intelligence (TypeScript/JavaScript via `typescript-language-server`, Go via `gopls`), auto-synced with file edits
- **Remote teleport**: Session multiplexing — local TUI serves as home base; remote workers can be spawned, detached, and re-attached
- **RTK token optimization**: Rewrites bash tool calls to compress command output by 60-90%
- **Agent discovery & migration**: Detects existing Claude Code, OpenCode, or Cursor installs on first run and offers to migrate their MCP servers
- **Tagging & phase tracking**: Every LLM request labeled with `phase:{name}` and user-defined tags for usage analytics and cost attribution
- **Hooks system**: Custom Bash hooks to rewrite or block shell commands before execution
- **Context files**: AGENTS.md/CLAUDE.md at global and project levels injected into system prompt

**Key Files:**
- `src/commands/` — CLI subcommands directory
- `src/extensions/` — extensions for subagents, orchestration, MCP, auth
- `src/multi-model/` — multi-model orchestration core
- `src/ferment/` — cross-session plan system
- `src/lsp/` — LSP integration
- `src/teleport/` — remote session multiplexing
- `src/rtk/` — RTK token optimization

---

## 10. qwen-code
**URL:** https://github.com/QwenLM/qwen-code  
**Stack:** TypeScript (79.2%) + Rust / Node.js 22+  
**What it is:** Open-source AI coding agent that started as a fork of Google Gemini CLI v0.8.2, now an independent multi-protocol agent framework with 25.7k stars.

**Key Patterns:**
- **Multi-protocol support**: Works with OpenAI, Anthropic, Gemini, and Qwen APIs, plus local models via Ollama/vLLM
- **Auto-Memory & Auto-Skills**: Dynamic workflows with no manual setup — memory, skills, sub-agents, agent teams, and MCP
- **Multiple operation modes**: Interactive TUI, headless (`qwen -p "..."`), daemon mode for shared agent sessions, IDE plugins (VS Code, JetBrains, Zed), desktop GUI
- **IM integrations**: Telegram, DingTalk, WeChat, and Feishu bots
- **First-party SDK**: TypeScript, Python, and Java SDKs for embedding agent capabilities
- **Agent delegation via ACP**: "Qwen Code Claw" pattern — other agents delegate coding tasks to Qwen Code
- **Self-iteration**: Actively iterating on itself — using its own agent and models to file issues, submit PRs, review code, run tests
- **Dual-license**: Apache-2.0

**Key Files:**
- `cli/` — CLI entry point and command handling
- `src/providers/` — multi-provider implementations
- `src/agents/` — agent delegation and team orchestration
- `src/im/` — IM integration handlers (Telegram, DingTalk, etc.)
- `sdk/` — first-party SDKs (TypeScript, Python, Java)
- `src/memory/` — auto-memory system

---

## 11. crush

**URL:** https://github.com/charmbracelet/crush
**Stack:** Go (98.4%) / Charm ecosystem (Bubble Tea TUI, GoReleaser, sqlc)
**What it is:** Open-source, terminal-based AI coding assistant built by Charm. Agentic coding buddy with session-based workspaces, multi-client sharing, LSP + MCP integration, and the Agent Skills open standard.

**Key Patterns:**

- **Bubble Tea TUI**: Full terminal UI built on Charm's Bubble Tea framework with ratatui-style composable widgets
- **Session workspaces**: Multiple sessions and contexts per project; clients sharing the same `--cwd` join the same workspace with live session mirroring
- **Shared multi-client sessions**: Multiple TUI instances or `crush serve` clients join the same workspace, with attached-client and busy signals
- **Agent Skills standard**: Discoverable skill packages from `AGENTS.md`, `.agents/skills/` paths; user-invocable via Ctrl+P
- **LSP-enhanced context**: Language servers for richer code understanding, auto-synced with file edits
- **MCP extensibility**: Add capabilities via MCP servers (HTTP, stdio, SSE) with shell expansion in config values
- **Provider abstraction**: Pluggable backend supporting OpenAI, Anthropic, Bedrock, Vertex AI, Ollama, llama.cpp
- **`crushignore` files**: Extends `.gitignore` to limit what the agent sees as context
- **Auto-provider updates**: Pulls latest model metadata from Catwalk community registry
- **Desktop notifications**: Configurable, sent when focus is lost and agent finishes a turn
- **Git attribution**: `Assisted-by` and `Co-Authored-By` trailers on commits
- **First-wins flag policy**: For shared workspaces, flags like `--yolo` and `--debug` are set by the first client

**Key Files:**

- `main.go` — CLI entry point
- `tui/` — Bubble Tea TUI implementation
- `session/` — session workspace management
- `provider/` — provider abstraction layer
- `lsp/` — LSP integration
- `mcp/` — MCP server client
- `skills/` — Agent Skills system
- `config/` — configuration layering (local `.crush.json` overrides global)
- `crushignore/` — ignore file parser

---

## Cross-Repo Quick Reference

| Feature Domain | Best Source Repo(s) |
|----------------|---------------------|
| Multi-agent orchestration | oh-my-openagent (Atlas/delegate-task), codebuff (4-agent pipeline), kimchi (multi-model roles), crush (multi-client shared sessions) |
| Model/provider abstraction | oh-my-openagent (resolveModel), oh-my-pi (40+ providers), pi-agent-rust (src/providers/), qwen-code (multi-protocol), crush (plugable backend) |
| Session persistence | pi-agent-rust (SQLite), claude-code (memory/dream), kimchi (Ferment cross-session plans), crush (session workspaces) |
| Security & sandboxing | codex (firewall), pi-agent-rust (capability gates, trust lifecycle), crush (--yolo, permission system) |
| Benchmarking | oh-my-pi (typescript-edit-benchmark), pi-agent-rust (benches/) |
| IDE integration | oh-my-pi (LSP/DAP), claude-code (ACP/Zed/Cursor), kimchi (LSP integration), crush (LSP-enhanced context) |
| Streaming | pi-agent-rust (SSE parser), opencode (provider abstraction) |
| Extension/plugin system | pi-agent-rust (WASM), oh-my-openagent (OpenCode plugin), claude-code (ACP), crush (Agent Skills, MCP) |
| Monitoring/observability | claude-code (Langfuse, Sentry), pi-agent-rust (runtime risk ledger) |
| TUI design | opencode (terminal UI), pi-agent-rust (rich_rust), oh-my-pi (IDE-wired), gajae-code (dual themes), crush (Bubble Tea) |
| Code understanding | codebuff (tree-sitter code map), oh-my-openagent (ripgrep-cli), crush (LSP-enhanced) |
| Prompt engineering | oh-my-openagent (per-model prompt variants), oh-my-pi (benchmark prompts) |
| Workflow planning | gajae-code (deep-interview → ralplan → ultragoal), kimchi (Ferment), qwen-code (auto-skills) |
| Notifications / Mobile | gajae-code (Telegram notification SDK), qwen-code (Telegram/DingTalk/WeChat bots), crush (desktop notifications) |
| Cross-instance / Teleport | kimchi (remote teleport), claude-code (Pipe IPC), qwen-code (daemon mode), crush (shared multi-client sessions) |
| Token optimization | kimchi (RTK rewrite) |
| Go ecosystem reference | crush (Bubble Tea, Go TUI patterns, GoReleaser) |