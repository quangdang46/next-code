# Plan Report — PR13 Sessions dashboard / picker parity

## Summary (read this first)
- **You asked:** Session list/resume inside Face for next-code disks.
- **What is going on:** Resume-by-id works; picker/dashboard may hit empty/foreign stubs.
- **We recommend:** **Keep Face picker UI (copy UX)**. **Wire** data → `~/.next-code/sessions`. **Delete** fake foreign demos. Do **not** revive old TUI session picker UI.
- **Risk:** Medium  
- **Status:** After PR9; better after PR12 persistence.

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

## Evidence (fill before BUILD)

| Claim | Citation | Status |
|-------|----------|--------|
| Resume attach path exists | `pager_agent.rs` `attach_session` | verified (pre-audit) |
| Picker data source today | Face views / shell foreign | unverified — needs path |
| Session on-disk format | `~/.next-code/sessions` | unverified — needs schema |

## Copy / wire / delete
| Action | What |
|--------|------|
| **Wire** | Picker ← next-code sessions |
| **Wire** | CLI `--resume` bare → Face picker when TTY |
| **Delete** | Fake foreign entries |

## Implementation steps
1. [ ] Field map `SessionPickerEntry` ← meta.  
2. [ ] List + open (+ delete if supported).  
3. [ ] CLI bare resume.  
4. [ ] Tests + manual 2 sessions.

## Manual verify
Welcome shows sessions; open loads transcript; bare `--resume` works in WT.

## Open questions
1. Rename/delete: daemon API exists?  
2. Share/cloud rows: hide always?

## Out of scope
Cloud sync; rewriting Face picker visuals.

## Done when
Session discovery is next-code-native in Face UI.
