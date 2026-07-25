#!/bin/bash
# GJC dogfood launcher for next-code
# Usage: ./gjc-dogfood.cmd [worktree-name]
cd "$(dirname "$0")/.." || exit 1
BRANCH="${1:-dogfood-$(date +%s)}"
echo "=== GJC Dogfood: next-code ==="
echo "Worktree branch: $BRANCH"
echo "Model: 9router/cmc/deepseek/deepseek-v4-flash"
echo ""
# Change profile to use next-code's skill dir
GJC_SKILL_DIR="$(pwd)/.gjc/skills" gjc --worktree "$BRANCH" -m 9router/cmc/deepseek/deepseek-v4-flash
