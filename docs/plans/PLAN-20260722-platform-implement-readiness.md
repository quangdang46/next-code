# Plan Report — Platform implement readiness (docs gate)

## Summary (read this first)

- **You asked:** A gate doc before Phase 1 code for Option B′ (Pi surfaces × herdr ABI + nextcode pack).
- **What is going on:** Phase 0 research + master plan exist; deepen D0–D13 now on disk; criterion 5 (bare no prompt inject) is **frozen**.
- **We recommend:** Treat this file as the **implement readiness checklist**. Phase 1 code only after owner **go ahead**. Smallest slice = trust + enable-gate + honest counts/merge + 3-lang cookbook.
- **Risk:** Medium (trust/RCE); Low for docs-only gate.
- **Status:** Docs ready for Phase 1 decision — reply **go ahead** to implement Phase 1. **No production Rust in this report.**

---

## Inputs (must exist)

| Artifact | Path | Role |
|----------|------|------|
| Master plan | [`PLAN-20260722-pi-full-custom-platform.md`](./PLAN-20260722-pi-full-custom-platform.md) | Vision + phases |
| Research ×5 | [`research/README.md`](./research/README.md) | Evidence |
| Deepen index | [`research/deepen/README.md`](./research/deepen/README.md) | D0–D13 contracts |
| Completeness audit | [`research/deepen/20260722-docs-completeness-audit.md`](./research/deepen/20260722-docs-completeness-audit.md) | Scorecard |

---

## Readiness scorecard (docs)

| Gate | Ready? | Notes |
|------|--------|-------|
| Option B′ chosen | Yes | Master plan |
| Platform vs nextcode pack | Yes | + bare-host freeze (D0) |
| Phase 1 ticket designs | Yes | D1–D7 |
| Phase 2+ ABI sketches | Yes | D8–D13 (open Qs remain) |
| Bare no opinionated prompts | **Frozen** | D0; inventory CORE→pack reclass |
| Owner implement OK | **Waiting** | Reply go ahead |

**Docs completeness (see audit):** vision ~90% · Phase 1 contracts ~85% · Phase 2+ ABI ~70% (open Qs).

---

## Phase 1 build order (when approved)

1. Trust gate (D1)
2. plugins-state → skill ingest (D2)
3. Bundle counts honesty and/or hooks+MCP merge (D3–D4)
4. Cookbook Bash/Python/Node (D5)
5. BIN_PATH if in slice (D6); reload hooks as needed (D7)
6. Product profile sketch started (D10) — at least respect D0 for prompt load path when touching prompts

## Explicitly not Phase 1

- Face plugin-host / custom TUI
- Full `next-code-plugin.toml` marketplace
- `[[tools]]` argv (Phase 3 / D11)
- Resolving all master open Qs (manifest name, tools timing, external pane)

---

## Verification when coding starts

- `cargo check` / targeted tests for hooks + skill load
- Manual: disable plugin → skills gone; untrusted project → no exe hooks
- Cookbook deny path exit 2

---

## Status

**Waiting for your OK** — reply **go ahead** to implement Phase 1 only.
