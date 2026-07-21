# Plan Report — PR12 Stub → real (shell / workspace Face hits)

## Summary (read this first)
- **You asked:** Real behavior behind Face-hit stubs.
- **What is going on:** PR3–6 stubs no-op; Face looks broken on trust/git/sessions chrome.
- **We recommend:** **Copy signature** (keep Face API). **Wire** body to next-code or **copy pure helper** from grok-build. Do **not** wholesale-vendor shell. Prefer Grok Face call sites over inventing next-code-tui chrome.
- **Risk:** Medium  
- **Status:** After PR9; can overlap late PR10.

## Workflow map (required)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | Pure grok-build helpers when next-code lacks equivalent | Vendor entire `xai-grok-shell` upstream tree |
| **Wire** | Stub → next-code sessions/git/trust | Silent `Ok(())` for P0 call sites |
| **Delete** | Only if Face never imports (else PR14) | Delete Face UI that calls the stub |

## Research first (LOOK)
1. rg Face → `xai_grok_shell::` frequency report → paste into Evidence.  
2. grok-build real body for P0 symbols.  
3. next-code equivalents.

## Evidence (fill before BUILD)

| Stub | Face callers (count) | grok-build source | next-code target | Status |
|------|---------------------|-------------------|------------------|--------|
| `active_sessions` | | | | unverified |
| folder trust / project dir | | | | unverified |
| session persistence | | | | unverified |
| git chrome | | | | unverified |

## Priority
P0: active_sessions, trust/project-dir, session persistence helpers.  
P1: git info, clipboard verify.  
P2: plugins/share — keep stub or hide UI (coordinate PR10 hide).

## Implementation steps
1. [ ] Frequency report in Evidence.  
2. [ ] Implement P0 with tests.  
3. [ ] Manual trust / session files / git hint.  
4. [ ] No new voice/GCS network.

## Manual verify
Real cwd/git/trust; no pretend success on P0.

## Open questions
1. Layering: shell → app-core forbidden — use composition-root callbacks?  
2. Hide P2 UI in PR10 vs leave stub?

## Out of scope
GrokHost, voice, full marketplace, TUI delete.

## Done when
P0 real; P2 listed in SUMMARY as stub.
