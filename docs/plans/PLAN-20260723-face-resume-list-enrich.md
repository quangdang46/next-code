# Plan Report — Face `--resume` list row enrichment

## Summary (read this first)
- **You asked:** Enrich Face resume-browser left rows toward legacy density + Claude Code–inspired briefs; survey CC session picker UI first.
- **What is going on:** Dual-entry 2-panel `--resume` shipped with **single-line** rows (title + relative time). List API used startup stubs → `numMessages` often **0**; no `firstPrompt`; right pane had only a “Preview” label.
- **We recommend / shipped:** Multi-line Face-styled briefs (title · time; counts; `prompt:` when title is memorable short_name; truncated cwd); list payload adds `firstPrompt` + real counts from flat store (+ journal); light right-pane header (title, cwd, model).
- **Risk:** Low–Medium (list scan cost per session file; UI scroll math for multi-line rows)
- **Status:** Building / shipping on `pr-face-resume-dual-entry`

## Claude Code research (LOOK)

Source tree: `.tmp-research-plugins/claude-code`

| Surface | Path | What each row shows |
|---------|------|---------------------|
| Data model | `src/types/logs.ts` `LogOption` | `firstPrompt`, `messageCount`, `customTitle`, `summary`, `modified`/`created`, `gitBranch`, `projectPath`, `tag`, file size, PR fields |
| Title | `src/utils/log.ts` `getLogDisplayTitle` | Priority: **agentName → customTitle → summary → firstPrompt** (skip autonomous tick tags) → truncated session id |
| Metadata line | `src/utils/format.ts` `formatLogMetadata` | **`relative time · [branch] · N messages`** (+ optional `#tag`, `@agent`, PR). Project path appended when browsing all projects |
| List UI | `src/components/LogSelector.tsx` `buildLogLabel` / `buildLogMetadata` | **2 conceptual lines**: title; dim metadata. Fork grouping / sidechain suffix. Not a 3-line prompt block |
| Preview | `src/components/SessionPreview.tsx` | Full transcript; footer: **`relative time · N messages · branch`** |

**Scannability takeaways (adapt, don’t clone branding):**
1. Primary label = memorable title, not raw id when possible.
2. One dense meta line with time + message count (CC); legacy next-code TUI adds user/assistant split + optional `prompt:` + cwd.
3. Prefer **prompt as secondary** when the primary title is only a short memorable name (CC folds prompt into title; we keep short_name primary and show `prompt:` under it — closer to legacy density).
4. Preview chrome benefits from a light identity header (title / cwd / model), Face-themed — CC’s preview is messages-first with a small footer meta strip.
5. **No** requirement to add CC-style server/project tree grouping for this pass (not cheap; list is already flat-store local).

## Steps
1. [x] Document CC brief fields (this file)
2. [x] Extend `list_nextcode_sessions` with `firstPrompt`, better `numMessages`, user/assistant counts from snapshot+journal
3. [x] Extend `SessionPickerEntry` + `parse_session_picker_entries`
4. [x] Multi-line left rows + right-pane header in `resume_browser`
5. [x] Tests + rebuild/install + push PR #67

## Files to touch
- `docs/plans/PLAN-20260723-face-resume-list-enrich.md` — this note
- `src/cli/face_auth.rs` — list payload enrichment
- `crates/xai-grok-pager/src/app/app_view.rs` — entry fields
- `crates/xai-grok-pager/src/app/effects/helpers.rs` — parse
- `crates/xai-grok-pager/src/views/resume_browser.rs` — multi-line + header
- Call sites constructing `SessionPickerEntry`

## Out of scope
- Defaulting `NEXT_CODE_LEGACY_TUI`
- Server / project tree grouping
- Changing `/resume` expand-card chrome beyond shared entry fields

## Shipped
- Commit `7c8c30d6e` on `pr-face-resume-dual-entry` → PR #67
- Local install: `%LOCALAPPDATA%\next-code\builds\versions\7c8c30d6e\` (bin replaced via rename)
