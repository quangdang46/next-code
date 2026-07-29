# LOOK — Face vs reference repos (best-of matrix)

**Date:** 2026-07-29  
**Branch:** `feat/face-session-diff` (session `/diff` in progress)  
**Base:** `review` @ 133089ce8 (PRs #124–#133 merged)  
**Scope:** Research compare of `.tmp/*` + open PRs on `review`; sequential P0 Face chrome.

**Sibling skip (do not duplicate):**
- `feat/face-bon-codebuff-parity` — Codebuff BoN cards / `mode=show`
- `feat/face-hashline-diff` — ACP Diff content for edit/hashline tools
- `feat/face-complete-gaps` — **idle** (no active work)
- PR #112 (`pr-face-permission-full`) — permission cards + plan gate + DCG AlwaysApprove wire

**Merged on `review` (#124–#133, selected):**
| PR | Theme | Status |
|----|-------|--------|
| #124–#127 | Permission cards, plan gate, sticky, connect polish | **Done** on `review` |
| #128–#131 | BoN baseline, multitask docs, tools facade, agent teams panel | **Done** on `review` |
| #132 | Memory / keybindings / `@` picker polish | **Done** on `review` |
| #133 | Statusline Claude parity (`8426c1ffe`) | **Done** on `review` — do not re-implement |

---

## Comparison matrix

| Feature | Best reference | next-code Face (`review` + this branch) | Priority | Wire approach |
|---------|----------------|---------------------------------------|----------|---------------|
| Statusline / footer / bg tasks | Claude `BuiltinStatusLine` + Crush/OpenCode footers | **Has** — #133 merged | — | Done |
| Best-of-N / candidates | **Codebuff** ImplementorCard + progress | **Partial** — PR #101; Codebuff chrome in sibling | P0 | Sibling: `feat/face-bon-codebuff-parity` |
| Edit/diff / hashline | **oh-my-pi** hashline + Claude DiffDialog + grok edit blocks | **In progress** — hashline ACP in sibling; **session `/diff` on `feat/face-session-diff`** | P0 | This branch: `/diff` DiffDialog; sibling hashline |
| Connect / auth / first-message | **OpenCode** provider dialog | **Has** — connect paste + welcome submit fixes on `review` | P1 | Polish only |
| Sticky prompt / chrome | **Claude** StickyPrompt | **Has** — PR #107 | — | Done |
| Agent teams / multitask / bg agents | **Claude** teams + Cursor multitask plan | **Partial** — team panel #91; multitask docs #82 | P1 | Integrate #82 later |
| Permissions / plan mode | **Claude** cycle + cards; **Codex** sandbox | **Partial** — #112 cards/gate | P0 | #112 into `review` |
| Memory / @picker / keybindings | Claude memdir; Face `@` stronger than Codebuff files | **Has** — memory #102, keybindings #93 | P2 | `@Agent` optional |
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

1. **P0 — Session `/diff` review** (**this branch**, in progress) — Claude `DiffDialog` over git HEAD + scrollback turn edits.
2. **P0 — AcceptEdits Face chrome** — Shift+Tab / settings / prompt flag (`feat/face-best-of-repos`).
3. **P0 — BoN Codebuff parity** — sibling worktree.
4. **P0 — Hashline Diff ACP** — sibling worktree.
5. **P0 — Permission cards + plan gate** — PR #112 into `review`.
6. **P1 — Multitask MVP** — docs #82 → chrome.
7. **P2 — `@Agent` mentions** — Codebuff; optional.
8. **P2 — Alt+T thinking toggle** — Claude muscle memory; Face has `/effort`.

---

## Explicit non-copies

- Do not pixel-clone Codebuff / Claude Ink.
- Do not replace DCG policy with Grok-only modes.
- Do not re-implement BoN/hashline/permission-cards/statusline while siblings or #133 own those surfaces.
