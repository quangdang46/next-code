# Bug — bare `--resume` stacks ResumeBrowser on SessionPicker Loading

## Summary
- **Symptom:** `next-code --resume` showed Face **2-panel ResumeBrowser** and, underneath / after Esc, a **SessionPicker-style Loading shell with `[✗]`** (welcome expand-card or modal). Closing one revealed the other.
- **Not the bug:** Dual entry itself (`--resume` 2-panel vs `/resume` modal) remains intentional.
- **Root cause:** `ShowResumeBrowser` → `FetchSessionList` armed welcome `session_picker_loading = true`. `SessionListLoaded` preferred `resume_browser` and `sessions.take()`’d the list, then returned **without** clearing welcome loading. Esc `CloseResumeBrowser` revealed stuck Loading + `[✗]`. Opening ResumeBrowser also did not dismiss an open `ActiveModal::SessionPicker`.
- **Fix:** On `ShowResumeBrowser`, dismiss SessionPicker modal + welcome picker fields and invalidate in-flight picker fetches; do not arm welcome loading while ResumeBrowser owns the list; clear loading when list is consumed / on close; `ShowSessionPicker` closes ResumeBrowser (mutual exclusion).
- **Status:** Fixed on `pr-face-resume-dual-entry`

## Smoke
- [ ] `next-code --resume` → only 2-panel; Esc → clean exit/welcome (no Loading `[✗]` flash)
- [ ] In-session `/resume` / Ctrl+S → modal only
