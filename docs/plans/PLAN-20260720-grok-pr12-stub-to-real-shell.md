# Plan Report — PR12 Stub → real (shell / workspace Face hits)

## Summary (read this first)
- **You asked:** Real behavior behind Face-hit stubs.
- **What is going on:** PR3–6 stubs no-op; Face looks broken on trust/git/sessions chrome.
- **We recommend:** **Copy signature** (keep Face API). **Wire** body to next-code or **copy pure helper** from grok-build. Do **not** wholesale-vendor shell. Prefer Grok Face call sites over inventing next-code-tui chrome.
- **Risk:** Medium
- **Status:** **P0 implemented** (active_sessions, folder_trust persist, flat session persistence). P1 git/clipboard already real (verify-only). P2 plugins/share remain stub.

## Workflow map (required)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | Pure grok-build helpers when next-code lacks equivalent | Vendor entire `xai-grok-shell` upstream tree |
| **Wire** | Stub → next-code sessions/git/trust | Silent `Ok(())` for P0 call sites |
| **Delete** | Only if Face never imports (else PR14) | Delete Face UI that calls the stub |

## Evidence

| Stub | Face callers (count) | grok-build source | next-code target | Status |
|------|---------------------|-------------------|------------------|--------|
| `active_sessions` | effects, signal_handler, headless (~4 prod) | `xai-grok-shell/active_sessions.rs` JSON+flock | Pure FS under `grok_home()` | **wired** |
| folder trust / project dir | `--trust`, headless grant, lifecycle `TrustFolder` | `workspace::trust` + `folder_trust` | `trusted_folders.toml` + inert/release gate | **wired** |
| session persistence | session_startup, load, picker, effects | nested Grok sessions | flat `sessions/<id>.json` scan/map | **wired** |
| git chrome | welcome/status/`git_info.rs` | N/A (pager) | already `git2` + ACP git_status | **already real** |
| clipboard | pager-render / shared | `xai_grok_shared::clipboard` | shell re-export | **already real** |

## Priority
P0: active_sessions, trust/project-dir, session persistence helpers. ✅  
P1: git info, clipboard verify. ✅ (pre-existing)  
P2: plugins/share — keep stub or hide UI (coordinate PR10 hide). ⏸️ still stub

## Implementation steps
1. [x] Frequency report in Evidence.
2. [x] Implement P0 with tests.
3. [ ] Manual trust / session files / git hint.
4. [x] No new voice/GCS network.

## Manual verify
Real cwd/git/trust; no pretend success on P0.

## Open questions
1. Layering: shell → app-core forbidden — **resolved**: pure FS under `grok_home()`, no app-core dep.
2. Hide P2 UI in PR10 vs leave stub? — leave stub for now.

## Out of scope
GrokHost, voice, full marketplace, TUI delete.

## Done when
P0 real; P2 listed in SUMMARY as stub. ✅
