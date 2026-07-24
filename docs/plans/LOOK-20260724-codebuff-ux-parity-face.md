# LOOK — Codebuff UX parity for Face (best-of-n / agent edit / filepicker)

**Date:** 2026-07-24  
**Scope:** Research / docs only — no product implementation.  
**Question:** “check xem logic UI UX của best of n, agent edit, filepicker,.. đã đúng chưa?” vs [CodebuffAI/codebuff](https://github.com/CodebuffAI/codebuff).  
**Sources:** uploaded GitHub README scrape · DeepWiki (Codebuff CLI) · Exa (docs + `editor-best-of-n` agent page) · `gh` raw `cli/src/*` · next-code Face (`xai-grok-pager`) + `next-code-best-of-n` / `next-code-app-core`

**Non-goals this survey:** pixel clone; keyword-highlight polish; `/btw` sidebar agents (in-flight elsewhere).

---

## Verdict summary

| Area | Verdict | One-line why |
|------|---------|--------------|
| **Best-of-N** | **partial** (engine) / **missing** (Face UX) | next-code has store + tools + auto orchestrator; Face has no parallel-proposal cards, no cancel/pick chrome, `show` mode explicitly stubbed. |
| **Agent edit** | **OK** (normal path) / **partial** (propose/BoN drafts) | Face edit blocks + permission gate are strong for applied diffs; Codebuff-style draft→select→apply UI for BoN proposals is not wired to Face. |
| **Filepicker (`@`)** | **OK** | Face `@` fuzzy picker (dirs/files, `!` hidden, line ranges, insert) matches or exceeds Codebuff mention menu for **files**; missing Codebuff **`@Agent`** mentions. |

---

## LOOK matrix

| Capability | Codebuff (today) | Face / next-code (today) | Gap / OK |
|------------|------------------|--------------------------|----------|
| **BoN spawn N implementors** | `editor-best-of-n` / `GENERATE_N` + spawn implementors; CLI shows `ImplementorGroup` / `ImplementorCard` masonry (`cli/src/components/blocks/implementor-row.tsx`) | `BestOfNOrchestrator` + `run_best_of_n` spawns candidates with `propose_*` only (`crates/next-code-app-core/src/agent/best_of_n_orchestrator.rs`); Face has **no** implementor card UI | **missing** Face surface |
| **BoN progress copy** | “Generating X proposals…”, “X/Y complete…”, “Selecting best…”, “Applying selected changes…” (`getMultiPromptProgress` / agent-branch wrapper) | Orchestrator emits progress strings to host; Face interactive path does not render BoN-specific chrome (only headless `--best-of-n`) | **missing** |
| **BoN pick winner** | **LLM selector agent** auto-picks (`best-of-n-selector*`); user does **not** manually choose among cards | Deterministic selector (`select_best_candidate`) + auto-apply; config `mode=show` intended for user picker but **not implemented** (`turn_execution.rs`: “picker UI is not implemented yet”) | **partial** engine / **wrong** vs advertised `show` |
| **BoN cancel** | Stream interrupt / Esc aborts current op | Normal agent cancel exists; no BoN-run-scoped cancel UX (no live candidate panel to abandon) | **partial** |
| **BoN apply** | Replay winning tool calls (`STEP_TEXT` / extract `<codebuff_tool_call>`) after select | `best_of_n_apply` / `apply_winner` writes staged store to disk | **OK** backend; Face only sees post-hoc text summary if auto path runs |
| **Propose vs apply tools** | `propose_str_replace` / `propose_write_file` draft (diff shown, no disk); `str_replace` / `write_file` apply | `propose_edit` / `propose_hashline` / `propose_write` + `best_of_n_*`; normal path uses search_replace / write / apply_patch | **OK** tools; Face does not specially chrome `propose_*` vs applied edits |
| **Edit review UI** | Inline `DiffViewer` on implementor file click; `StrReplaceComponent` for both propose + apply | `EditToolCallBlock` with hunk HL, coalesce, collapse (`scrollback/blocks/tool/edit.rs`); permission card via `permission_view` before edits | **OK** for applied edits (different model: ACP permission, not propose-stack) |
| **`@` file mention** | Mention menu: files (+ agents); ↑↓ / Tab / Shift+Tab; insert `@path ` (`use-suggestion-engine.ts`, chat keyboard) | `views/file_search/*` + prompt widget: fuzzy daemon, dir mode (`@src/`), hidden (`@!`), drill whitespace dirs, Tab/Right accept, `@path:N-M`, chips | **OK** (file path) |
| **`@` agent mention** | `@AgentName` invokes specialized agents | No `@agent` mention menu; agents via slash / spawn / keywords | **missing** (related) |
| **File-picker agent** | Specialized Fletcher agent finds ≤12–20 paths for context (orchestration, not just TUI menu) | Fuzzy `@` is UI-local; no Codebuff-equivalent file-picker agent in Face chrome (backlog D3 pipeline) | **partial** / related |
| **`/review` scope picker** | `ReviewScreen`: conversation / uncommitted / branch / custom (`cli/src/components/review-screen.tsx`) — **not** BoN winner pick | Face has no Codebuff-style `/review` scope modal (session `/diff` also thin per Claude LOOK) | **missing** (related, not BoN) |

---

## Area deep-dives

### 1. Best-of-N — verdict: **partial** (core) / **missing** (Face UX)

**Codebuff logic + UX**

1. Orchestrator agent fans out N implementors (often via `GENERATE_N` / spawn).
2. CLI paints parallel **ImplementorCard**s (status, ± stats, click file → inline diff).
3. **Selector agent** chooses `implementationId` + reason (simple-text row, not a full agent box).
4. Winning tool calls are applied; UI says selecting / applying.
5. Cancel = interrupt stream (global), not a dedicated “discard candidate #3” control.

**Face / next-code today**

| Layer | Status |
|-------|--------|
| Crate `next-code-best-of-n` | Phase 1–2-ish: store, strategies, deterministic selector, config `auto` / `show` / `off` |
| Tools | `best_of_n_edit` → `propose_*` → `best_of_n_apply` |
| Auto orchestrator | `run_best_of_n` parallel candidates; keyword `$bestofn` / sticky guidance |
| Face interactive | **No** BoN cards, progress strip, or picker; pager only documents **`--best-of-n` headless** |
| `mode=show` | Explicitly falls back to auto-apply + note string |

**Wrong / smell:** advertising `show` without a Face (or TUI) picker is a product lie until Phase 3 lands. Logic of “auto select then apply” is aligned with Codebuff’s default (selector agent), but **visibility** of N proposals is the UX gap users feel.

---

### 2. Agent edit — verdict: **OK** (main path) / **partial** (draft pipeline)

**Codebuff**

- Normal editor: `str_replace` / `write_file` apply to disk; UI shows tool diff.
- BoN / implementor path: prefer **propose_*** so drafts stack and can be shown before apply.
- Reviewer agent is a separate orchestration step (architecture), not Face chrome.

**Face**

- ACP tool calls → `EditToolCallBlock` (multi-hunk, syntect, collapse, untrusted summary).
- Gate: `permission_view` Approve / Always / Reject (generic card; tool-specific polish still thinner than Claude).
- Plan mode still blocks file edits until exit (documented in Face dispatch).
- `propose_*` exists in app-core for BoN but Face does not treat draft proposals as first-class “pick among drafts” UI.

**OK if** the user cares about “see the diff, approve, get syntax-highlighted hunks.”  
**Not OK if** they expect Codebuff implementor cards + propose-before-write for every multi-attempt edit.

---

### 3. Filepicker — verdict: **OK**

**Codebuff:** `@` opens suggestion menu (files + agents); ranking from project file tree + fuzzy; select inserts `@${filePath} `.

**Face:** `@` opens fuzzy file search rooted at agent cwd (worktree-retargetable):

- Ranking: `FuzzyFileMatcherDaemon` top-K.
- Dirs vs files: dir mode when query ends with `/`; drill into directories (incl. whitespace names).
- Hidden: `@!…`.
- Insert: Tab (path + trailing space as chip), Right (no space / dir drill variants), line-range refs `@file:N-M`.
- Docs: `docs/user-guide/01-getting-started.md` + extensive `prompt_widget` tests.

**Gap vs Codebuff (related, not a filepicker bug):** no `@AgentName` in the same menu. Do not confuse with the **file-picker agent** (backend context gatherer) — that is orchestration (D3), not the `@` dropdown.

---

## Related Codebuff patterns (not primary ask)

| Pattern | Face note |
|---------|-----------|
| `@Agent` mentions | Missing |
| Parallel file-picker / editor / reviewer pipeline | Backlog D3 partial; not Face chrome |
| `!` bash mode | Claude LOOK already flagged; Codebuff also has it |
| `/review` scope screen | Missing; distinct from BoN |
| Knowledge.md / custom `.agents/` | Different product surface |

---

## Top 3 fix priorities

1. **Best-of-N Face surface (read-only first)** — While a BoN run is active, show N candidate rows (status + file ± summary + selection reason), even if winner stays auto. Stops the “black box” vs Codebuff ImplementorCard. Touch: ACP/session events from `run_best_of_n_with_progress` → pager block; avoid inventing a second orchestrator.

2. **Implement or demote `mode=show`** — Either ship a minimal arrow/Enter candidate picker that applies the chosen store snapshot, or remove/hide `show` from config UX until ready. Current auto-apply + footnote is **wrong** relative to the mode name.

3. **Chrome `propose_*` drafts in edit blocks** — Label staged proposals vs applied edits (and optionally stack same-file proposes like Codebuff). Unblocks trust during BoN and any future draft-edit flows without waiting on full Implementor masonry.

**Runners-up:** `@Agent` in mention menu · `/review`-style scope picker · wire interactive `$bestofn` progress into Face (not only headless).

---

## Explicit collisions / do-not-touch

- Keyword-highlight work.
- `/btw` sidebar agents (separate Face track).
- Do not treat Codebuff `ReviewScreen` as BoN winner UI (it is `/review` scope).

---

## Research footnotes

| Topic | Where |
|-------|--------|
| Codebuff BoN agent | https://www.codebuff.com/publishers/codebuff/agents/editor-best-of-n/0.0.5 |
| Implementor UI | `cli/src/components/blocks/implementor-row.tsx`, `agent-branch-wrapper.tsx` |
| Mentions | `cli/src/hooks/use-suggestion-engine.ts`, `cli/src/chat.tsx` |
| `/review` screen | `cli/src/components/review-screen.tsx` |
| next-code BoN crate | `crates/next-code-best-of-n/` |
| Tools + orchestrator | `crates/next-code-app-core/src/tool/best_of_n.rs`, `…/agent/best_of_n_orchestrator.rs` |
| `show` stub | `crates/next-code-app-core/src/agent/turn_execution.rs` (~L108–110) |
| Face `@` | `crates/xai-grok-pager/src/views/file_search/`, prompt widget accept helpers |
| Face edits | `…/scrollback/blocks/tool/edit.rs`, `…/views/permission_view.rs` |
| Backlog | `docs/PR_BACKLOG.md` D5 / D3 |
| Prior Face UX survey | `docs/plans/LOOK-20260724-claude-code-ux-gaps-for-face.md` |
