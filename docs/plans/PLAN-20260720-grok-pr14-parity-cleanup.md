# Plan Report — PR14 Parity cleanup / close migration

## Summary (read this first)
- **You asked:** Close migration cleanly.
- **What is going on:** After PR9–13, leftovers: stubs, brand strings, docs, dead crates.
- **We recommend:** **Delete**-heavy cleanup. No architecture. Prefer Grok Face already shipped; remove next-code dual paths and Grok brand leaks.
- **Risk:** Low–Medium  
- **Status:** Last PR.

## Workflow map (required)

| Kind | Do | Do not |
|------|----|--------|
| **Copy** | — | New features |
| **Wire** | Hotfix only if smoke fails | Big new adapters |
| **Delete** | Dead stubs/crates; leftover TUI; user-facing `grok` leaks in embed | Voice “someday” code left half-wired |

## Checklist
1. [ ] SUMMARY: PR1–14 DONE; GrokHost abandoned.  
2. [ ] Delete unused stub crates Face never imports.  
3. [ ] `rg` user-facing `grok` / grok.com under embed paths — fix or document intentional.  
4. [ ] Update `grok-migration-workflow` `reference.md` with PR9–13 lessons.  
5. [ ] Close tracker issue #35.  
6. [ ] 10‑min smoke: start, tool, settings, slash menu brand, sessions, quit hint, logo.

## Evidence (fill at end)

| Smoke item | Result | Status |
|------------|--------|--------|
| Cold start Face | | |
| Tool + permission | | |
| Settings persist | | |
| Slash no grok.com/gboom | | |
| Session picker | | |
| Quit `nextcode --resume` | | |

## Explicit won’t-do
Voice/STT; GrokHost; grok.com foreign sync; re-adding legacy TUI.

## Done when
SUMMARY = migration complete; smoke green; no open cutover musts.
