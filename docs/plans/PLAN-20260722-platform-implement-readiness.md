# Plan Report — Platform implement readiness (docs gate)

## Summary (read this first)

- **You asked:** A gate doc before Phase 1 code for Option B′ (Pi surfaces × herdr ABI + nextcode pack).
- **What is going on:** Phase 0 research + master plan exist; deepen D0–D13 on disk at **contract depth** (expand + finish wave 2026-07-22: D0 ≥80, D1–D13 ≥120). Criterion 5 (bare no prompt inject) is **frozen** — `system_prompt.md` is **nextcode pack**, not host CORE. Tools open Q #2 frozen **MCP-first** (argv gated on D13).
- **We recommend:** Treat this file as the **implement readiness checklist**. Phase 1 code only after owner **go ahead**. Smallest slice = trust + enable-gate + honest counts/merge + 3-lang cookbook.
- **Risk:** Medium (trust/RCE); Low for docs-only gate.
- **Status:** Docs ready for Phase 1 decision — reply **go ahead** to implement Phase 1. **No production Rust in this report.**

---

## Inputs (must exist)

| Artifact | Path | Role |
|----------|------|------|
| Master plan | [`PLAN-20260722-pi-full-custom-platform.md`](./PLAN-20260722-pi-full-custom-platform.md) | Vision + phases |
| Research ×5 | [`research/README.md`](./research/README.md) | Evidence |
| Deepen index | [`research/deepen/README.md`](./research/deepen/README.md) | D0–D13 contracts + **final line-count table** |
| Completeness audit | [`research/deepen/20260722-docs-completeness-audit.md`](./research/deepen/20260722-docs-completeness-audit.md) | Scorecard + contradiction freeze |

---

## Deepen final line counts (finish pass)

| ID | Lines | Target | Notes |
|----|------:|-------:|-------|
| D0 | 154 | ≥80 | Frozen bare rule |
| D1 | 339 | ≥120 | Trust |
| D2 | 307 | ≥120 | Enable → skills |
| D3 | 316 | ≥120 | Counts honesty (parallel wave OK) |
| D4 | 423 | ≥120 | Hooks/MCP merge (parallel wave OK) |
| D5 | 442 | ≥120 | 3-lang cookbook (parallel wave OK) |
| D6 | 314 | ≥120 | `NEXT_CODE_BIN_PATH` |
| D7 | 215 | ≥120 | Reload; Face Reload = UI-only today |
| D8 | 152 | ≥120 | Manifest shape; filename open |
| D9 | 285 | ≥120 | Package slash ACP |
| D10 | 174 | ≥120 | Pack extraction + `prompt.rs` |
| D11 | 293 | ≥120 | Tools; **MCP-first** freeze |
| D12 | 141 | ≥120 | UI hints vs pane; Q#3 open |
| D13 | 174 | ≥120 | Argv security gate |

Recount: `@(Get-Content path).Count` under `docs/plans/research/deepen/`.

---

## Readiness scorecard (docs)

| Gate | Ready? | Notes |
|------|--------|-------|
| Option B′ chosen | Yes | Master plan |
| Platform vs nextcode pack | Yes | + bare-host freeze (D0) |
| Phase 1 ticket designs | Yes | D1–D7 at ≥120 lines (post-finish) |
| Phase 2+ ABI sketches | Yes | D8–D13 ≥120; open Qs #1/#3 remain |
| Bare no opinionated prompts | **Frozen** | D0 ≥80; inventory CORE→pack reclass pending |
| Tools timing (open Q #2) | **Frozen MCP-first** | D11 + audit; argv after D13 |
| Stub depth fixed | **Yes** | Expand + finish; see deepen README table |
| Owner implement OK | **Waiting** | Reply go ahead |

**Docs completeness (see audit, after finish):** vision ~95% · Phase 1 contracts ~93% · Phase 2+ ABI ~86% (open Qs #1/#3). Recreate-era ~85% combined was **overstated** while files were stubs.

---

## Phase 1 build order (when approved)

1. Trust gate (D1)
2. plugins-state → skill ingest (D2)
3. Bundle counts honesty and/or hooks+MCP merge (D3–D4)
4. Cookbook Bash/Python/Node (D5)
5. BIN_PATH if in slice (D6); reload hooks as needed (D7 — close Face UI-only reload gap)
6. Product profile sketch started (D10) — at least respect D0 for prompt load path when touching prompts

## Explicitly not Phase 1

- Face plugin-host / custom TUI
- Full `next-code-plugin.toml` marketplace
- `[[tools]]` argv as day-one default (Phase 3 / D11) — **MCP-first**; argv after D13
- Resolving master open Qs #1 (manifest name) and #3 (external pane)

---

## Verification when coding starts

- `cargo check` / targeted tests for hooks + skill load
- Manual: disable plugin → skills gone; untrusted project → no exe hooks
- Cookbook deny path exit 2
- Bare profile: no baked nextcode persona from `system_prompt.md` (D0)
- Face Hooks Reload: live `hook_registry` matches disk (D7)

---

## Status

**Waiting for your OK** — reply **go ahead** to implement Phase 1 only.
