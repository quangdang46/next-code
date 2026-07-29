# Plan — Face AskUserQuestion multi-question / multi-select parity

**Date:** 2026-07-24  
**Branch:** `pr-face-ask-user-multiquestion`  
**Status:** Implemented — Phase A (schema/`header`/`multiSelect`) + Phase B (chip tab bar)  
**Related:** [PLAN-20260722-face-ask-user-question-wire.md](./PLAN-20260722-face-ask-user-question-wire.md) (wire landed via PR #76/#77); docs plan was PR #83.

---

## Summary (read this first)

- **User ask:** Claude Code AskUserQuestion cho nhiều checkbox, radio khi cần, và nhiều tab/câu trong một tool call — Face hiện chỉ thấy radio + 1 câu.
- **LOOK verdict:** Face **đã có** runtime multi-select (checkbox) + multi-question paging (`active_tab`, ←/→, footer `[1/N]`). Gap thật là **(1)** schema/prompt model không đủ gần Claude → model ít khi emit `multiSelect` / nhiều `questions`, **(2)** thiếu **header chip tab bar** như Claude `QuestionNavigationBar`, **(3)** thiếu field `header` + giới hạn 1–4 câu / 2–4 options trên tool schema.
- **Recommend MVP:** Phase A schema+prompt parity trước (rẻ, làm model emit đúng payload), rồi Phase B chip tab bar; Phase C chỉ smoke/fix checkbox path nếu còn bug thật.

---

## Evidence (LOOK)

| # | Claim | Status | Citation |
|---|-------|--------|----------|
| E1 | Claude schema: `questions` 1–4; each `question`, **required `header`** (chip ≤12 chars), `options` 2–4, **`multiSelect`** default false | **verified** | `.tmp-research-plugins/claude-code/.../AskUserQuestionTool.tsx` (+ DeepWiki `claude-code-best/claude-code`); Exa fetch same file on GitHub |
| E2 | Claude UI: `QuestionNavigationBar` — ← / header chips with answered checkbox / optional Submit tab / → | **verified** | `AskUserQuestionPermissionRequest/QuestionNavigationBar.tsx` |
| E3 | Claude answers wire to model: `Record<questionText, string>`; multi-select comma-joined | **verified** | tool `outputSchema` + `mapToolResultToToolResultBlockParam` |
| E4 | Official Agent SDK docs same shape (`multiSelect`, multi `questions`) | **verified** | https://code.claude.com/docs/en/agent-sdk/user-input |
| E5 | next-code tool already accepts `questions[]` + `multi_select` boolean; no `header`; no maxItems | **verified** | `crates/next-code-app-core/src/tool/ask_user_question.rs` `parameters_schema` |
| E6 | Wire `Question` has `multi_select` (camelCase + snake alias); ACP answers `IndexMap<String, Vec<String>>` then join at format | **verified** | `xai-grok-tools/.../ask_user_question/mod.rs` + `types.rs` + `format.rs` |
| E7 | Face `question_view` already branches checkbox vs radio on `multi_select`; multi-q via `active_tab` | **verified** | `crates/xai-grok-pager/src/views/question_view.rs` |
| E8 | Face multi-q chrome today: footer `[i/N]` + ←/→ hints — **not** Claude header chip bar | **verified** | `agent_view/render.rs` ~2636–2646 |
| E9 | PR #76/#77 shipped AskUserQuestion Face wire (`face_ask_user`, tool registry, protocol) | **verified** | `gh pr view 76/77` |

---

## Gap matrix

| Claude Code capability | Face / next-code today | Gap |
|------------------------|------------------------|-----|
| Multi-select checkboxes (`multiSelect: true`) | Runtime: yes (`QuestionSelection::Multi`, `[x]`/`[ ]`) | **Model/schema:** tool JSON exposes `multi_select`; description thinner than CC prompt → models rarely set it. Serialize path OK if flag arrives. |
| Single-select radio (`multiSelect: false`) | Runtime: yes (`(●)`/`(○)`) | None for paint; matches CC default. |
| Multiple questions in one tool call (1–4) | Runtime: yes (`questions: Vec`, Left/Right / Enter advances) | **Discoverability + schema:** no `header` chips; no `maxItems: 4` / options 2–4; model often emits 1 question. |
| Tab / chip nav bar keyed by `header` | Footer counter `[1/N]` only; no chip row; no answered ✓ on chips; no Submit tab | **UX parity gap** — largest visual difference vs CC. |
| Required short `header` per question | Field **absent** on `Question` | Schema + wire + UI need optional→required migration. |
| Preview side-by-side (single-select only) | Face has preview chrome/caps in `question_view` | Out of MVP unless smoke shows broken; track as follow-up. |
| Answers: string map, multi joined by `", "` | ACP: `Vec<String>` per question; format joins with `", "` for model text | Compatible for model-facing result; keep Vec on ACP (strict). |
| Plan-mode interview extras | Face `AskUserQuestionMode::Plan` (Chat about this / Skip) | Wired; mode often forced `"default"` in tool — separate from multi-q. |
| Auto “Other” freeform | Face freeform row + annotations notes | Parity OK. |

### Wire schema diff (exact)

**Claude input (model → tool):**
```json
{
  "questions": [
    {
      "question": "How should I format the output?",
      "header": "Format",
      "options": [
        { "label": "Summary", "description": "Brief overview" },
        { "label": "Detailed", "description": "Full explanation" }
      ],
      "multiSelect": false
    },
    {
      "question": "Which sections should I include?",
      "header": "Sections",
      "options": [ ... ],
      "multiSelect": true
    }
  ]
}
```

**next-code tool input today:**
```json
{
  "questions": [
    {
      "question": "...",
      "options": [{ "label": "...", "description": "...", "preview": "..." }],
      "multi_select": true
    }
  ]
}
```

| Field | Claude | next-code |
|-------|--------|-----------|
| `header` | required, ≤12 chars | **missing** |
| multi-select flag | `multiSelect` | `multi_select` in parameters_schema (serde accepts camelCase on `Question`) |
| questions count | min 1 max **4** | min 1, **no max** |
| options count | min 2 max **4** | unbounded in schema |
| uniqueness | question text + labels | duplicate question text rejected in execute; labels not schema-enforced |

**Claude output (tool result / permission updatedInput):**  
`answers: { "<question text>": "<label>" | "a, b" }` (+ optional `annotations`).

**Face ACP response:**  
`{ "outcome": "accepted", "answers": { "<question text>": ["a","b"] }, "annotations": ... }`  
→ `format_accepted_tool_result` joins labels → same model-visible sentence shape as CC.

---

## Non-goals

- Re-implement AskUserQuestion wire / ACP reverse path (done in #76/#77).
- Port Claude Ink `AskUserQuestionPermissionRequest` wholesale into Face.
- Change permission overlay / `request_permission`.
- Force-push or merge this docs PR.
- Touch `docs/multitask-mvp-plan` or paste-token Face auth WIP.

---

## Phases (temporary plan)

### Phase A — Schema + prompt parity (MVP slice — do first)

**Goal:** Models emit Claude-shaped questionnaires so Face’s existing multi-q / multi-select runtime lights up.

1. Add optional `header: Option<String>` on `Question` (wire + serde camelCase). Prefer accept-without-fail if missing; truncate display to 12 for chips later.
2. Align tool `parameters_schema` + description with CC: document `multiSelect` **and** `multi_select`, `header`, 1–4 questions, 2–4 options, recommend first option `(Recommended)`.
3. Optionally tighten validation (max 4 questions / 2–4 options / unique labels) — fail soft or hard TBD in BUILD.
4. Unit tests: deserialize `multiSelect` + `header`; format still joins multi answers.

**Files:** `ask_user_question/{mod,types}.rs`, `next-code-app-core/.../ask_user_question.rs`, format tests.

**Smoke:** Force a tool call with 2 questions (one `multiSelect: true`) via playground or scripted agent → Face shows checkbox on Q2 and advances with ←/→.

### Phase B — Multi-question tab / chip bar (CC UX)

**Goal:** Visible nav like `QuestionNavigationBar`.

1. Render chip row above question chrome when `questions.len() > 1`: `header` or fallback `Q{n}`; answered state from selections; highlight `active_tab`.
2. Optional Submit chip on last step (or keep Enter-on-last = submit — match CC `hideSubmitTab` when single single-select).
3. Mouse hit-targets on chips (extend `question_nav_buttons` or new hit list).
4. Keep footer `[i/N]` + ←/→ as secondary hints or trim once chips prove discoverable.

**Files:** `question_view.rs` (chrome), `agent_view/render.rs`, `agent_view/interactions.rs`, playground.

**Smoke:** 3-question fixture → chips visible; click/←→ switch; answered chip marks; submit from last.

### Phase C — Mixed radio + checkbox per question (verify)

**Goal:** Confirm mixed questionnaire in one call (already intended by per-question `multi_select`).

1. Playground + integration fixture: Q1 radio, Q2 checkbox, Q3 radio.
2. Fix any selection/submit bugs only if smoke fails (do not invent new selection model).
3. Document model guidance: mutually exclusive → radio; non-exclusive → `multiSelect: true`.

### Phase D (follow-up, not MVP)

- Preview side-by-side parity polish.
- Plan-mode `AskUserQuestionMode::Plan` mapping from next-code plan sessions.
- Enforce `header` required once models are updated.

---

## Recommended MVP order

1. **Phase A** (schema/prompt) — unblocks “nhiều câu + checkbox” without big UI.  
2. **Phase B** (chip tabs) — closes the “không thấy tab” complaint.  
3. **Phase C** — smoke only unless broken.

---

## Smoke checklist (post-implementation)

- [ ] Single radio question still works (regression).
- [ ] One question `multiSelect: true` → checkboxes; Space toggles; submit joins labels.
- [ ] Two+ questions → chip bar (after B) or at least `[i/N]` + ←/→ (today); answers map all keys.
- [ ] Mixed radio+checkbox in one call.
- [ ] Cancel / timeout still return cancel text (not tool error).
- [ ] `cargo check -p next-code-app-core -p xai-grok-pager`; targeted question_view tests.

---

## Open questions (≤3)

1. Make `header` required in schema (CC-strict) or optional with `Q{n}` fallback until models adapt?
2. Expose only `multiSelect` in JSON schema (CC name) vs dual `multi_select`/`multiSelect`?
3. Chip bar Submit tab vs keep Enter-on-last-only (Face today)?

---

## Status

**Implemented (Phase A + B).** Phase C remains smoke-only verification.

Smoke checklist:
- [x] Schema exposes `header`, `multiSelect`, 1–4 questions, 2–4 options
- [x] Wire deserializes Claude-shaped payloads (`header` + `multiSelect`)
- [x] Multi-question chip tab bar renders with answered `[x]` / unanswered `[ ]`
- [x] Chip click + ←/→ switch tabs; radio + checkbox mix preserved
- [ ] Live Face smoke with model-emitted multi-question payload (manual)
