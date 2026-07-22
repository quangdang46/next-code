# Deepen designs — platform Phase 1–4 tickets (2026-07-22)

Frozen / near-frozen design notes that turn the master plan into implementable contracts.

**Master:** [`../../PLAN-20260722-pi-full-custom-platform.md`](../../PLAN-20260722-pi-full-custom-platform.md)  
**Readiness gate:** [`../../PLAN-20260722-platform-implement-readiness.md`](../../PLAN-20260722-platform-implement-readiness.md)  
**Research evidence:** [`../README.md`](../README.md)  
**Audit:** [`20260722-docs-completeness-audit.md`](./20260722-docs-completeness-audit.md)

---

## Line-count expectations

| Class | Target | Notes |
|-------|--------|-------|
| **D0** (bare-host freeze) | **≥80** lines | Product-law companion to D10; met after expand |
| **D1–D13** design contracts | **≥120** lines | Problem, frozen intent, acceptance tests, exit criteria |
| **README** / **audit** | Meta | Index + honesty scorecard; not subject to ≥120 |

**Stubs expanded:** **Yes** (2026-07-22 expand wave + finish pass). Re-count with full line count (including blanks), not PowerShell `Measure-Object -Line` (skips blanks).

```powershell
Get-ChildItem .\*.md | ForEach-Object {
  $n = @(Get-Content $_.FullName).Count
  "{0,5} {1}" -f $n, $_.Name
}
```

---

## Final line-count table (2026-07-22 finish pass)

Counts = `@(Get-Content).Count` (includes blanks).

| ID | File | Lines | Target | Status |
|----|------|------:|-------:|--------|
| — | `README.md` | 81 | meta | OK |
| — | `20260722-docs-completeness-audit.md` | 145 | meta | OK |
| D0 | `20260722-bare-host-no-prompt-inject.md` | 154 | ≥80 | **met** |
| D1 | `20260722-trust-gate-design.md` | 339 | ≥120 | **met** |
| D2 | `20260722-plugins-state-skill-gate.md` | 307 | ≥120 | **met** |
| D3 | `20260722-bundle-counts-honesty.md` | 316 | ≥120 | **met** |
| D4 | `20260722-bundle-hooks-mcp-merge.md` | 423 | ≥120 | **met** |
| D5 | `20260722-hooks-cookbook-layout.md` | 442 | ≥120 | **met** |
| D6 | `20260722-host-callback-bin-path.md` | 314 | ≥120 | **met** |
| D7 | `20260722-hook-registry-reload.md` | 215 | ≥120 | **met** (expanded finish) |
| D8 | `20260722-plugin-manifest-abi-v1.md` | 152 | ≥120 | **met** |
| D9 | `20260722-package-slash-acp.md` | 285 | ≥120 | **met** |
| D10 | `20260722-nextcode-pack-extraction.md` | 174 | ≥120 | **met** (expanded finish) |
| D11 | `20260722-tools-abi-v1.md` | 293 | ≥120 | **met** |
| D12 | `20260722-face-ui-hints-external-pane.md` | 141 | ≥120 | **met** |
| D13 | `20260722-argv-plugin-security.md` | 174 | ≥120 | **met** (expanded finish) |

**Finish-pass expansions:** D7 (live vs UI reload), D10 (prompt.rs cite), D13 (execute.rs + D11 freeze link). D3–D5 were already full contracts from the parallel wave (no stub left).

---

## Index (D0–D13 + audit)

| ID | Priority | File | Topic | Depth target |
|----|----------|------|-------|--------------|
| D0 | P0 | [`20260722-bare-host-no-prompt-inject.md`](./20260722-bare-host-no-prompt-inject.md) | Bare/platform no opinionated prompt inject; `system_prompt.md` → pack | ≥80 |
| D1 | P0 | [`20260722-trust-gate-design.md`](./20260722-trust-gate-design.md) | Project trust before exe hooks/plugins | ≥120 |
| D2 | P0 | [`20260722-plugins-state-skill-gate.md`](./20260722-plugins-state-skill-gate.md) | Enable-state gates skill ingest | ≥120 |
| D3 | P1 | [`20260722-bundle-counts-honesty.md`](./20260722-bundle-counts-honesty.md) | Stop UI lies on Active/Blocked | ≥120 |
| D4 | P1 | [`20260722-bundle-hooks-mcp-merge.md`](./20260722-bundle-hooks-mcp-merge.md) | Wire bundle hooks/MCP into registries | ≥120 |
| D5 | P1 | [`20260722-hooks-cookbook-layout.md`](./20260722-hooks-cookbook-layout.md) | Bash/Python/Node PreToolUse cookbook | ≥120 |
| D6 | P1 | [`20260722-host-callback-bin-path.md`](./20260722-host-callback-bin-path.md) | `NEXT_CODE_BIN_PATH` | ≥120 |
| D7 | P1 | [`20260722-hook-registry-reload.md`](./20260722-hook-registry-reload.md) | Reload after package compile-in | ≥120 |
| D8 | P2 | [`20260722-plugin-manifest-abi-v1.md`](./20260722-plugin-manifest-abi-v1.md) | Manifest ABI v1 | ≥120 |
| D9 | P2 | [`20260722-package-slash-acp.md`](./20260722-package-slash-acp.md) | Package slash via ACP | ≥120 |
| D10 | P2 | [`20260722-nextcode-pack-extraction.md`](./20260722-nextcode-pack-extraction.md) | Product profile / pack extraction | ≥120 |
| D11 | P2 | [`20260722-tools-abi-v1.md`](./20260722-tools-abi-v1.md) | `[[tools]]` argv / MCP | ≥120 |
| D12 | P2 | [`20260722-face-ui-hints-external-pane.md`](./20260722-face-ui-hints-external-pane.md) | Phase 4 UI ceiling | ≥120 |
| D13 | P2 | [`20260722-argv-plugin-security.md`](./20260722-argv-plugin-security.md) | Security beyond trust | ≥120 |

---

## Docs only

These files are **not** an implement approval. Reply **go ahead** on the master / readiness plan before production Rust.
