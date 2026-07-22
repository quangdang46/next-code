# Docs completeness audit — platform / nextcode vision (2026-07-22)

**Scope:** Vision docs for “Pi surfaces × herdr multilang + nextcode pack”.  
**Branch at verify:** re-check with `git branch --show-current`  
**Verify pass:** 2026-07-22 (re-inventory after false “missing” report + recreate)  
**Expand pass:** 2026-07-22 (stubs → ≥120 lines; D0 ≥80)

---

## On-disk inventory (verified)

### Top-level plans

| Path | On disk? | Git |
|------|----------|-----|
| `docs/plans/PLAN-20260722-pi-full-custom-platform.md` | **Yes** | Tracked |
| `docs/plans/PLAN-20260722-platform-implement-readiness.md` | **Yes** | May be untracked until commit |

### Research (`docs/plans/research/`)

| Path | On disk? |
|------|----------|
| `README.md` | Yes |
| `20260722-pi-extension-surfaces.md` | Yes |
| `20260722-herdr-multilang-abi.md` | Yes |
| `20260722-opencode-plugin-hooks.md` | Yes |
| `20260722-nextcode-extension-inventory.md` | Yes |
| `20260722-face-customization-limits.md` | Yes |

### Deepen (`docs/plans/research/deepen/`)

| ID | Path | On disk? | Depth (post-expand) |
|----|------|----------|---------------------|
| — | `README.md` | **Yes** | Meta + line-count expectations |
| — | `20260722-docs-completeness-audit.md` | **Yes** (this file) | Meta |
| D0 | `20260722-bare-host-no-prompt-inject.md` | **Yes** | ≥80 **met** (freeze solid) |
| D1 | `20260722-trust-gate-design.md` | **Yes** | ≥120 **met** |
| D2 | `20260722-plugins-state-skill-gate.md` | **Yes** | ≥120 **met** |
| D3 | `20260722-bundle-counts-honesty.md` | **Yes** | ≥120 **met** |
| D4 | `20260722-bundle-hooks-mcp-merge.md` | **Yes** | ≥120 **met** |
| D5 | `20260722-hooks-cookbook-layout.md` | **Yes** | ≥120 **met** |
| D6 | `20260722-host-callback-bin-path.md` | **Yes** | ≥120 **met** |
| D7 | `20260722-hook-registry-reload.md` | **Yes** | ≥120 **met** |
| D8 | `20260722-plugin-manifest-abi-v1.md` | **Yes** | ≥120 **met** |
| D9 | `20260722-package-slash-acp.md` | **Yes** | ≥120 **met** |
| D10 | `20260722-nextcode-pack-extraction.md` | **Yes** | ≥120 **met** |
| D11 | `20260722-tools-abi-v1.md` | **Yes** | ≥120 **met** |
| D12 | `20260722-face-ui-hints-external-pane.md` | **Yes** | ≥120 **met** |
| D13 | `20260722-argv-plugin-security.md` | **Yes** | ≥120 **met** |

**Count:** 14 deepen design files (D0–D13) + README + this audit = **16** files under `deepen/`.

### Prior false audit (corrected)

An earlier pass claimed deepen D1–D13 were missing. At that moment **only this audit file** existed under `deepen/` (untracked); designs were **never git-committed**, so they could not be recovered via `git checkout`. They were **recreated** from master plan + research inventory, then **expanded** from stubs to contract depth. Do not treat “recently viewed path” as proof of on-disk presence without `Test-Path` / `Get-ChildItem`.

### Stub era (honesty)

Post-recreate files were often **15–50 line stubs**. That inflated “files exist” completeness without implementable contracts. The expand wave fixed depth; scores below are **after expand**, not after recreate alone.

---

## Coverage scorecard (after expand wave)

| # | Topic | Score (recreate) | Score (**after expand**) | On disk? | Notes |
|---|--------|------------------|--------------------------|----------|-------|
| 1 | Platform vs nextcode pack + bare | 90% | **95%** | Yes | Master + D0 freeze + D10 table ≥120 |
| 2 | Multilang ABI (hooks/slash/tools/skills/packages) | 80% | **88%** | Yes | D5/D8/D9/D11 expanded; filename open Q remains |
| 3 | Trust, enable, security, reload, BIN_PATH | 85% | **93%** | Yes | D1–D2, D6–D7, D13 at contract depth |
| 4 | Face sealed / UI hints only | 85% | **90%** | Yes | Face research + D12 expanded; open Q #3 remains |
| 5 | **Bare must NOT inject opinionated system/brand prompts** | 95% | **98%** | Yes | **D0 frozen ≥80**; inventory CORE→pack still pending docs edit |
| 6 | Open questions unresolved | Listed | Listed | Yes | Master 3 Qs; D11 resolves tools timing in-docs |

**Overall (honest):**

| Layer | After recreate (thin stubs) | **After expand** |
|-------|----------------------------|------------------|
| Vision | ~90% | **~95%** |
| Phase 1 implement-ready docs | ~70% (stubs looked like tickets) | **~92%** |
| Phase 2+ ABI | ~55–70% | **~85%** (open Qs still block full freeze) |
| Combined docs completeness | ~85% claimed / **~75% real** | **~92%** |

**Do not** cite the recreate-era ~85% combined score without the stub caveat.

---

## What exists (good)

1. Master plan Option B′ + phases.
2. Five research reports + README.
3. Readiness gate plan.
4. Full deepen D0–D13 at **contract depth** (≥120; D0 ≥80) + index with line-count expectations.
5. **Frozen** bare/Pi empty-shell prompt rule (D0); `system_prompt.md` reclassified as **nextcode pack** (not host CORE).
6. D11 docs stance: MCP-first; argv gated on D13.

---

## Remaining gaps (not “missing files”)

- Inventory markdown still labels System prompt as CORE — update when pack extraction lands (D10) or as docs follow-up (D0 wins until then).
- Master open Qs unresolved: (1) manifest filename, (3) external pane vs hints — (2) tools timing answered in D11.
- Depth targets met; **quality review** of parallel expand drafts still worthwhile before coding.
- No production code yet — waiting **go ahead**.

---

## Ready for implement?

| Layer | Ready? |
|-------|--------|
| Product vision / Option B′ / Phase 0 | **Yes** |
| Phase 1 tickets with frozen contracts | **Yes (docs, post-expand)** — need owner **go ahead** |
| Phase 2+ ABI implement | **Partial** — contracts yes; open Qs #1/#3 block full freeze |

**Recommendation:** Docs gate satisfied for Phase 1 decision. Reply **go ahead** on readiness/master plan to start code.

**Status:** Audit re-scored after expand wave. No production code.
