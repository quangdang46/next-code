#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
prompt=${1:-"Use the bash tool to run 'pwd', then use the ls tool to list the current directory, then respond with DONE."}
provider=${NEXT_CODE_PROVIDER:-${JCODE_PROVIDER:-auto}}
cargo_exec="$repo_root/scripts/cargo_exec.sh"

if [[ ! -x "$repo_root/target/release/next-code" && ! -x "$repo_root/target/release/next-code" ]]; then
  (cd "$repo_root" && "$cargo_exec" build --release)
fi

workdir=$(mktemp -d)
trap 'rm -rf "$workdir"' EXIT

NEXT_CODE_HOME="$workdir" PATH="$repo_root/target/release:$PATH" \
  next-code run --no-update --trace --provider "$provider" "$prompt"
