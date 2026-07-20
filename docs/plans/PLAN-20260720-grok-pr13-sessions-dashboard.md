# Plan Report — PR13 Sessions dashboard / picker parity

## Summary (read this first)
- **You asked:** Face session UX should list/resume next-code sessions like product UI, not Grok-cloud sessions.
- **What is going on:** Resume by id works via CLI/`LoadSession`. Welcome/dashboard pickers may still talk to shell foreign-session stubs or empty lists. Users need `nextcode --resume` picker + in-UI session browser backed by `~/.next-code/sessions`.
- **We recommend:** Wire Face session picker / dashboard data source → next-code session store (same files legacy TUI used). Keep Face UI components; replace data adapters.
- **Risk:** Medium
- **Status:** After PR9 (resume attach already exists); better after PR12 persistence helpers.

## Goal for this PR
From Face welcome/dashboard: see recent next-code sessions, open one, delete/rename if Face supports it — all on next-code disk format.

## Research first (LOOK)
1. Face: `views/session_picker.rs`, welcome resume tips, dashboard.
2. next-code: session list used by `tui_launch::list_sessions`, session journal paths.
3. grok-build: picker expected fields (`SessionPickerEntry`).

## Copy / wire / delete
| Action | What |
|--------|------|
| **Wire** | Picker load → next-code sessions directory |
| **Wire** | Open → existing `load_session` / `ResumeSession` path |
| **Delete** | Fake/empty foreign session demos if any |

## Implementation steps
1. [ ] Map `SessionPickerEntry` fields ← next-code session meta (title, path, mtime, short_name).
2. [ ] Implement list + open; wire delete if UI has it and daemon supports.
3. [ ] CLI: `nextcode --resume` with no id should list or open Face picker (TTY required).
4. [ ] Tests for meta parsing; manual list of 2+ sessions.

## Files (expected)
- Face views + thin adapter (prefer `src/cli` or shell session module)
- Avoid duplicating journal format — reuse next-code session types

## Manual verify
1. Create 2 sessions → quit → welcome shows both.
2. Open from picker → transcript loads.
3. `nextcode --resume` in WT shows picker or list (not “requires interactive terminal” when TTY).

## Out of scope
- Cloud sync / foreign grok.com sessions

## Done when
Session discovery is next-code-native inside Face UI.
