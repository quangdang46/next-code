#!/usr/bin/env bash
# Install the current release binary into the immutable version store,
# update the stable + current channel symlinks, and point the launcher at current.
#
# Paths after install:
# - ~/.next-code/builds/versions/<hash>/next-code (immutable)
# - ~/.next-code/builds/stable/next-code -> .../versions/<hash>/next-code
# - ~/.next-code/builds/current/next-code -> .../versions/<hash>/next-code
# - ~/.local/bin/next-code -> ~/.next-code/builds/current/next-code (launcher)
# - ~/.local/bin/jcode -> next-code (compat symlink for one release)
#
# Legacy ~/.jcode is dual-read by the binary and migrates automatically when
# ~/.next-code is missing. Installers write to ~/.next-code going forward.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

profile="${NEXT_CODE_RELEASE_PROFILE:-${JCODE_RELEASE_PROFILE:-release-lto}}"
if [[ "${1:-}" == "--fast" ]]; then
  profile="release"
  shift
fi

if [[ "$#" -gt 0 ]]; then
  echo "Usage: $0 [--fast]" >&2
  exit 1
fi

case "$profile" in
  release-lto)
    echo "Building with LTO (this takes a few minutes)..."
    ;;
  release)
    echo "Building fast release profile (no LTO)..."
    ;;
  *)
    echo "Unsupported profile: $profile (expected: release or release-lto)" >&2
    exit 1
    ;;
esac

cargo build --profile "$profile" --manifest-path "$repo_root/Cargo.toml"
bin="$repo_root/target/$profile/next-code"

if [[ ! -x "$bin" ]]; then
  # Fall back to a still-present legacy binary name during the rebrand window.
  if [[ -x "$repo_root/target/$profile/jcode" ]]; then
    bin="$repo_root/target/$profile/jcode"
  else
    echo "Release binary not found: $bin" >&2
    exit 1
  fi
fi

hash=""
if command -v git >/dev/null 2>&1; then
  if git -C "$repo_root" rev-parse --git-dir >/dev/null 2>&1; then
    hash="$(git -C "$repo_root" rev-parse --short HEAD 2>/dev/null || true)"
    if [[ -n "${hash}" ]] && [[ -n "$(git -C "$repo_root" status --porcelain 2>/dev/null || true)" ]]; then
      hash="${hash}-dirty"
    fi
  fi
fi

if [[ -z "$hash" ]]; then
  hash="$(date +%Y%m%d%H%M%S)"
fi

# Prefer NEXT_CODE_HOME, then legacy JCODE_HOME, then ~/.next-code (with legacy
# ~/.jcode builds dir as a last-resort fallback when it already exists alone).
if [ -n "${NEXT_CODE_HOME:-}" ]; then
  next_code_home="$NEXT_CODE_HOME"
elif [ -n "${JCODE_HOME:-}" ]; then
  next_code_home="$JCODE_HOME"
elif [ -d "$HOME/.next-code" ] || [ ! -d "$HOME/.jcode" ]; then
  next_code_home="$HOME/.next-code"
else
  # Legacy install still present; keep writing builds there until migration runs.
  next_code_home="$HOME/.jcode"
fi

# Install versioned binary into <home>/builds/versions/<hash>/
builds_dir="$next_code_home/builds"
version_dir="$builds_dir/versions/$hash"
mkdir -p "$version_dir"
install -m 755 "$bin" "$version_dir/next-code"

# Update stable symlink
stable_dir="$builds_dir/stable"
mkdir -p "$stable_dir"
ln -sfn "$version_dir/next-code" "$stable_dir/next-code"

# Update stable-version marker
printf '%s\n' "$hash" > "$builds_dir/stable-version"

# Update current symlink + marker
current_dir="$builds_dir/current"
mkdir -p "$current_dir"
ln -sfn "$version_dir/next-code" "$current_dir/next-code"
printf '%s\n' "$hash" > "$builds_dir/current-version"

# Update launcher path to current channel
install_dir="${NEXT_CODE_INSTALL_DIR:-${JCODE_INSTALL_DIR:-$HOME/.local/bin}}"
mkdir -p "$install_dir"
ln -sfn "$current_dir/next-code" "$install_dir/next-code"
# Compat symlink for one release so existing `jcode` muscle memory keeps working.
ln -sfn "next-code" "$install_dir/jcode"

echo "Installed: $version_dir/next-code"
echo "Updated stable symlink: $stable_dir/next-code -> $version_dir/next-code"
echo "Updated current symlink: $current_dir/next-code -> $version_dir/next-code"
echo "Updated launcher symlink: $install_dir/next-code -> $current_dir/next-code"
echo "Updated compat symlink: $install_dir/jcode -> next-code"

# Configure supported desktop launch hotkeys as part of installation. This is
# idempotent and best-effort because headless installs may not expose a desktop
# session; the first interactive launch retries automatically.
case "$(uname -s)" in
  Darwin|Linux)
    if "$install_dir/next-code" setup-hotkey </dev/null >/dev/null 2>&1; then
      echo "Configured system-wide next-code launch hotkeys (when supported)."
    fi
    ;;
esac

# Force-reload any running background server onto the binary we just installed
# (issue #291). Unconditional `--force` is required after install: version
# comparison on an older daemon can report "already current" while the process
# image is still the previous build (stale /proc mapping vs updated symlink).
# Hands live headless/swarm sessions to the new process; no-op if no server.
if [ "${NEXT_CODE_SKIP_SERVER_RELOAD:-${JCODE_SKIP_SERVER_RELOAD:-}}" != "1" ]; then
  if "$install_dir/next-code" server reload --force </dev/null >/dev/null 2>&1; then
    echo "Force-reloaded the running next-code server onto $hash (if one was active)."
  else
    echo "Warning: server reload --force failed; restart serve manually if tools look stale."
  fi
fi

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$install_dir"; then
  echo ""
  echo "Tip: add $install_dir to PATH if needed."
fi

# Ensure the launcher dir is on PATH for bash, zsh and fish in future shells.
# shellcheck source=scripts/lib/configure_path.sh
. "$(dirname "$0")/lib/configure_path.sh"
next_code_configure_path "$install_dir"
