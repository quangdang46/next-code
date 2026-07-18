#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
provider=${NEXT_CODE_PROVIDER:-auto}
prompt=${1:-"Use the bash tool to run 'pwd', then use the ls tool to list the current directory, then respond with DONE."}
expect=${NEXT_CODE_TRACE_EXPECT:-DONE}
cargo_exec="$repo_root/scripts/cargo_exec.sh"

echo "=== Real Provider Smoke ==="
echo "Provider: ${provider}"

if [[ "${NEXT_CODE_REAL_PROVIDER_TEST_API:-1}" == "1" ]]; then
  if [[ "${provider}" == "claude" && "${NEXT_CODE_USE_DIRECT_API:-0}" != "1" ]]; then
    echo ""
    echo "Test 1: Claude CLI smoke (test_api)"
    if [[ "${NEXT_CODE_REMOTE_CARGO:-0}" == "1" ]]; then
      (cd "$repo_root" && "$cargo_exec" build --bin test_api)
      (cd "$repo_root" && ./target/debug/test_api)
    else
      (cd "$repo_root" && cargo run --bin test_api)
    fi
  else
    echo ""
    echo "Test 1: Skipping test_api (provider=${provider}, NEXT_CODE_USE_DIRECT_API=${NEXT_CODE_USE_DIRECT_API:-0})"
  fi
fi

echo ""
echo "Test 2: Tool harness (network tools enabled)"
if [[ "${NEXT_CODE_REMOTE_CARGO:-0}" == "1" ]]; then
  (cd "$repo_root" && "$cargo_exec" build --bin next-code-harness)
  (cd "$repo_root" && ./target/debug/next-code-harness -- --include-network)
else
  (cd "$repo_root" && cargo run --bin next-code-harness -- --include-network)
fi

echo ""
echo "Test 3: End-to-end trace"
if [[ ! -x "$repo_root/target/release/next-code" && ! -x "$repo_root/target/release/next-code" ]]; then
  (cd "$repo_root" && "$cargo_exec" build --release)
fi

workdir=$(mktemp -d)
trap 'rm -rf "$workdir"' EXIT

set +e
output=$(NEXT_CODE_HOME="$workdir" PATH="$repo_root/target/release:$PATH" \
  next-code run --no-update --trace --provider "$provider" "$prompt" 2>&1)
status=$?
set -e

printf "%s\n" "$output"

if [[ $status -ne 0 ]]; then
  echo "Trace failed with exit code $status" >&2
  exit $status
fi

if [[ -n "$expect" ]] && ! grep -q "$expect" <<<"$output"; then
  echo "Trace output did not include expected marker: ${expect}" >&2
  exit 1
fi

echo ""
echo "=== Real provider smoke OK ==="
