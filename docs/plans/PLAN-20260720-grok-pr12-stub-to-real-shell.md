# Plan Report — PR12 Stub → real (shell / workspace Face hits)

## Summary (read this first)
- **You asked:** Replace empty Face dependency stubs with real behavior where the UI actually calls them.
- **What is going on:** PR3–6 shipped compile stubs. Face now runs; many `xai-grok-shell` / tools / workspace calls no-op (`Ok(())`, empty lists). That makes dashboard, trust, git, share, plugins look broken.
- **We recommend:** Priority by **Face call frequency** (rg), not wholesale vendor of grok-build’s 400+ shell files. Prefer implementing via next-code APIs (sessions dir, git, trust) behind the existing Face-facing function signatures (**copy signature, wire body**).
- **Risk:** Medium (crate layering; avoid shell→app-core cycles)
- **Status:** After PR9; can overlap late PR10.

## Goal for this PR
Top broken Face surfaces stop being silent no-ops: at least folder trust, active session register/unregister, basic git status/cwd helpers Face shows on welcome/agent chrome.

## Research first (LOOK)
1. From a Face session log / `rg` pager → `xai_grok_shell::` call sites; rank by path.
2. grok-build real implementations for those symbols (DeepWiki + upstream file).
3. next-code equivalents (session store under `~/.next-code/sessions`, git helpers).

## Priority table (fill during LOOK; starter list)

| Stub area | Likely file | Priority |
|-----------|-------------|----------|
| `active_sessions` register/unregister | `xai-grok-shell/src/active_sessions.rs` | P0 |
| folder trust / workspace classify | shell trust + `xai-file-utils` | P0 |
| session persistence helpers Face reads | `session/persistence` | P0 |
| git info for chrome | `session/git` / `git_info` | P1 |
| clipboard | already re-exports shared — verify | P1 |
| plugins install | keep stub / hide UI | P2 |
| share | keep stub | P2 |

## Copy / wire / delete
| Action | What |
|--------|------|
| **Wire** | Stub body → next-code or faithful port of pure grok helper |
| **Copy** | Pure functions from grok-build when next-code has no equivalent |
| **Delete** | Dead stub modules Face never imports (PR14 if unsure) |

## Implementation steps
1. [ ] Produce call-site frequency report (paste into this plan under Evidence).
2. [ ] Implement P0 stubs with tests.
3. [ ] Manual: trust prompt / workspace badge / session files update on disk.
4. [ ] Do not vendor entire upstream shell.

## Files (expected)
- `crates/xai-grok-shell/src/**` selective
- `crates/xai-grok-workspace/**`, `xai-file-utils` as needed
- Composition root registration if callbacks required

## Manual verify
1. Face shows real cwd/git hint if applicable.
2. Trust / project-dir behavior matches next-code expectations.
3. No new network calls from “voice” or GCS stubs.

## Out of scope
- Full plugin marketplace
- Voice audio
- GrokHost

## Done when
P0 Face-hit stubs are real; P2 remain stub and are listed in SUMMARY.
