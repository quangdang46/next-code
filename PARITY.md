# Subagent UI/UX Parity with Claude Code

This document tracks jcode's subagent UI/UX feature parity with Claude Code.
Each row maps a Claude Code feature to its jcode equivalent, with implementation status and remaining work.

---

## Legend

| Column | Description |
|--------|-------------|
| Name | Feature name (Claude Code terminology) |
| Features | What the feature does |
| References | Claude Code source reference (CCB repo) |
| jcode Implementation | Where the feature lives in jcode |
| Progress | Implementation status |
| Remaining | What still needs to be done |

---

## 1. Running Items List

| Field | Value |
|-------|-------|
| **Name** | Running items list (below status bar) |
| **Features** | - Interactive list below status bar showing running subagents, shell commands, background tasks<br>- Arrow key navigation (↓/↑)<br>- Enter to view detail<br>- Esc to close |
| **References** | CCB: `src/hooks/useBackgroundAgentTasks.ts`, `src/hooks/useTasksV2.ts` |
| **jcode Implementation** | - `ui_running_items.rs`: `draw_running_items()` renders list with status icons<br>- `ui.rs`: running items zone (chunks[8]) between status bar and donut<br>- `input.rs`, `key_handling.rs`: Ctrl+O toggle, ↑/↓ navigate, Enter detail, Esc close |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 2. Detail Overlay

| Field | Value |
|-------|-------|
| **Name** | Subagent/tool detail overlay |
| **Features** | - Rounded border popup showing live item status<br>- Real-time update (rebuilt every frame)<br>- Shows: status, kind, ID, session ID, elapsed time, detail text<br>- Backspace/Ctrl+C to cancel running item |
| **References** | CCB: `src/components/agents/AgentDetail.tsx` |
| **jcode Implementation** | - `ui_running_items.rs`: `draw_running_item_detail()` builds dynamic content each frame<br>- `input.rs`: Enter toggles `detail_open`, Esc closes |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 3. Enter to Subagent Session

| Field | Value |
|-------|-------|
| **Name** | Attach to subagent session |
| **Features** | - Enter on subagent/swarm member in detail view → switches to that agent's session<br>- Uses existing session resume infrastructure<br>- Shows subagent's live transcript |
| **References** | CCB: `src/hooks/useRemoteSession.ts`, session resume via `workspace_client.queue_resume_session()` |
| **jcode Implementation** | - `input.rs`, `key_handling.rs`: Enter in detail overlay → `workspace_client.queue_resume_session(sid)`<br>- UI hint: "Enter to open session · Esc to close" |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 4. Stop/Kill Running Item

| Field | Value |
|-------|-------|
| **Name** | Stop/kill subagent or tool |
| **Features** | - Cancel running subagent, background task, or batch tool<br>- Uses existing cancel/interrupt infrastructure |
| **References** | CCB: `src/tasks/stopTask.ts`, `src/hooks/useCancelRequest.ts` |
| **jcode Implementation** | - `input.rs`, `key_handling.rs`: Ctrl+C or Backspace while detail open → `cancel_requested = true`<br>- Cancel mechanism: `InterruptSignal`, `BackgroundTaskManager::cancel()` |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 5. `/agents` Command — Running Tab

| Field | Value |
|-------|-------|
| **Name** | /agents Running tab |
| **Features** | - Tab 0 (Running): live subagents, background tasks, batch tools, swarm members<br>- Enter to open detail or switch to running items list<br>- Arrow keys navigate within tab |
| **References** | CCB: `src/commands/agents/index.ts`, `src/commands/agents/agents.tsx` |
| **jcode Implementation** | - `openers.rs`: `open_agents_picker()` builds Running tab entries from `build_running_tab_entries()`<br>- `inline_interactive.rs`: Enter dispatches to running items list<br>- Tab/BackTab or Right/Left to switch between Running and Library tabs |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 6. `/agents` Command — Library Tab

| Field | Value |
|-------|-------|
| **Name** | /agents Library tab |
| **Features** | - Tab 1 (Library): agent definitions loaded from disk<br>- `+ Create new agent`: open $EDITOR with TOML template, parse & save<br>- `+ Generate via AI`: open $EDITOR with prompt, queue to current model<br>- Agent files from `~/.jcode/agents/*.toml` and `.jcode/agents/*.toml`<br>- Color badge display (red/blue/green/yellow/purple/orange/pink/cyan)<br>- Enter on agent file → open $EDITOR for editing<br>- Delete agent via `PickerAction::DeleteAgent` |
| **References** | CCB: `src/components/agents/AgentsList.tsx`, `src/components/agents/agentFileUtils.ts`, `src/components/agents/AgentEditor.tsx`, `src/components/agents/generateAgent.ts` |
| **jcode Implementation** | - `openers.rs`: `open_agents_picker()` loads agent definitions via `AgentRegistry`<br>- `run_agent_creation_flow()` in `openers.rs` handles $EDITOR template → parse TOML → save<br>- `agent_color_icon()` adds badge to agent entry names |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 7. Agent File Format

| Field | Value |
|-------|-------|
| **Name** | Agent definition files |
| **Features** | - TOML-based agent definitions<br>- Fields: id, display_name, model_override, prefer_tier, tool_names, disallowed_tools, spawnable_agents, system_prompt, instructions_prompt, step_prompt, spawner_prompt, inherit_parent_system_prompt, include_message_history, permission_mode, max_turns, output_mode, output_schema, color<br>- Loaded from `~/.jcode/agents/<id>.toml` and `.jcode/agents/<id>.toml` |
| **References** | CCB: `.claude/agents/*.md` (Markdown + YAML frontmatter) |
| **jcode Implementation** | - `jcode-agent-runtime/src/definition.rs`: `AgentDefinition` struct (TOML)<br>- `jcode-agent-runtime/src/registry.rs`: `AgentRegistry` with `register_builtin()`, `load_directory()`, `load_file()` |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 8. AgentRegistry & Loading

| Field | Value |
|-------|-------|
| **Name** | Agent registry and discovery |
| **Features** | - 3-tier priority: Builtin < UserGlobal < ProjectLocal<br>- Builtin agents registered programmatically<br>- User agents from `~/.jcode/agents/<id>.toml`<br>- Project agents from `.jcode/agents/<id>.toml`<br>- Sorted iteration, conflict resolution |
| **References** | CCB: `.claude/agents/` with 4 scopes (managed, project, user, plugin) |
| **jcode Implementation** | - `jcode-agent-runtime/src/registry.rs`: `AgentRegistry` struct<br>- `load_directory()`, `register_builtin()`, `load_file()`, `iter_sorted()`, `get()` |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 9. Agent Colors

| Field | Value |
|-------|-------|
| **Name** | Agent color badges |
| **Features** | - 8 named colors: red, blue, green, yellow, purple, orange, pink, cyan<br>- Color stored in `AgentDefinition.color` field<br>- Displayed as badge `●` in agent list + detail shows `color: blue` |
| **References** | CCB: `packages/builtin-tools/src/tools/AgentTool/agentColorManager.ts`, `src/components/agents/ColorPicker.tsx` |
| **jcode Implementation** | - `definition.rs`: `color: Option<String>` field<br>- `openers.rs`: `agent_color_icon()` renders badge<br>- Built-in agents have colors assigned: basher=green, code-reviewer=purple, editor=blue, file-picker=cyan |
| **Progress** | ✅ **Complete** |
| **Remaining** | ACTUAL colored rendering in ratatui (currently uses plain `●` character) — needs PickerEntry color field + Span styling |

---

## 10. AI Agent Generation

| Field | Value |
|-------|-------|
| **Name** | AI-powered agent generation |
| **Features** | - User describes agent in $EDITOR<br>- Description queued as message to current model<br>- Model returns TOML agent definition in chat response |
| **References** | CCB: `src/components/agents/generateAgent.ts` (uses Claude API: `queryModelWithoutStreaming`) |
| **jcode Implementation** | - `inline_interactive.rs`: `PickerAction::GenerateAgent` → opens $EDITOR with prompt template → queues to `self.queued_messages` |
| **Progress** | ⚠️ **Partial** |
| **Remaining** | Auto-parse model response and save agent file automatically (currently user must manually save). Need to hook into turn completion to detect agent TOML in response and save to `~/.jcode/agents/`. |

---

## 11. Context Window Visualization

| Field | Value |
|-------|-------|
| **Name** | Subagent context usage |
| **Features** | - Visualize how much context each subagent uses<br>- Show context window pressure per agent |
| **References** | CCB: `src/commands/context/index.ts`, `src/components/PromptInput/useSwarmBanner.ts` |
| **jcode Implementation** | — |
| **Progress** | ❌ **Not implemented** |
| **Remaining** | Need to track per-subagent context token usage and render in detail overlay or info widget |

---

## 12. `/tasks` Command

| Field | Value |
|-------|-------|
| **Name** | /tasks — background task management |
| **Features** | - List running/completed background tasks<br>- Attach to running task (view output)<br>- Stop/kill task<br>- Similar to running items list but standalone command |
| **References** | CCB: `src/commands/tasks/index.ts`, `src/commands/tasks/tasks.tsx`, `src/hooks/useTasksV2.ts` |
| **jcode Implementation** | — |
| **Progress** | ❌ **Not implemented** |
| **Remaining** | Need to add `PickerKind::Tasks`, `open_tasks_picker()`, handler for Enter/stop. Data already available via `background::global().running_snapshot()`. |

---

## 13. Agent Tool Restrictions

| Field | Value |
|-------|-------|
| **Name** | Per-agent tool allow/deny |
| **Features** | - `tool_names` whitelist: only these tools are available<br>- `disallowed_tools` denylist: block specific tools<br>- `spawnable_agents` whitelist: who this agent can spawn |
| **References** | CCB: `.claude/agents/*.md` frontmatter `tools:` field, `src/Tool.ts` permission system |
| **jcode Implementation** | - `definition.rs`: `tool_names: Vec<String>`, `disallowed_tools: Vec<String>`, `spawnable_agents: Vec<String>`<br>- Enforced at runtime by agent runtime |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 14. Agent Permission Modes

| Field | Value |
|-------|-------|
| **Name** | Subagent permission overrides |
| **Features** | - Override permission mode per agent (Plan, AcceptEdits, etc.)<br>- Override max turns per agent |
| **References** | CCB: `.claude/agents/*.md` frontmatter `permissionMode:`, `maxTurns:` |
| **jcode Implementation** | - `definition.rs`: `permission_mode: Option<PermissionMode>`, `max_turns: Option<u32>`<br>- Built-in agents configure these (e.g., code-reviewer: `permission_mode = "plan"`) |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 15. Model Override for Agent Types

| Field | Value |
|-------|-------|
| **Name** | Built-in agent model override (Swarm, Review, Judge, Memory, Ambient) |
| **Features** | - 5 hardcoded agent types with model override via `model_prefs.json`<br>- Settings stored at config paths<br>- User can override model per agent type |
| **References** | CCB: /agents Library → agent config with `model:` field |
| **jcode Implementation** | - `openers.rs`: model override entries for Swarm, Review, Judge, Memory, Ambient<br>- `inline_interactive/helpers.rs`: `save_agent_model_override()`, `load_agent_model_override()` |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 16. Agent Prompts

| Field | Value |
|-------|-------|
| **Name** | Multi-prompt agent system |
| **Features** | - `system_prompt`: core system prompt<br>- `instructions_prompt`: task-specific instructions<br>- `step_prompt`: per-step instructions<br>- `spawner_prompt`: instructions for parent on when to spawn this agent<br>- `inherit_parent_system_prompt`: share parent's system prompt (prompt cache sharing)<br>- `include_message_history`: carry message history into spawned session |
| **References** | CCB: AgentTool prompts, skill system |
| **jcode Implementation** | - `definition.rs`: all prompt fields defined<br>- Built-in agents use these (e.g., basher: `system_prompt` + `instructions_prompt` + `spawner_prompt`)<br>- Cache sharing: `inherit_parent_system_prompt = true` for editor/code-reviewer |
| **Progress** | ✅ **Complete** |
| **Remaining** | None |

---

## 17. Color Picker UI

| Field | Value |
|-------|-------|
| **Name** | Interactive color picker for agents |
| **Features** | - 8 swatches + "Automatic" option<br>- Live preview of selected color |
| **References** | CCB: `src/components/agents/ColorPicker.tsx` (8 named colors with visual swatches) |
| **jcode Implementation** | — |
| **Progress** | ❌ **Not implemented** |
| **Remaining** | Need `open_color_picker()` with PickerKind::ColorPicker. Currently colors are set by editing TOML file directly. |

---

## 18. Agent Edit Menu

| Field | Value |
|-------|-------|
| **Name** | Agent edit menu (model, tools, color) |
| **Features** | - Change model override<br>- Change tool allowlist<br>- Change color without editing file<br>- Save changes via updateAgentFile + setAppState |
| **References** | CCB: `src/components/agents/AgentEditor.tsx` (edit menu with model/tools/color options) |
| **jcode Implementation** | — |
| **Progress** | ❌ **Not implemented** |
| **Remaining** | Currently editing opens $EDITOR for raw TOML. Need UI pickers for model/tools/color. |

---

## 19. Agent Scopes

| Field | Value |
|-------|-------|
| **Name** | Agent storage scopes |
| **Features** | - 4 scopes: managed (read-only), project (`.claude/agents/`), user (`~/.claude/agents/`), plugin<br>- Each scope has different priority, conflict resolution |
| **References** | CCB: `src/components/agents/agentFileUtils.ts`: `getAgentDirectoryPath(location: SettingSource)`, 4 SettingSource variants |
| **jcode Implementation** | - Openers: only user-global (`~/.jcode/agents/`) and project-local (`.jcode/agents/`)<br>- No managed/plugin scopes |
| **Progress** | ⚠️ **Partial** |
| **Remaining** | Add managed scope (read-only builtin directory), plugin scope. Currently only 2 of 4 scopes. |

---

## 20. `/agents save` Command

| Field | Value |
|-------|-------|
| **Name** | Save generated agent from chat response |
| **Features** | - Save agent definition from last model response<br>- Parse TOML from response code block<br>- Save to `~/.jcode/agents/<id>.toml` |
| **References** | CCB: agent creation → auto-save to `.claude/agents/` after AI generation |
| **jcode Implementation** | — |
| **Progress** | ❌ **Not implemented** |
| **Remaining** | Need `/agents save` command that parses last assistant message for TOML agent definition and saves to disk. Currently user must manually extract from chat response. |

---

## Summary

| # | Feature | Progress |
|---|---------|----------|
| 1 | Running items list | ✅ Complete |
| 2 | Detail overlay | ✅ Complete |
| 3 | Enter to subagent session | ✅ Complete |
| 4 | Stop/kill running item | ✅ Complete |
| 5 | /agents Running tab | ✅ Complete |
| 6 | /agents Library tab | ✅ Complete |
| 7 | Agent file format | ✅ Complete |
| 8 | AgentRegistry & loading | ✅ Complete |
| 9 | Agent colors | ✅ Complete |
| 10 | AI agent generation | ⚠️ Partial |
| 11 | Context window visualization | ❌ Missing |
| 12 | /tasks command | ❌ Missing |
| 13 | Agent tool restrictions | ✅ Complete |
| 14 | Agent permission modes | ✅ Complete |
| 15 | Model override for agent types | ✅ Complete |
| 16 | Agent prompts | ✅ Complete |
| 17 | Color picker UI | ❌ Missing |
| 18 | Agent edit menu | ❌ Missing |
| 19 | Agent scopes | ⚠️ Partial |
| 20 | /agents save command | ❌ Missing |

**Complete:** 14 / 20 (70%)
**Partial:** 2 / 20 (10%)
**Missing:** 4 / 20 (20%)

### Next priorities

1. **`/tasks` command** — reuses existing `background::global().running_snapshot()` infrastructure
2. **`/agents save`** — parse TOML from last assistant message, save to disk
3. **AI generation auto-save** — hook into turn completion to auto-save model-generated definitions
4. **Color picker UI** — inline interactive picker with 8 color swatches
