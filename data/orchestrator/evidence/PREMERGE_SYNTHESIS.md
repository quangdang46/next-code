# Pre-merge check synthesis
Date: 2026-07-18
Branch: rebrand/next-code @ 00428e461
Policy: do **not** merge main until residual product surfaces are clean **and** suite policy is accepted.

## Agent results

### 1) Residual surface (check-residual)
- Risk: **mostly-clean**
- Merge blocker for residual product identity: **no**
- Clean: jcode, desktop/mobile crates, telemetry runtime, hosted next-code login provider, `/subscription` registration
- Non-blocking leftovers: stale docs (`ACCOUNT_*`), installer funnel `telemetry.next-code.sh`, dead `"next-code"` display/color arms, budget JSON paths

### 2) next-code-base lib (check-base)
- **1072 passed / 12 failed / 1 ignored** (~55.6s serial)
- Failures classified **B** (product/test drift) or **C** (env flake) — **not** residual telemetry/subscription
- Package recommendation: **ok-with-known-failures** (or block-merge if policy requires full green)

Fail list:
1. config Luna sidecar default text
2–3. tool profiles still expect `apply_patch` tool name
4–5. memory hybrid ranking empty/0
6. platform spawn_detached child exit
7. failover access_denied classify
8–11. session initial cwd / auto-poke rendering
12. skill reload cwd NotFound

### 3) tui + app-core + metadata (check-tui)
- `cargo check` app-core/tui/metadata/next-code bin: **pass**
- provider-metadata lib: **14/14 pass**
- tui onboarding_welcome: **4/4 pass**
- tui top-level suggestions config/alignment: **1/1 pass**
- Package recommendation: **merge-safe** for these focused packages

## Overall merge recommendation

### **NO-MERGE to main yet** if gate = full `next-code-base --lib` green.

### **Possible merge** only if team accepts:
1. known 12 base lib failures as pre-existing/test drift (not rebrand regressions), **and**
2. optional follow-ups for installer telemetry funnel + stale docs as non-blockers.

Rebrand residual product surfaces (jcode/desktop/mobile/telemetry phone-home/hosted next-code login) are **not** the merge blocker.

## Evidence
- data/orchestrator/evidence/PREMERGE_BASE_REPORT.md
- data/orchestrator/evidence/PREMERGE_RESIDUAL_REPORT.md
- data/orchestrator/evidence/PREMERGE_TUI_APPCORE_REPORT.md
- /tmp/premerge-base-lib.txt
