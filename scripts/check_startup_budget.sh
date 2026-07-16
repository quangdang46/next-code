#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
binary=${1:-${NEXT_CODE_BIN:-${JCODE_BIN:-"$repo_root/target/release/next-code"}}}

if [[ ! -x "$binary" ]]; then
  echo "Binary not found or not executable: $binary" >&2
  echo "Build it first with: cargo build --release" >&2
  exit 1
fi

exec python3 "$repo_root/scripts/bench_startup.py" "$binary" --check --runs 3
