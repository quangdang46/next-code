# LOOK — Face vs reference repos (best-of matrix)

**Date:** 2026-07-29  
**Branch:** `feat/face-session-diff`  
**Base:** `review` @ 133089ce8 (PRs #112/#124–#133 merged)  
**Scope:** Finish deferred session `/diff` (Claude DiffDialog) after chrome ports landed.

**Sibling skip (do not duplicate):**
- Statusline Claude interactivity — **#133 merged**
- BoN / hashline / permission cards — already on `review`

**Merged on `review` (selected):**
| PR | Theme | Status |
|----|-------|--------|
| #112 | Permission cards + plan gate + DCG AcceptEdits/AlwaysApprove | **Done** |
| #124–#127 | Diff hunks, AcceptEdits chrome, sticky, connect polish | **Done** |
| #128–#131 | BoN Codebuff cards, multitask, mermaid sidebar, agent teams | **Done** |
| #132 | Memory edit + keybindings / `@` picker polish | **Done** |
| #133 | Statusline Claude parity (`@agent` / tasks / expand) | **Done** — do not re-implement |

---

## Comparison matrix

| Feature | Best reference | next-code Face (`review` + this branch) | Priority | Wire approach |
|---------|----------------|---------------------------------------|----------|---------------|
| Statusline / footer / bg tasks | Claude `BuiltinStatusLine` + Crush/OpenCode footers | **Has** — #133 merged | — | Done |
| Best-of-N / candidates | **Codebuff** ImplementorCard + progress | **Has** — #128 masonry/cards/± bars | — | Done |
| Edit/diff / hashline | **oh-my-pi** hashline + Claude DiffDialog + grok edit blocks | **Has** (this PR) — `/diff` DiffReview modal over git HEAD + turn scrollback; hashline Diff on `review` | — | Done |
| Connect / auth / first-message | **OpenCode** provider dialog | **Has** — #130 connect paths | — | Done |
| Sticky prompt / chrome | **Claude** StickyPrompt | **Has** — PR #107 | — | Done |
| Agent teams / multitask / bg agents | **Claude** teams + Cursor multitask plan | **Has** — team panel + multitask spawn/`--to` + statusline pills (#126/#133) | — | Done |
| Permissions / plan mode | **Claude** cycle + cards; **Codex** sandbox | **Has** — #112 + AcceptEdits chrome #125 | — | Done |
| Memory / @picker / keybindings | Claude memdir; Face `@` stronger than Codebuff files | **Has** — #132 memory edit; delete = double-press confirm | P2 | AskUser-on-delete optional polish |
| Submit / session lifecycle / DX | OpenCode connecting states; grok-build toasts | **Has** — welcome/connect fixes | P1 | Polish |
| Tools facade / LSP / tasks graph | Claude Agent + Task* APIs | **Has** — #111 / #105 / #108 on `review` | — | Done |

### Per-repo unique strengths (portable only)

| Repo | Steal | Skip / non-portable |
|------|-------|---------------------|
| **grok-build** | Face chrome patterns, ACP pager, mode banners | Branding, soft-default YOLO |
| **claude-code** | AcceptEdits cycle, DiffDialog, permission cards, sticky, statusline | Buddy sprites, remote/mobile, Claude Desktop |
| **opencode** | Connect/provider UX, footer density, which-key | TS-in-process hooks rewrite |
| **codex** | Approval overlay, effort statusline, sandbox presets | Seatbelt/landlock OS policy as Face UI |
| **crush** | Compact agent footer / task readiness | Go TUI clone |
| **codebuff** | BoN implementor cards, propose_* drafts, `@Agent` | Full CLI masonry; ReviewScreen ≠ BoN |
| **gajae-code** | pi-shell / hashline crates adjacent | Product fork specifics |
| **oh-my-pi** | Hashline edit model, TUI editor UX | Bazel monorepo glue |
| **oh-my-openagent** | Hashline-core, LSP daemon patterns, Claude-compat modes | Plugin swarm surface |

---

## Prioritized backlog

1. ~~**P0 — Session `/diff` review**~~ — **this PR** (`/diff` → `ActiveModal::DiffReview`).
2. ~~**P0 — AcceptEdits / BoN / hashline / permission cards / statusline**~~ — on `review` (#112/#124–#133).
3. **P2 — Memory delete via AskUserQuestion** — Claude uses AskUser; Face uses double-press confirm (intentional simpler UX unless product wants parity).
4. **P2 — `@Agent` mentions** — Codebuff; optional.
5. **P2 — Alt+T thinking toggle** — Claude muscle memory; Face has `/effort`.

---

## Explicit non-copies

- Do not pixel-clone Codebuff / Claude Ink.
- Do not replace DCG policy with Grok-only modes.
- Do not re-implement BoN/hashline/permission-cards/statusline while siblings or #133 own those surfaces.
