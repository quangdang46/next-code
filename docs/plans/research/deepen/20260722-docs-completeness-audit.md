# Docs completeness audit — platform / nextcode vision (2026-07-22)

**Scope:** Vision docs for “Pi surfaces × herdr multilang + nextcode pack”.  
**Branch at verify:** re-check with `git branch --show-current`  
**Verify pass:** 2026-07-22 (re-inventory after false “missing” report + recreate)  
**Expand pass:** 2026-07-22 (stubs → ≥120 lines; D0 ≥80)  
**Finish pass:** 2026-07-22 (D7/D10/D13 thickened with real-code cites; meta tables refreshed)

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

### Deepen (`docs/plans/research/deepen/`) — final line counts

Counts = `@(Get-Content).Count` (includes blanks). Meta files not subject to ≥120.

| ID | Path | Lines | Depth |
|----|------|------:|-------|
| — | `README.md` | 81 | Meta + final table |
| — | `20260722-docs-completeness-audit.md` | 145 | Meta (this file) |
| D0 | `20260722-bare-host-no-prompt-inject.md` | 154 | ≥80 **met** |
| D1 | `20260722-trust-gate-design.md` | 339 | ≥120 **met** |
| D2 | `20260722-plugins-state-skill-gate.md` | 307 | ≥120 **met** |
| D3 | `20260722-bundle-counts-honesty.md` | 316 | ≥120 **met** |
| D4 | `20260722-bundle-hooks-mcp-merge.md` | 423 | ≥120 **met** |
| D5 | `20260722-hooks-cookbook-layout.md` | 442 | ≥120 **met** |
| D6 | `20260722-host-callback-bin-path.md` | 314 | ≥120 **met** |
| D7 | `20260722-hook-registry-reload.md` | 215 | ≥120 **met** |
| D8 | `20260722-plugin-manifest-abi-v1.md` | 152 | ≥120 **met** |
| D9 | `20260722-package-slash-acp.md` | 285 | ≥120 **met** |
| D10 | `20260722-nextcode-pack-extraction.md` | 174 | ≥120 **met** |
| D11 | `20260722-tools-abi-v1.md` | 293 | ≥120 **met** |
| D12 | `20260722-face-ui-hints-external-pane.md` | 141 | ≥120 **met** |
| D13 | `20260722-argv-plugin-security.md` | 174 | ≥120 **met** |

**Count:** 14 deepen design files (D0–D13) + README + this audit = **16** files under `deepen/`.

### Prior false audit (corrected)

An earlier pass claimed deepen D1–D13 were missing. At that moment **only this audit file** existed under `deepen/` (untracked); designs were **never git-committed**, so they could not be recovered via `git checkout`. They were **recreated** from master plan + research inventory, then **expanded** from stubs to contract depth. Do not treat “recently viewed path” as proof of on-disk presence without `Test-Path` / `Get-ChildItem`.

### Stub era (honesty)

Post-recreate files were often **15–50 line stubs**. That inflated “files exist” completeness without implementable contracts. The expand + finish waves fixed depth; scores below are **after finish**, not after recreate alone.

### Parallel-agent check (D3–D5)

D3/D4/D5 did **not** stall — on disk at 316 / 423 / 442 lines with real `face_plugins.rs` / registry / cookbook contracts. Finish pass focused on thinner D7/D10/D13 instead.

---

## Contradiction freeze (tools)

| Source | Wording | Tension |
|--------|---------|---------|
| Master open Q #2 | “argv in Phase 3, or **MCP-only** until then?” | Binary choice |
| Master Phase 3 bullet | “allowlisted argv runners **and/or** MCP bridge” | Both |
| D11 | “**MCP-first**; argv gated on D13” + “Phase 3 ships MCP **and/or** argv behind gates” | Looks like both |

**Frozen pick (docs law until master changelog updates):**

> **MCP-first ship order.** Phase 3 exit can be satisfied by `[[tools]] kind=mcp` alone.  
> `kind=argv` is **allowed in the same phase** only after D1 + D13 controls (or named follow-up tickets).  
> It is **not** “MCP-only forever,” and **not** “argv+MCP as equal day-one defaults.”

Authoritative file: [`20260722-tools-abi-v1.md`](./20260722-tools-abi-v1.md) §2 + D13 cross-link. Mirror into master open Q #2 when product docs update.

Other open Qs unchanged: **#1** manifest filename (D8 prefer `next-code-plugin.toml`, not frozen); **#3** Face hints vs external pane (D12 recommend A, not frozen).

---

## Coverage scorecard (after finish pass)

| # | Topic | Score (recreate) | Score (**after finish**) | On disk? | Notes |
|---|--------|------------------|--------------------------|----------|-------|
| 1 | Platform vs nextcode pack + bare | 90% | **95%** | Yes | Master + D0 freeze + D10 table + prompt.rs cites |
| 2 | Multilang ABI (hooks/slash/tools/skills/packages) | 80% | **88%** | Yes | D5/D8/D9/D11; filename open Q remains |
| 3 | Trust, enable, security, reload, BIN_PATH | 85% | **95%** | Yes | D7 now cites Face UI-only reload gap |
| 4 | Face sealed / UI hints only | 85% | **90%** | Yes | D12; open Q #3 remains |
| 5 | **Bare must NOT inject opinionated system/brand prompts** | 95% | **98%** | Yes | **D0 frozen**; inventory CORE→pack pending |
| 6 | Open questions unresolved | Listed | Listed | Yes | #2 frozen MCP-first; #1/#3 still open |

**Overall (honest):**

| Layer | After recreate (thin stubs) | **After finish** |
|-------|----------------------------|------------------|
| Vision | ~90% | **~95%** |
| Phase 1 implement-ready docs | ~70% (stubs looked like tickets) | **~93%** |
| Phase 2+ ABI | ~55–70% | **~86%** (open Qs #1/#3) |
| Combined docs completeness | ~85% claimed / **~75% real** | **~93%** |

**Do not** cite the recreate-era ~85% combined score without the stub caveat.

---

## What exists (good)

1. Master plan Option B′ + phases.
2. Five research reports + README.
3. Readiness gate plan.
4. Full deepen D0–D13 at **contract depth** (≥120; D0 ≥80) + index with final line-count table.
5. **Frozen** bare/Pi empty-shell prompt rule (D0); `system_prompt.md` reclassified as **nextcode pack** (not host CORE).
6. D11 docs stance: **MCP-first**; argv gated on D13 (contradiction freeze above).
7. D7 documents verified gap: Face `HooksAction::Reload` is catalog-only vs live `ToolRegistry.hook_registry`.

---

## Remaining gaps (not “missing files”)

- Inventory markdown still labels System prompt as CORE — update when pack extraction lands (D10) or as docs follow-up (D0 wins until then).
- Master open Qs: (1) manifest filename, (3) external pane vs hints — (2) tools timing **frozen MCP-first** in D11/audit.
- Quality review of expand drafts still worthwhile before coding.
- No production code yet — waiting **go ahead**.

---

## Ready for implement?

| Layer | Ready? |
|-------|--------|
| Product vision / Option B′ / Phase 0 | **Yes** |
| Phase 1 tickets with frozen contracts | **Yes (docs, post-finish)** — need owner **go ahead** |
| Phase 2+ ABI implement | **Partial** — contracts yes; open Qs #1/#3 block full freeze |

**Recommendation:** Docs gate satisfied for Phase 1 decision. Reply **go ahead** on readiness/master plan to start code.

**Status:** Audit re-scored after finish pass. No production code.
