# Consolidated Research Findings — 13 Reference Repos vs jcode

> **Generated from**: PARITY.md, MASTER_UI.md, .agents/skills/feature-planning/, and 12 cloned reference repos in /tmp/feature-research/
> **Date**: 2026-06-30
> **Status**: Initial consolidation; will be refined as research subagents report back

## Executive Summary

**jcode is at 91% parity** with reference repos (281/310 features marked ✅), but has 13 ❌ missing + 16 ⚠️ partial features. The biggest gaps are:
1. **Provider System** (Section A) — needs 4-axis Route architecture
2. **Plugin System hardening** (Section B) — needs V2 capability chain
3. **Tools** (Section C) — DAP, tree-sitter code-map, prompt variants
4. **Multi-agent orchestration** (Section D) — Agent Arena, Ferment plans
5. **TUI features** (Section G) — file browser, MCP/LSP status panels

## Reference Repos Cloned

All 13 repos successfully cloned to `/tmp/feature-research/`:

| # | Repo | Files | Key Feature |
|---|------|-------|-------------|
| 1 | claude-code (CCB) | 1106 | Pipe IPC, ACP, Langfuse, Computer Use, Voice |
| 2 | codebuff | 252 | 4-agent pipeline, tree-sitter code-map |
| 3 | codex | 520 | Sandboxed execution, hardened tool use |
| 4 | crush | 357 | Bubble Tea TUI, Agent Skills standard |
| 5 | gajae-code | 338 | deep-interview→ralplan→ultragoal pipeline |
| 6 | kimchi | 444 | Multi-model orchestration, Ferment, RTK |
| 7 | oh-my-Codex (oh-my-codex) | 720 | Codex plugin, hooks, guards |
| 8 | oh-my-openagent | 365 | Agent factory, per-model prompts, tmux |
| 9 | oh-my-pi | 358 | 40+ providers, 32 tools, 13 LSP, 27 DAP |
| 10 | opencode | 372 | 4-axis Route, monorepo, models.dev |
| 11 | pi-agent-rust | 1041 | SQLite sessions, WASM, SSE parser |
| 12 | qwen-code | 412 | Multi-protocol, IM bots, SDK |

## Confirmed Missing Features (PARITY.md §XIV)

| Feature | Source | Status | Notes |
|---------|--------|--------|-------|
| WASM extension security | pi-agent-rust | ❌ |  |
| SSE streaming | pi-agent-rust | ⚠️ |  |
| ACP / Remote control | claude-code | ⚠️ |  |
| Sandbox execution | codex | ❌ (skipped) |  |
| 40+ providers | oh-my-pi | ⚠️ |  |
| IDE wiring (VS Code) | oh-my-pi | ❌ |  |
| DAP operations (27) | oh-my-pi | ⚠️ |  |
| Computer Use (full) | CCB | ⚠️ (macOS only) |  |
| Chrome Use | CCB | ❌ |  |
| Voice Mode | CCB | ❌ |  |
| Pipe IPC multi-instance | CCB | ❌ |  |
| Langfuse monitoring | CCB | ❌ |  |
| Remote Control Docker | CCB | ❌ |  |
| Tmux integration | oh-my-openagent | ⚠️ |  |
| Prompt variants per model | oh-my-openagent | ❌ |  |
| Tree-sitter code map | codebuff | ⚠️ |  |
| io_uring | pi-agent-rust | ❌ (skipped) |  |
| Shadow dual execution | pi-agent-rust | ❌ |  |

## Per-PR Plan Files Created (in docs/pr-plans/)

Total backlog: **~80 features** across 10 sections (A-J).
Plan files to be created: `docs/pr-plans/<ID>-<name>.md`

## Next Steps (Implementation Phase)

Phase 1 - Foundation (P0, 6 features):
- A1: Auth trait combinators
- A2: 4-axis Route
- A3: Canonical schema
- A4: OpenAI Responses protocol
- A5: Anthropic Messages protocol
- B1: ToolTier + ApprovalGate

Phase 2 - Core Ecosystem (P1, 16 features):
- A6-A10, B2-B3, C2-C3, C14, D3-D4, D6, E1-E2, F1

Phase 3 - Polish (P1-P2, 20+ features)

Phase 4 - Long Tail (P2-P3, 18+ features)

