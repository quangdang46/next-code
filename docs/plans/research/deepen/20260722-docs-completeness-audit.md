# Docs completeness audit — platform / nextcode vision (2026-07-22)

**Scope:** Vision docs for “Pi surfaces × herdr multilang + nextcode pack”.  
**Branch at verify:** `pr-face-failover-visible-notice` (worktree may switch; re-check with `git branch --show-current`)  
**Verify pass:** 2026-07-22 (re-inventory after false “missing” report + recreate)

---

## On-disk inventory (verified)

### Top-level plans

| Path | On disk? | Git |
|------|----------|-----|
| `docs/plans/PLAN-20260722-pi-full-custom-platform.md` | **Yes** | Tracked |
| `docs/plans/PLAN-20260722-platform-implement-readiness.md` | **Yes** | Untracked (written this verify) |

### Research (`docs/plans/research/`)

| Path | On disk? | Git |
|------|----------|-----|
| `README.md` | Yes | Tracked |
| `20260722-pi-extension-surfaces.md` | Yes | Tracked |
| `20260722-herdr-multilang-abi.md` | Yes | Tracked |
| `20260722-opencode-plugin-hooks.md` | Yes | Tracked |
| `20260722-nextcode-extension-inventory.md` | Yes | Tracked |
| `20260722-face-customization-limits.md` | Yes | Tracked |

### Deepen (`docs/plans/research/deepen/`)

| ID | Path | On disk? |
|----|------|----------|
| — | `README.md` | **Yes** |
| — | `20260722-docs-completeness-audit.md` | **Yes** (this file) |
| D0 | `20260722-bare-host-no-prompt-inject.md` | **Yes** (P0 freeze) |
| D1 | `20260722-trust-gate-design.md` | **Yes** |
| D2 | `20260722-plugins-state-skill-gate.md` | **Yes** |
| D3 | `20260722-bundle-counts-honesty.md` | **Yes** |
| D4 | `20260722-bundle-hooks-mcp-merge.md` | **Yes** |
| D5 | `20260722-hooks-cookbook-layout.md` | **Yes** |
| D6 | `20260722-host-callback-bin-path.md` | **Yes** |
| D7 | `20260722-hook-registry-reload.md` | **Yes** |
| D8 | `20260722-plugin-manifest-abi-v1.md` | **Yes** |
| D9 | `20260722-package-slash-acp.md` | **Yes** |
| D10 | `20260722-nextcode-pack-extraction.md` | **Yes** |
| D11 | `20260722-tools-abi-v1.md` | **Yes** |
| D12 | `20260722-face-ui-hints-external-pane.md` | **Yes** |
| D13 | `20260722-argv-plugin-security.md` | **Yes** |

**Count:** 14 deepen design files (D0–D13) + README + this audit = **16** files under `deepen/`.

### Prior false audit (corrected)

An earlier pass claimed deepen D1–D13 were missing. At that moment **only this audit file** existed under `deepen/` (untracked); designs were **never git-committed**, so they could not be recovered via `git checkout`. They were **recreated** from master plan + research inventory in the verify pass. Do not treat “recently viewed path” as proof of on-disk presence without `Test-Path` / `Get-ChildItem`.

---

## Coverage scorecard (after recreate)

| # | Topic | Score | On disk? | Notes |
|---|--------|-------|----------|-------|
| 1 | Platform vs nextcode pack + bare | **90%** | Yes | Master + D0 + D10 |
| 2 | Multilang ABI (hooks/slash/tools/skills/packages) | **80%** | Yes | Research + D5/D8/D9/D11 sketches; filename open Q |
| 3 | Trust, enable, security, reload, BIN_PATH | **85%** | Yes | D1–D2, D6–D7, D13 |
| 4 | Face sealed / UI hints only | **85%** | Yes | Face research + D12; Open Q remains |
| 5 | **Bare must NOT inject opinionated system/brand prompts** | **95%** | **Yes** | **D0 frozen**; inventory still says CORE until code/KB follow-up |
| 6 | Open questions unresolved | Listed | Yes | Master 3 Qs + leftovers |

**Overall:** Vision ~**90%**. Implement-ready Phase 1 docs ~**85%**. Phase 2+ ABI ~**70%** (open Qs). Combined docs completeness ~**85%**.

---

## What exists (good)

1. Master plan Option B′ + phases.
2. Five research reports + README.
3. Readiness gate plan.
4. Full deepen D0–D13 + index.
5. **Frozen** bare/Pi empty-shell prompt rule (D0); `system_prompt.md` reclassified as **nextcode pack** (not host CORE).

---

## Remaining gaps (not “missing files”)

- Inventory markdown still labels System prompt as CORE — update when pack extraction lands (D10) or as docs follow-up.
- Master open Qs unresolved (manifest name, tools timing, external pane).
- No production code yet — waiting **go ahead**.

---

## Ready for implement?

| Layer | Ready? |
|-------|--------|
| Product vision / Option B′ / Phase 0 | **Yes** |
| Phase 1 tickets with frozen contracts | **Yes (docs)** — need owner **go ahead** |
| Phase 2+ ABI implement | **Partial** — sketches yes; open Qs block freeze |

**Recommendation:** Docs gate satisfied for Phase 1 decision. Reply **go ahead** on readiness/master plan to start code.

**Status:** Audit updated after on-disk verify + recreate. No production code.
