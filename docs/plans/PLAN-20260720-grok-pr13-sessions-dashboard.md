# Plan Report — PR13 Sessions dashboard / picker parity

## Summary (read this first)
- **You asked:** Session list/resume inside Face for next-code disks.
- **What is going on:** Resume-by-id works; picker/dashboard may hit empty/foreign stubs.
- **We recommend:** **Keep Face picker UI (copy UX)**. **Wire** data → `~/.next-code/sessions`. **Delete** fake foreign demos. Do **not** revive old TUI session picker UI.
- **Risk:** Medium  
- **Status:** Implemented on `pr-13-sessions-dashboard` (stacked on PR12 / #58).

## Workflow map (required)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | Face `session_picker` / welcome UX | Rebuild picker in `next-code-tui` |
| **Wire** | List/meta → next-code journal/meta; open → `LoadSession`/`ResumeSession` | Point at grok.com sessions |
| **Delete** | Empty foreign-session placeholders | Face dashboard chrome |

## Research first (LOOK)
1. Face: `views/session_picker.rs`, welcome, dashboard.  
2. next-code session list (`tui_launch::list_sessions` as **data** reference only).  
3. grok-build `SessionPickerEntry` fields.

## Evidence

| Claim | Citation | Status |
|-------|----------|--------|
| Resume attach path exists | `pager_agent.rs` `attach_session` / `load_session` | verified |
| Picker data source | `x.ai/session/list` → `face_auth::list_nextcode_sessions` | verified |
| Session on-disk format | `~/.next-code/sessions/{id}.json` (+ `.journal.jsonl`) | verified |
| Face parse fields | `helpers.rs` `parse_session_picker_entries` (`cwd`, `modelId`, `source`, `lastActiveAt`, …) | verified |
| Delete effect | Face `Effect::DeleteSession` → `x.ai/session/delete` | verified (now handled in `face_ext`) |
| Bare `--resume` | Empty id kept for Face; `NEXT_CODE_OPEN_SESSION_PICKER_AT_STARTUP` → `ShowSessionPicker` | verified |

## Copy / wire / delete
| Action | What |
|--------|------|
| **Wire** | Picker ← next-code sessions (cwd/model/source/summary) |
| **Wire** | CLI `--resume` bare → Face picker when TTY (not continue-last / not legacy TUI list) |
| **Wire** | `x.ai/session/delete` removes snapshot + journal |
| **Delete** | Fake foreign entries (out of scope if not present; hide cloud rows — prefer hide / don’t invent) |

## Implementation steps
1. [x] Field map `SessionPickerEntry` ← meta (`cwd`, `modelId`, `source=local`, `lastActiveAt`, `summary` via `display_title_or_name`).  
2. [x] List + open (+ delete).  
3. [x] CLI bare resume → Face `ShowSessionPicker` (legacy TUI still uses `list_sessions`).  
4. [x] Tests (`session_list_and_delete_roundtrip_against_flat_store`).

## Manual verify
Welcome / `/resume` shows sessions; open loads transcript; bare `--resume` opens Face picker; delete removes disk files.

## Open questions (resolved for this PR)
1. Rename/delete: rename already in `face_ext`; delete now wired.  
2. Share/cloud rows: hide / do not invent cloud rows.

## Out of scope
Cloud sync; rewriting Face picker visuals; PR11 legacy TUI delete; PR14.

## Done when
Session discovery is next-code-native in Face UI.
