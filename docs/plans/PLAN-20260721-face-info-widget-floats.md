# Plan Report — Face floating info widgets (full remaining set)

## Summary (read this first)
- **You asked:** Full remaining legacy info-widget float set on Face (copy → wire), scroll-gated.
- **What shipped:** Paste-copied renderers + Left/Right stacked placement; data wired where Face/ACP can supply it.
- **Risk:** Low–Medium (Memory/Git depend on pager_agent ACP emits; WorkspaceMap/Diagrams still deferred)
- **Status:** Implemented on `pr-face-info-widget-floats` — rebuild/install + restart serve

## WidgetKind → Face status

| WidgetKind | Face status | Data source |
|---|---|---|
| Overview | **wired** | model/provider/sessions + TokenUsage context |
| KvCache | **wired** | TokenUsage cache fields |
| MemoryActivity | **wired** | `next-code/memory_info` (pager_agent MemoryManager + MemoryActivity) |
| UsageLimits | **wired** | Face `credit_balance` → Credits bars (OAuth 5h/weekly not on Face path) |
| GitStatus | **wired** | `next-code/git_status` (porcelain gather in pager_agent) |
| BackgroundTasks | **wired** | `session.bg_tasks` Running |
| Compaction | **wired** | AutoCompact* → `info_float_compaction` |
| SwarmStatus | **wired** | `subagent_sessions` as managed_members |
| Todos | **wired** | TodoPane items (float parity with pane) |
| WorkspaceMap | **deferred** | no Face `workspace_client`; stub renderer + empty gate |
| Diagrams | **deferred** | mermaid image pipeline not on Face floats; stub + empty gate |
| AmbientMode | **skipped** | hard-disabled (confirmed) |
| Tips | **skipped** | hard-disabled (confirmed) |
| TeamView | **skipped** | `has_data_for` always false (confirmed) |

## Visibility
All floats: show **only while scrolling**; hide after **1000ms** idle (`SCROLL_IDLE_HIDE_MS`).

## Overview merge
Face keeps **separate floats** by `preferred_side` (no Overview merge) so the full set appears as distinct boxes while scrolling.

## Copy map
| Module | Source |
|---|---|
| `views/info_floats/mod.rs` | Overview/KV + layout stack |
| `views/info_floats/widgets.rs` | Memory / Usage / Git / Background / Compaction / Swarm / Todos / stubs |

## Wire map
| Seam | What |
|---|---|
| `pager_agent.rs` | emit `next-code/memory_info` + `next-code/git_status` on session create/attach; refresh memory on `ServerEvent::MemoryActivity` |
| `acp_handler/mod.rs` | fold memory_info / git_status into `AgentView` |
| `session_notification.rs` | AutoCompact* → compaction float card |
| `agent_view/session.rs` | `build_info_float_data` assembles all float fields |

## Checklist
- [x] MemoryActivity (Right)
- [x] UsageLimits (Left) — credits mapping
- [x] GitStatus (Left)
- [x] BackgroundTasks (Left)
- [x] Compaction (Left)
- [x] SwarmStatus (Left) when subagents exist
- [x] Todos (Right)
- [x] WorkspaceMap stub + empty gate (deferred data)
- [x] Diagrams stub + empty gate (deferred image pipeline)
- [x] Unit tests for formatters / has_data gates
- [x] `cargo test -p xai-grok-pager --lib views::info_floats` (14 passed)
- [x] `cargo check -p xai-grok-pager --lib` + `cargo check -p next-code --bin next-code`
- [ ] Rebuild/install Face binary + restart serve (owner smoke)

## Smoke
1. Rebuild + install; **restart** `next-code serve`.
2. Scroll during a session with todos / bg tasks / subagents / dirty git / memories.
3. Expect stacked floats Left (KV, Usage, Compaction, Background, Git, Swarm) and Right (Memory, Overview, Todos) while scrolling.
4. Idle ~1s → all floats hide.
5. WorkspaceMap / Diagrams stay hidden until data pipelines exist.
