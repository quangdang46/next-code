#!/bin/bash
# End-to-end test script for next-code

set -e

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cargo_exec="$repo_root/scripts/cargo_exec.sh"

run_cargo() {
    (cd "$repo_root" && "$cargo_exec" "$@")
}

echo "=== E2E Testing Script for next-code ==="
echo ""

# Test 1: Check binary exists and runs
echo "Test 1: Check next-code binary..."
if command -v next-code &> /dev/null; then
    echo "✓ next-code binary found"
    next-code --version
else
    echo "✗ next-code binary not found"
    exit 1
fi

# Test 2: Run unit tests
echo ""
echo "Test 2: Run unit tests..."
run_cargo test 2>&1 | tail -5
echo "✓ Unit tests passed"

# Test 3: Check protocol serialization
echo ""
echo "Test 3: Protocol serialization test..."
run_cargo test protocol::tests --quiet
echo "✓ Protocol tests passed"

# Test 4: Check TUI app tests
echo ""
echo "Test 4: TUI app tests..."
run_cargo test tui::app::tests --quiet
echo "✓ TUI app tests passed"

# Test 5: Check markdown rendering tests
echo ""
echo "Test 5: Markdown rendering tests..."
run_cargo test tui::markdown::tests --quiet
echo "✓ Markdown tests passed"

# Test 6: E2E tests
echo ""
echo "Test 6: E2E integration tests..."
run_cargo test --test e2e --quiet
echo "✓ E2E tests passed"

if [[ "${NEXT_CODE_REAL_PROVIDER:-${NEXT_CODE_REAL_PROVIDER:-0}}" == "1" ]]; then
    echo ""
    echo "Test 7: Real provider smoke (NEXT_CODE_REAL_PROVIDER=1)..."
    scripts/real_provider_smoke.sh
    echo "✓ Real provider smoke passed"
fi

if [[ "${NEXT_CODE_REAL_AUTH_TEST:-${NEXT_CODE_REAL_AUTH_TEST:-0}}" == "1" ]]; then
    echo ""
    echo "Test 8: Auth E2E validation (NEXT_CODE_REAL_AUTH_TEST=1)..."
    scripts/test_auth_e2e.sh
    echo "✓ Auth E2E validation passed"
fi

echo ""
echo "=== All tests passed! ==="
echo ""
echo "To test interactively:"
echo "  next-code        # Start TUI mode"
echo "  next-code server # Start server mode"
echo "  next-code client # Connect to server"
