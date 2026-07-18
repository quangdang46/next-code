# Plan: pre-merge-check
Date: 2026-07-18
Status: done
Goal: Determine whether rebrand/next-code is clean enough to merge to main.
Constraint: DO NOT merge main. Report only.

## Packages
1. check-base — full `cargo test -p next-code-base --lib`, classify every failure
2. check-residual — product-surface scan (telemetry, hosted subscription, jcode/desktop/mobile leftovers)
3. check-tui-appcore — focused tests for packages we heavily changed

## Success
- Evidence under data/orchestrator/evidence/
- Clear merge recommendation: merge / no-merge + why
