# Spike B: Message / Block Format — Grok vs Next-Code

## Date: 2026-07-17
## Status: ✅ Complete

---

## 1. Grok Scrollback Blocks (RenderBlock)

Grok's TUI renders everything as **blocks** in a unified scrollback. There is no concept of "messages" in the TUI — everything is a `RenderBlock` variant:

```rust
pub enum RenderBlock {
    Stub(StubBlock),
    UserPrompt(UserPromptBlock),     // User text + images
    AgentMessage(AgentMessageBlock), // AI text response
    ToolCall(ToolCallBlock),         // ⚠️ ToolCallBlock is a mega-variant with sub-kinds:
        // EditToolCallBlock, ExecuteToolCallBlock, ReadToolCallBlock,
        // ListDirToolCallBlock, SearchToolCallBlock, OtherToolCallBlock
    Thinking(ThinkingBlock),         // Thinking/reasoning 
    System(SystemMessageBlock),
    SessionEvent(SessionEventBlock), // Turn completed, cancelled, failed
    BgTask(BgTaskBlock),             // Background tasks (always collapsed)
    Subagent(SubagentBlock),         // Subagent lifecycle
    Btw(BtwBlock),                   // Side-question response
    ContextInfo(ContextInfoBlock),   // /context snapshot
    CreditLimit(CreditLimitBlock),   // Upsell card
}
```

Block common interface (via `BlockContent` trait):
- `output(ctx) -> BlockOutput` — render to ratatui `Line`s
- `accent(ctx) -> Option<AccentStyle>` — accent color/animation
- `is_foldable() -> bool` — collapse/expand
- `display_mode` — Collapsed | Truncated | Expanded
- `is_groupable() -> bool` — dense rendering
- `has_bullet() -> bool` — bullet icon

**Blocks come from ACP events** — the pager receives them from the leader, it doesn't construct them:
- `acp::SessionEvent::StreamingDelta(Delta(text))` → `AgentMessageBlock`
- `acp::SessionEvent::ToolCall(...)` → `ToolCallBlock`
- `acp::SessionEvent::Thinking(...)` → `ThinkingBlock`
- etc.

---

## 2. Next-Code Message Types

next-code's representation is **message-oriented**, not block-oriented:

```rust
pub enum ContentBlock {
    Text { text, cache_control },
    Reasoning { text },
    ReasoningTrace { text },
    AnthropicThinking { thinking, signature },
    OpenAIReasoning { id, summary, encrypted_content, status },
    ToolUse { id, name, input, thought_signature },
    ToolResult { tool_use_id, content, is_error },
    Image { media_type, data },
    OpenAICompaction { encrypted_content },
}

pub struct Message {
    pub role: Role (User | Assistant),
    pub content: Vec<ContentBlock>,
    pub timestamp: Option<DateTime>,
    pub tool_duration_ms: Option<u64>,
}
```

---

## 3. Semantic Mapping

| Grok RenderBlock | Next-Code ContentBlock | Notes |
|---|---|---|
| `UserPromptBlock` | `Message(role:User)` with `Text` + `Image` blocks | User sends via prompt_widget |
| `AgentMessageBlock` | `Message(role:Assistant)` with `Text` | Assisted via stream delta |
| `ToolCallBlock::Edit` | `ToolUse` → `ToolResult` pair | Diff file edit |
| `ToolCallBlock::Execute` | `ToolUse` → `ToolResult` pair | Shell command |
| `ToolCallBlock::Read` | `ToolUse` → `ToolResult` pair | Read file |
| `ToolCallBlock::Search` | `ToolUse` → `ToolResult` pair | Code search |
| `ToolCallBlock::ListDir` | `ToolUse` → `ToolResult` pair | ls directory |
| `ToolCallBlock::Other` | `ToolUse` → `ToolResult` pair | Any other tool |
| `ThinkingBlock` | `ContentBlock::Reasoning` or `AnthropicThinking` or `OpenAIReasoning` | Depends on provider |
| `SystemMessageBlock` | — | Internal system messages |
| `SessionEventBlock` | — | Turn lifecycle events |
| `BgTaskBlock` | — | Background task indicator |
| `SubagentBlock` | — | Multi-agent lifecycle |
| `BtwBlock` | — | Side questions |
| `ContextInfoBlock` | — | Context stats |

**Key insight:** Grok blocks are **display-oriented** (collapsible, accent-colored, bulleted). next-code messages are **model-oriented** (role, content blocks, token tracking).

The gap is not gap — Grok pager already receives blocks from ACP as opaque rendered content. The **adapter** (shim) constructs `RenderBlock`s from next-code's agent output:

```
next-code agent produces:
  [Message(role:Assistant, content: [Text("Hello"), ToolUse("edit", ...)])]
  
ACP shim translates to:
  [AgentMessageBlock("Hello"), ToolCallBlock::Edit(...)]
  
Pager renders:
  [Block with accent + bullet + foldable output]
```

The translation is straightforward: for each turn result from next-code. This is where the `GrokHost` trait in the adapter does the work.

---

## 4. Compatible Paths

| Path | Grok | Next-Code | Compatible? |
|------|------|-----------|:-----------:|
| Message text | `UserPromptBlock.text` | `ContentBlock::Text { text }` | ✅ Direct |
| Reasoning | `ThinkingBlock.thoughts` | `ContentBlock::Reasoning { text }` | ✅ Direct |
| Tool edit/read | `EditToolCallBlock` | `ToolUse { name:"edit" }` + `ToolResult` | ✅ Chat template→block |
| Tool execute | `ExecuteToolCallBlock` | `ToolUse { name:"bash" }` + `ToolResult` | ✅ |
| File search | `SearchToolCallBlock` | `ToolUse { name:"search" }` + `ToolResult` | ✅ |
| Images | `UserPromptBlock.images` | `ContentBlock::Image { data }` | ✅ Direct |
| Subagent | `SubagentBlock` | next-code has swarm-core | 🟡 Cross-reference |
| Session events | `SessionEventBlock` | Server has turn events | 🟡 Need conversion |

**No incompatible paths — all have a 1:1 or adapter-friendly mapping.**

---

## 5. Where Blocks Come From (ACP → Pager)

The pager receives blocks through ACP `SessionNotification` events from the leader:

```
ACP SessionEvent (from leader)
  ├── StreamingDelta { block_index: usize, text: String }
  │     → appends text to the `blocks[idx]` AgentMessageBlock
  ├── ToolCallStart { tool_name, args }
  │     → insert ToolCallBlock::Execute or Edit or Read etc.
  ├── ToolCallOutput { stream: String }
  │     → appends to the running ToolCallBlock
  ├── ThinkingStart/ThinkingDelta/ThinkingEnd
  │     → insert/appends ThinkingBlock 
  ├── TurnComplete / TurnCancelled / TurnFailed
  │     → link to SessionEventBlock
  ├── SubagentStarted / SubagentCompleted
  │     → SubagentBlock
  └── ...
```

The pager's `app/dispatch/` module processes these events and inserts/appends blocks to the current agent's scrollback.

**For the adapter:** instead of receiving these from ACP, the shim:
1. Polls next-code server events (via `ServerEvent` → translates to ACP-style events)
2. Feeds them to the pager's existing dispatch module
3. Pager draws them as normal blocks

```
Next-Code ServerEvent:
  ToolActivity { session_id, tool_name, status, output }
  → ACP shim → AcpClientMessage::SessionNotification(SessionEvent::ToolCallOutput(...))
  → dispatch → pager's existing handle_acp_message()
```

This means the pager's block rendering code needs **zero changes** — the shim just translates next-code events into the ACP event format the pager already understands.
