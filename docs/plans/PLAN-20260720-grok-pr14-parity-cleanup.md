# Plan Report — PR14 Parity cleanup / close migration

## Summary (read this first)
- **You asked:** Final PR to close the Grok UI migration goal.
- **What is going on:** After PR9–13, leftovers: unused stubs, brand strings, docs drift, optional Face features still stubbed, CI weight from dead crates.
- **We recommend:** Cleanup-only PR — no new architecture. Document remaining stubs as permanent “won’t do” (voice, marketplace) or file follow-ups outside migration.
- **Risk:** Low–Medium
- **Status:** Last PR of the series.

## Goal for this PR
Migration marked **DONE** in SUMMARY; repo teaches Face-first; dead code/crates trimmed; skill + docs consistent.

## Checklist
1. [ ] `docs/grok-migration-SUMMARY.md` — PR1–14 table, status DONE, GrokHost marked abandoned/deferred forever.
2. [ ] Remove or feature-gate unused stub crates Face never imports.
3. [ ] rg for user-facing `grok` strings in quit/hints/titles under next-code embed paths; fix or document.
4. [ ] Confirm `grok-migration-workflow` skill still accurate; update `reference.md` with PR9–13 lessons.
5. [ ] Issue #35 (or tracker): close with link to this PR.
6. [ ] Manual smoke: cold start, tool turn, settings, resume picker, quit hint, logo — 10 min script.

## Copy / wire / delete
| Action | What |
|--------|------|
| **Delete** | Dead stubs / leftover `next-code-tui*` if PR11 left them |
| **Wire** | None required unless smoke finds a hole → bounce to hotfix PR |

## Explicit won’t-do (write into SUMMARY)
- Voice / STT
- GrokHost rewrite
- Full grok.com foreign session sync
- Re-adding legacy TUI

## Done when
SUMMARY says migration complete; smoke script green; no open “must for Face cutover” items.
