#!/usr/bin/env bash
# Phase 1 helper: rename crate directories jcode-* → next-code-*.
# Does NOT rewrite Cargo.toml contents (see rewrite_cargo.py).
# Safe to re-run: skips dirs already renamed / missing.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

DRY_RUN=0
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=1
fi

mv_dir() {
  local src="$1"
  local dst="$2"
  if [[ ! -e "$src" ]]; then
    echo "skip (missing): $src"
    return 0
  fi
  if [[ -e "$dst" ]]; then
    echo "skip (exists):  $dst"
    return 0
  fi
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "DRY  git mv $src $dst"
    return 0
  fi
  git mv "$src" "$dst"
  echo "OK   git mv $src $dst"
}

echo "==> renaming crates/jcode-* → crates/next-code-*"
shopt -s nullglob
for d in crates/jcode-*; do
  base="${d#crates/jcode-}"
  mv_dir "$d" "crates/next-code-${base}"
done

echo "==> renaming evals/jcode-edit-bench (if present)"
mv_dir "evals/jcode-edit-bench" "evals/next-code-edit-bench"

echo "==> done (dirs only; run rewrite_cargo.py next)"
