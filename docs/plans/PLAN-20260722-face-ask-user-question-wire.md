# Plan Report — Face AskUserQuestion wire

## Summary (read this first)
- **You asked:** Wire AskUserQuestion the Face-migration way: **copy → wire → delete old logic**. No production code until OK.
- **What is going on:** Face already opens `question_view` on ACP reverse `ext_method("x.ai/ask_user_question")`. Stock grok-build’s brain emits that reverse request from an `AskUserQuestion` tool + shell coordinator. next-code’s daemon/tool registry has **no** AskUserQuestion tool and never calls `Client::ext_method` for questions; interactive UX today is permission overlays / legacy `StdinRequest` freeform — not structured Q&A.
- **We recommend:** Copy the **stock tool + wire types + format helpers** from grok-build into `xai-grok-tools` (replace today’s type-only stub), then wire a thin next-code bridge: app-core tool → pager_agent/daemon → `gateway.ext_method("x.ai/ask_user_question")` → Face `handle_ask_user_question` (already present). Do **not** reimplement question UI in `next-code-tui`. Prefer Grok ACP wire over Claude Code Best’s in-process elicitation (CC is prior art for *having* the tool, not for Face paint).
- **Risk:** Medium — blocking reverse-request + timeout + plan-mode outcomes; Face UI is already proven.
- **Status:** Waiting for your OK — reply **go ahead** to implement

### Copy / wire / delete map

| Kind | What | Notes |
|------|------|--------|
| **Copy** | Stock `ask_user_question` tool body (`mod.rs` Tool impl, `format`, full `types` incl. `UserQuestionSender` / oneshot coordinator types), timeout consts/params | Vendored tree today is **types-only** (`AskUserQuestionExtRequest/Response`, `Question`) — incomplete vs upstream |
| **Copy (keep)** | Face `question_view` + `acp_handler::interactions::handle_ask_user_question` | Already vendored; do not rewrite |
| **Wire** | Register next-code tool → emit reverse ACP → await `AskUserQuestionExtResponse` → format tool result for model | Seam: app-core tool + `pager_agent` `AcpGatewaySender<AgentSide>::ext_method` |
| **Wire** | Timeout / `[toolset.ask_user_question]` / `GROK_ASK_USER_QUESTION_TIMEOUT_SECS` (or next-code config mirror) | Face settings already expose timeout toggle |
| **Delete / gate** | Do **not** build a parallel TUI question dialog; do **not** treat ambient `request_permission` as AskUserQuestion | Keep permission path separate |
| **Delete / gate** | Legacy freeform `ServerEvent::StdinRequest` must **not** become the Face Ask path; gate any “fake QuestionsSent / fire-and-forget” fallback once reverse wire works | Stock had `MIGRATION_FALLBACK`; next-code should fail closed or cancel, not pretend success |
| **Do not delete** | Face `permission_view` | Different interaction (ACP `request_permission`); PR9 U7 notes daemon may not fire it — out of scope here |

---

## Feature planning
- **Recommended approach:** One blocking reverse-request per tool call, same three-crate contract as stock (tools ↔ shell/coordinator ↔ pager). In next-code, the “shell coordinator” role is **`NextCodeFaceAgent` / `pager_agent` + daemon tool runtime**, not stock `xai-grok-shell` session actor. Face stays paint-only.
- **Prior art (GitHub):**
  - [xai-org/grok-build ask_user_question/mod.rs](https://raw.githubusercontent.com/xai-org/grok-build/main/crates/codegen/xai-grok-tools/src/implementations/grok_build/ask_user_question/mod.rs) — Tool + timeout + `UserQuestionSender` path
  - [types.rs contract](https://raw.githubusercontent.com/xai-org/grok-build/main/crates/codegen/xai-grok-tools/src/implementations/grok_build/ask_user_question/types.rs) — tools / shell / pager share ExtRequest/ExtResponse
  - [pending_interaction.rs](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-shell/src/session/pending_interaction.rs) — Question is a **blocking reverse-request** alongside Permission / PlanApproval
  - [claude-code-best](https://github.com/claude-code-best/claude-code) registers `AskUserQuestionTool` in `getAllBaseTools()` — product parity signal; **not** the Face wire
- **Integration points:** `xai-grok-tools` → `next-code-app-core` tool registry → Face ACP via `pager_agent` gateway
- **Sub-agents used:** skipped — LOOK scoped to grok-migration skill + local tree + Exa (DeepWiki MCP unavailable this session)
- **Option B (avoid):** Re-home question UI into `next-code-tui` / invent a new `ServerEvent::AskUserQuestion` paint path for Face — violates Face-migration north star
- **Open questions:** see below (≤3)

---

## Evidence

| # | Claim | Status | Citation |
|---|-------|--------|----------|
| E1 | Face handles reverse `x.ai/ask_user_question` → opens `question_view`, parks `response_tx` | **verified** | `crates/xai-grok-pager/src/app/acp_handler/mod.rs` (`handle_ext_method` match ~947–950); `interactions.rs` `handle_ask_user_question` |
| E2 | Face agent state already has `question_view: Option<QuestionViewState>` | **verified** | `crates/xai-grok-pager/src/app/agent_view/mod.rs` (~1212–1214) |
| E3 | Vendored `xai-grok-tools` ask_user_question is **wire types only** (no Tool execute / no `UserQuestionSender`) | **verified** | `crates/xai-grok-tools/src/implementations/grok_build/ask_user_question/{mod,types}.rs` vs upstream full `mod.rs` |
| E4 | Stock flow: tool blocks on oneshot; shell calls client ACP `ext_method`; pager returns typed outcome | **verified** | Upstream `types.rs` header comment (tools / shell / pager); `AskUserQuestionExtRequest` / `AskUserQuestionExtResponse` |
| E5 | ACP gateway supports **Agent→Client** `ext_method` (blocking forward) | **verified** | `crates/xai-acp-lib/src/gateway.rs` `impl Client for AcpGatewaySender<AgentSide>` `ext_method` → `self.forward(args)` (~459–461) |
| E6 | `pager_agent` has `gateway: AcpGatewaySender<AgentSide>` and uses it for `session_notification`; **never** sends ask_user_question reverse | **verified** | `src/cli/pager_agent.rs` (~247, ~1166+) — `Agent::ext_method` is Face→agent (auth/skills/mcp), not question emit |
| E7 | next-code app-core tool registry has no AskUserQuestion; ambient has `request_permission` only | **verified** | `crates/next-code-app-core/src/tool/mod.rs` insert list + `register_ambient_tools` (~1510–1514); grep zero `AskUserQuestion` under app-core |
| E8 | Gap already tracked as PR9 U8 / A3 (deferred; A3 wording slightly wrong — need **emit** reverse, not Agent handler) | **verified** | `docs/plans/PLAN-20260720-grok-pr9-face-brain-harden.md` U8/A3 |
| E9 | Claude Code Best ships AskUserQuestion as a core tool | **verified** | `claude-code-best/claude-code` `src/tools.ts` `getAllBaseTools()` includes `AskUserQuestionTool` |
| E10 | Legacy next-code interactive prompt path is `ServerEvent::StdinRequest` (freeform), not structured questions | **verified** | `client_lifecycle.rs` stdin forwarder (~636–661); `next-code-protocol` `StdinRequest` |
| E11 | `ToolOutput::AskUserQuestion` enum stub exists in vendored tools output | **verified** | `crates/xai-grok-tools/src/types/output.rs` (~402) |
| E12 | DeepWiki `xai-org/grok-build` | **unverified this session** — MCP server missing; used Exa + raw GitHub + local vendor | Note in LOOK |

---

## Architecture (target)

```text
Model
  → next-code AskUserQuestion tool (app-core registry)
  → UserQuestionSender / oneshot (copied contract)
  → pager_agent / bridge: Client::ext_method("x.ai/ask_user_question",
        AskUserQuestionExtRequest { session_id, tool_call_id, questions, mode })
  → Face handle_ask_user_question → question_view
  → user Accept / Cancel / (plan) ChatAboutThis | SkipInterview
  → AskUserQuestionExtResponse
  → format → tool result → model continues
```

**Correction vs PR9 A3:** Face already implements the **client** handler. The missing piece is the **agent/brain** emit + tool registration. Implementing another `Agent::ext_method` case for `"x.ai/ask_user_question"` on `pager_agent` would be the wrong direction (that path is Face→daemon).

---

## Phased steps

### Phase 0 — LOOK freeze (done in this report)
- [x] Stock copy surface, Face handler, next-code gap, CC prior art cited

### Phase 1 — Copy stock tool surface into `xai-grok-tools`
1. [ ] Replace stub `ask_user_question/` with upstream modules needed for execute: full `mod.rs`, `format.rs`, complete `types.rs` (`UserQuestionSender`, errors, params).
2. [ ] Keep Face-compatible wire: `AskUserQuestionExtRequest` / `AskUserQuestionExtResponse` / `Question` camelCase.
3. [ ] `cargo check -p xai-grok-tools -p xai-grok-pager` (Face already imports these types).

### Phase 2 — Wire next-code brain
1. [ ] Add `AskUserQuestion` (or `ask_user_question`) to `next-code-app-core` `Registry` default tools — thin adapter over copied types **or** call into grok_build tool with injected `UserQuestionSender`.
2. [ ] Inject sender from Face ACP path: when session runs under `NextCodeFaceAgent`, sender calls `gateway.ext_method` with typed params; map response → tool output text (stock `format`).
3. [ ] Pass `session_id` + `tool_call_id` from tool context; set `mode: Default` until plan-mode mapping is decided (Q2).
4. [ ] Honor timeout: copied `AskUserQuestionParams` / env; on expiry return cancel/skip text (stock behavior), not a hard tool error.
5. [ ] Unit/integration: mock gateway oneshot Accepted + Cancelled; optional Face-side regression already in `acp_handler/tests/interactions.rs`.

### Phase 3 — Delete / gate legacy
1. [ ] Gate or remove any stub “QuestionsSent without waiting” path for Face sessions.
2. [ ] Document: Face AskUserQuestion ≠ ambient `request_permission` ≠ `StdinRequest`; do not route structured questions through stdin.
3. [ ] No new `next-code-tui` question overlay; leave `permission_view` alone.
4. [ ] Update PR9 gap row U8/A3 → done (or pointer to this plan) when shipping.

### Phase 4 — Prove
1. [ ] Interactive: model calls AskUserQuestion → Face overlay → Accept → turn continues with answers.
2. [ ] Cancel path returns Cancelled outcome without red tool failure.
3. [ ] Rebuild/install both `next-code` and `nextcode` aliases if CLI touched.

---

## Files to touch

| Path | Role |
|------|------|
| `crates/xai-grok-tools/src/implementations/grok_build/ask_user_question/**` | **Copy** full tool + types |
| `crates/next-code-app-core/src/tool/mod.rs` (+ new `ask_user_question.rs` or adapter) | **Wire** register tool |
| `src/cli/pager_agent.rs` (and/or small `face_ask_user.rs`) | **Wire** reverse `ext_method` sender injection |
| Daemon/session tool context plumbing (exact file TBD in BUILD — likely agent/tool context where stdin tx is set) | Attach Face-capable `UserQuestionSender` |
| `docs/plans/PLAN-20260720-grok-pr9-face-brain-harden.md` | Mark U8/A3 closed (docs only when shipping) |
| Tests under app-core + optional pager interactions | Prove outcomes |

**Unlikely to touch:** `question_view.rs`, Face render/input (already complete); `permission_view` (separate).

---

## Risk

| Risk | Mitigation |
|------|------------|
| Tool hangs forever if Face never answers / session detached | Stock parks without error when no local view; timeout config; cancel on session close |
| Wrong ACP direction (implement Agent handler instead of Client emit) | Follow E1/E5/E6; review checklist |
| Plan-mode buttons without real plan mode | Ship `mode: Default` first; enable Plan only when next-code has a clear plan session flag (Q2) |
| Dual UX (Stdin + Question) confuses users | Gate Face path explicitly; no TUI re-home |

**Overall:** Medium.

---

## Open questions (≤3)

1. **Tool name for the model:** expose as `AskUserQuestion` (Grok/CC parity) or snake `ask_user_question` (config key / toolset)? Default recommendation: **`AskUserQuestion`** display/schema name matching CC/Grok.
2. **Plan mode:** next-code has swarm/plan DAG “light/deep”, not stock Grok plan session — ship **`AskUserQuestionMode::Default` only** in v1, or map some existing mode to `Plan` so ChatAboutThis / SkipInterview appear?
3. **Coordinator home:** keep oneshot coordinator inside **`pager_agent`** (ACP-adjacent) vs daemon app-core with a channel up to the Face process — which ownership matches other reverse UX (stdin today is daemon→TUI event)? Default recommendation: **daemon owns tool wait; Face bridge in pager_agent performs `ext_method`**, mirrored after stdin forwarder pattern.

---

## Status

Waiting for your OK — reply **go ahead** to implement.
