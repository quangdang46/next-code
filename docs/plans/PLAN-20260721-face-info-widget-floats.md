# Plan Report — Face floating info widgets (full remaining set)

## Summary (read this first)
- **You asked:** Full remaining legacy info-widget float set on Face (copy → wire), scroll-gated.
- **What shipped:** Paste-copied renderers + Right-stacked placement (non-centered legacy); data wired where Face/ACP can supply it.
- **Risk:** Low–Medium (Memory/Git depend on pager_agent ACP emits; WorkspaceMap/Diagrams text-ready but data empty; Ambient/Tips/TeamView commented stubs only)
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
| WorkspaceMap | **text interim + commented buffer paint** | no Face `workspace_client`; `legacy_deferred` holds copied `render_workspace_map` registration under `TODO(face-floats)`; `build_info_float_data` leaves `None` with citation TODOs |
| Diagrams | **text interim + commented image paint** | mermaid image float not registered; `legacy_deferred` holds copied `render_diagrams_widget` under `TODO(face-floats)`; data `None` until pipeline |
| AmbientMode | **commented stub only** | legacy hard-disabled (`widget_disabled` + `has_data_for => false`) — copy in `legacy_deferred.rs` |
| Tips | **commented stub only** | same hard-disable — copy in `legacy_deferred.rs` |
| TeamView | **commented stub only** | `has_data_for(TeamView) => false` (dead) — copy in `legacy_deferred.rs` |

## Visibility
All floats: show **only while scrolling**; hide after **1000ms** idle (`SCROLL_IDLE_HIDE_MS`).

## Overview merge
Face keeps **separate floats** by `preferred_side` (no Overview merge) so the full set appears as distinct boxes while scrolling. **Placement:** non-centered agent view → all docks **Right** (legacy Left only when `margins.centered`).

## Copy map
| Module | Source |
|---|---|
| `views/info_floats/mod.rs` | Overview/KV + layout stack |
| `views/info_floats/widgets.rs` | Memory / Usage / Git / Background / Compaction / Swarm / Todos |
| `views/info_floats/legacy_deferred.rs` | WorkspaceMap / Diagrams text + commented paint; Ambient / Tips / TeamView commented copies |

## Wire map
| Seam | What |
|---|---|
| `pager_agent.rs` | emit `next-code/memory_info` + `next-code/git_status` on session create/attach; refresh memory on `ServerEvent::MemoryActivity` |
| `acp_handler/mod.rs` | fold memory_info / git_status into `AgentView` |
| `session_notification.rs` | AutoCompact* → compaction float card |
| `agent_view/session.rs` | `build_info_float_data` assembles all float fields |

## Checklist
- [x] MemoryActivity (Right)
- [x] UsageLimits (Left preferred / Right dock non-centered) — credits mapping
- [x] GitStatus (Left preferred / Right dock)
- [x] BackgroundTasks (Left preferred / Right dock)
- [x] Compaction (Left preferred / Right dock)
- [x] SwarmStatus (Left preferred / Right dock) when subagents exist
- [x] Todos (Right)
- [x] WorkspaceMap text interim + empty gate + commented buffer paint / data-fetch TODOs
- [x] Diagrams text interim + empty gate + commented image paint / data-fetch TODOs
- [x] AmbientMode / Tips / TeamView commented copy stubs with legacy disable citations
- [x] Unit tests for formatters / has_data gates
- [x] `cargo test -p xai-grok-pager --lib views::info_floats` (19 passed)
- [x] `cargo check -p xai-grok-pager --lib` + `cargo check -p next-code --bin next-code`
- [x] Rebuild/install Face binary (selfdev @ `f1c60a408`) � **restart serve** still required

## Smoke
1. Rebuild + install; **restart** `next-code serve`.
2. Scroll during a session with todos / bg tasks / subagents / dirty git / memories.
3. Expect stacked floats on the **Right** (KV, Usage, Compaction, Background, Git, Swarm, Memory, Overview, Todos) while scrolling.
4. Idle ~1s → all floats hide.
5. WorkspaceMap / Diagrams stay hidden until data pipelines exist; Ambient / Tips / TeamView remain commented-only.
