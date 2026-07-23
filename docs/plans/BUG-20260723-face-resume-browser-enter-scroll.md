# Plan Report

## Summary (read this first)
- **You asked:** Face `--resume` 2-panel: Enter resume / scroll empty; screenshot shows **origin** Preview loads full tool rows (`✓ read … · 1.1k tok`).
- **What is going on:** Face `load_transcript_preview` only keeps `content[].type == "text"`. Real next-code sessions store tool turns as `tool_use` on assistant + `tool_result` on user — often **zero text**. Those turns are dropped → thin/empty Preview. Enter→`dispatch_load_session` already matches SessionPicker (not the thin-preview bug).
- **We recommend:** Expand shell preview walker to emit tool fold lines (status + name + path/summary + approx tok), matching origin density; keep Enter on the shared `LoadSession` path; add regression tests.
- **Risk:** Low
- **Status:** Implementing (user bug report = go-ahead)

## Bug investigation
- **Verified root cause:** `xai-grok-shell::session::persistence::message_to_preview_line` / `extract_message_text` ignore `tool_use` / `tool_result` → bonehound-like sessions preview as empty or “2” only. Origin uses `session::render_messages` → `role: "tool"` + `tool_data` → TUI `render_tool_message` (`✓ read path · N tok`).
- **Hypotheses ranked:**
  1. Preview FS loader text-only (confirmed vs bonehound JSON)
  2. Enter bypasses LoadSession (ruled out — `dispatch_pick_resume_browser_session` mirrors pick → `dispatch_load_session`)
  3. ACP history replay broken only from ResumeBrowser (unverified as primary; same Effect as `/resume` pick)
- **Sub-agents used:** skipped (narrow, citation-ready)
- **Citations checked:**
  - Face: `persistence.rs` `load_transcript_preview` / `extract_message_text`
  - Origin: `next-code-tui/.../loading.rs` `build_messages_preview` → `render_messages`; `session_picker.rs` `"tool" => render_tool_message`
  - Live fixture: `~/.next-code/sessions/session_bonehound_*.json` — assistant `tool_use` read AGENTS.md / SPEC; user `tool_result` only

## Evidence
1. **Our Face loader:** text-only filter — verified
2. **Origin preview:** full `RenderedMessage` incl. tools — verified
3. **Enter wire:** `PickResumeBrowserSession` → `dispatch_pick_resume_browser_session` → `dispatch_load_session` — verified same family as `PickSession`
4. **Screenshot expectation:** origin Preview tool rows with tok badges

## Steps
1. [x] Rewrite preview walker to pair tool_use → tool_result fold lines
2. [x] Style Face preview `tool` role (no noisy `tool:` prefix)
3. [x] Tests: bonehound-shaped fixture emits `✓ read … · … tok`; Enter emits `LoadSession`
4. [ ] `cargo test` targeted + commit/push #67 + install if feasible

## Files to touch
- `crates/xai-grok-shell/src/session/persistence.rs`
- `crates/xai-grok-pager/src/views/resume_browser.rs` (render tool lines)
- `crates/xai-grok-pager/src/app/dispatch/tests/resume_browser.rs`
- this BUG plan

## Ruled out
- Stubborn Enter-only “close browser” without load (code path calls load)
- Needing `NEXT_CODE_LEGACY_TUI` for tool preview
