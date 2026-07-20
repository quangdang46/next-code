---
name: grok-migration-workflow
description: >-
  Discipline for replacing next-code UI with Grok Face (xai-grok-pager) via copy,
  delete old UI, and wire into next-code — never invent Face behavior. Use when
  migrating TUI/Face, PR8 cutover, wiring xai-grok-pager, removing next-code-tui,
  adapting ratatui 0.28 shims, quit/resume/auth wire bugs, or comparing grok-build
  vs next-code embed. Triggers on: grok migration, Face cutover, wire Face,
  copy grok UI, replace next-code TUI, grok-build research, SharedTermWriter,
  pager_launch, NextCodeFaceAgent.
---

# Grok Migration Workflow

**Goal:** Replace next-code interactive UI with Grok Face by **copy → delete old → wire**, and prefer **Grok logic** over weaker next-code UI logic. Face stays presentation; next-code daemon stays auth/brain.

**North star:** Match stock Grok behavior unless a deliberate next-code product difference is documented and approved.

## Hard rules (never violate)

1. **LOOK → PLAN → BUILD.** No production edits until a Plan Report exists and the user says **go ahead** / **approved** / **implement** (tiny typos exempt only if said so).
2. **No auto-fix on hunches.** Bugs → root-cause first. Stubborn patches (extra sleeps, dual Leave, `no_alt_screen`, “just skip drain”) are forbidden until the hang/seam is proven.
3. **No lying.** Every load-bearing claim is `verified` (path/symbol/line/doc) or `unverified — needs X`. Never invent grok-build APIs, wire formats, or “Grok does X” from memory.
4. **Copy / wire / remove — do not rewrite Face.** Prefer vendoring upstream Face and adapting at the **seam**. Do not reimplement Face features in next-code-tui style.
5. **Prefer Grok over inferior next-code UI.** When stock Grok and next-code disagree on UI/UX lifecycle (quit hint, screen restore, writer drain), default to Grok’s approach; only diverge for product branding (`nextcode` vs `grok`) or daemon ACP.
6. **Adaptation shims are first-class suspects.** ratatui 0.28 / missing `writer_mut` / `SharedTermWriter` / TLS activate-without-deactivate are migration bugs, not “Windows is weird.”

## Mandatory sequence

```
Research (grok-build + our tree) → Plan Report → User OK → Implement → Prove → Rebuild/install both aliases
```

### 1) Research (LOOK)

Sources **in order** — skip only if irrelevant:

| Order | Source | Why |
|------:|--------|-----|
| 1 | **DeepWiki** `xai-org/grok-build` | Stock Face composition root, quit/restore, ACP |
| 2 | **Exa** / GitHub issues on grok-build / agents-js only if needed | Prior art, regressions |
| 3 | **Vendored Face in this repo** `crates/xai-grok-pager*`, `xai-grok-pager-render` | What we actually run |
| 4 | **Wire seam** `src/cli/pager_launch.rs`, `pager_agent.rs`, `dispatch.rs` | Embed deltas vs `grok` bin |
| 5 | **A/B evidence** | Same WT tab: official `grok` vs `nextcode` when claim is “Grok OK, nextcode not” |

**Always ask:** What does **stock grok-bin** do before `app::run` / after quit that our embed skips? (stderr redirect, crash handler, argv0 branding, agent factory.)

### 2) Plan Report (PLAN)

Write `docs/plans/PLAN-YYYYMMDD-<slug>.md` or `BUG-YYYYMMDD-<slug>.md`.

Chat: **path + 1–3 sentences only** — do not paste the full report.

Required sections:

- Summary (plain English) + Risk + Status waiting for OK
- Evidence with verified citations
- Files to touch
- Explicit **copy / delete / wire** map
- Open questions (≤3 blocking)

Bug reports additionally:

- Verified root cause **or** `unverified — needs …`
- Ranked hypotheses
- What was **ruled out** (auth wire, dual-Leave, etc.)

### 3) Implement (BUILD) — only after OK

Allowed shapes:

| Kind | Do |
|------|----|
| **Copy** | Vendor Face crates / files from grok-build; keep NOTICE; pin crates.io where required |
| **Delete** | Remove or gate legacy `next-code-tui` paths (`NEXT_CODE_LEGACY_TUI` escape only) |
| **Wire** | Thin `pager_launch` + ACP factory → daemon; map `--resume`; brand argv0 in hints |
| **Shim** | Minimal ratatui 0.28 adapters; every shim needs deactivate/teardown parity with upstream |

Forbidden shapes:

- Re-homing Face features into `next-code-tui`
- “Temporary” quit sprays that paper over writer/TLS hangs
- Hardcoding `grok` in user-facing quit/resume when argv0 is `nextcode` / `next-code`
- Claiming wire mismatch without comparing stock vs embed on the **same** code path

### 4) Prove

- Unit/regression for the seam you fixed
- Operator repro when TTY-involved (quit, resume hint)
- Install **both** `next-code` and `nextcode` aliases (Windows: `%LOCALAPPDATA%\next-code\bin\`)
- Diag logs only when gated (`NEXT_CODE_FACE_QUIT_DIAG=1`) — never spam quit tail by default

## Copy → delete → wire checklist

```
- [ ] Identified stock grok-build symbol/path for the behavior
- [ ] Identified next-code seam (launch / ACP / branding / shim)
- [ ] Plan states what is copied vs adapted vs deleted
- [ ] Old UI path removed or explicitly legacy-gated
- [ ] Embed does not skip a stock teardown/init step without documented reason
- [ ] User-facing strings use nextcode/next-code when embedded
- [ ] Daemon/auth stay on next-code; Face does not grow a second brain
```

## Bug hunt (Face / wire)

When user reports black screen, hung quit, wrong resume brand, auth loop:

1. Reproduce + capture durable evidence (diag log, process still alive?, A/B grok).
2. Diff **stock restore/quit** vs **embed** — prefer TLS/`Sender` lifetime, drain/join, Leave order over “Windows ConPTY.”
3. Write `BUG-*.md` with verified root cause.
4. **Wait for go ahead.**
5. Smallest fix at the proven seam; rebuild; confirm Leave / process exit / hint text.

Known migration pitfall (lesson): `SharedTermWriter::activate` TLS clone kept writer `Sender` alive → unbounded `join` after alt clear → black screen. Fix = deactivate before drop/join (+ bounded join safety). Prefer proving this class of bug over quitting patches.

## Branding & product deltas (allowed)

| Keep next-code | Prefer Grok |
|----------------|-------------|
| Daemon / providers / sessions store | Face event loop, restore order, writer model |
| `nextcode --resume <id>` hint text | Quit lifecycle (Leave after drain) |
| ACP `NextCodeFaceAgent` | Composition patterns from grok-bin |

Document every intentional delta in the Plan Report.

## Research tools cheat-sheet

- DeepWiki: `ask_question` / wiki on `xai-org/grok-build`
- Exa: prior art only after DeepWiki + local tree
- Local: `crates/xai-grok-pager/**`, `src/cli/pager_launch.rs`, `pager_agent.rs`
- Rebuild helper (Windows): `scripts/_tmp_rebuild_install.ps1` (must update **both** aliases)

## Anti-patterns (fail the review)

- Implement before Plan + OK
- “Fixed” by skipping Leave / clearing alt without drain proof
- Inventing grok-build behavior
- Keeping dual UIs without an explicit escape hatch
- Quit diag / debug eprintln left on by default
- Resume hint still says `grok` under `nextcode`
- Replaying last error/prompt on quit when product asked for clean resume-only tail

## Relation to other skills

- **feature-planning** — general feature research; this skill **overrides** for Face/Grok UI migration specifics.
- **origin-sync** — upstream fork sync; do not confuse with Face vendor copy from grok-build.
- Repo **AGENTS.md** LOOK→PLAN→BUILD still applies; this skill specializes it for the migration goal.

## Done means

- Behavior matches approved Plan
- Root cause cited for any bug fix
- Tests/repro green
- Both CLI aliases installed when UI changed
- Chat summary short; Plan/BUG file holds the detail
