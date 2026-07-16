#!/usr/bin/env bash
# Phase 1 orchestrator: directory rename + Cargo.toml + Rust idents.
#
# Does NOT run rewrite_strings.py (product strings / env / home dual-read
# surface is a separate pass after the tree compiles under new crate names).
#
# Usage:
#   scripts/rebrand/run_p1.sh           # apply
#   scripts/rebrand/run_p1.sh --dry-run # report only (rename dry-run + py dry-run)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

DRY=()
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY=(--dry-run)
  echo "=== P1 dry-run (no writes / no git mv) ==="
else
  echo "=== P1 apply: rename_crates → rewrite_cargo → rewrite_rust_idents ==="
fi

echo ""
echo "--- 1/3 rename_crates.sh ---"
bash "$DIR/rename_crates.sh" ${DRY:+--dry-run}

echo ""
echo "--- 2/3 rewrite_cargo.py ---"
python3 "$DIR/rewrite_cargo.py" "${DRY[@]}" -v

echo ""
echo "--- 3/3 rewrite_rust_idents.py ---"
python3 "$DIR/rewrite_rust_idents.py" "${DRY[@]}" -v

echo ""
echo "=== P1 core complete ==="
echo "Next (separate, not run here):"
echo "  python3 scripts/rebrand/rewrite_strings.py [--dry-run]"
echo "  cargo generate-lockfile   # or cargo check"
echo "  python3 scripts/rebrand/rg_gate.py"
echo ""
echo "Note: strings / dual-read / installers are Phase 2–4 work."
echo "Do not expect a green tree until Cargo.lock is regenerated and"
echo "any remaining string path deps outside Cargo.toml are rewritten."
