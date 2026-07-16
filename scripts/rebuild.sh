#!/bin/bash
# Build and restart next-code in one shot.
# Usage: scripts/rebuild.sh

set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building next-code..."
cargo build --release 2>&1 | tail -3

echo "==> Installing..."
mkdir -p ~/.next-code/builds/current
if [[ -x target/release/next-code ]]; then
  cp target/release/next-code ~/.next-code/builds/current/next-code
elif [[ -x target/release/jcode ]]; then
  cp target/release/jcode ~/.next-code/builds/current/next-code
else
  echo "error: release binary not found (expected target/release/next-code)" >&2
  exit 1
fi
strip ~/.next-code/builds/current/next-code

# Keep launcher + one-release compat symlink in sync when present.
if [[ -d "$HOME/.local/bin" ]]; then
  ln -sfn "$HOME/.next-code/builds/current/next-code" "$HOME/.local/bin/next-code"
  ln -sfn "next-code" "$HOME/.local/bin/jcode"
fi

echo "==> Killing old server daemon..."
pkill -f "next-code.*serve" 2>/dev/null || true
pkill -f "jcode.*serve" 2>/dev/null || true
sleep 0.5

echo "==> Done. Binary: $(ls -lh ~/.next-code/builds/current/next-code | awk '{print $5}')"
echo "    Server stopped. Launch next-code again to use the new binary."
echo "    (Cmd+; or ~/.next-code/builds/current/next-code)"
