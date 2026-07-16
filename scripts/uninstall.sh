#!/usr/bin/env bash
# Uninstall next-code binaries and (optionally) all user data.
#
# Default: removes installed binaries, build channels, and the launcher
# symlink, but keeps user data (config, auth, sessions, logs) so a clean
# reinstall picks up where you left off.
#
# Flags:
#   --purge     Also delete ~/.next-code (and legacy ~/.jcode if present).
#   --dry-run   Print what would be removed without deleting anything.
#   --yes       Skip the confirmation prompt.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/quangdang46/next-code/master/scripts/uninstall.sh | bash
#   bash scripts/uninstall.sh --purge
set -euo pipefail

info() { printf '\033[1;34m%s\033[0m\n' "$*"; }
warn() { printf '\033[1;33m%s\033[0m\n' "$*"; }
err()  { printf '\033[1;31merror: %s\033[0m\n' "$*" >&2; exit 1; }

PURGE=false
DRY_RUN=false
ASSUME_YES=false

for arg in "$@"; do
  case "$arg" in
    --purge)   PURGE=true ;;
    --dry-run) DRY_RUN=true ;;
    --yes|-y)  ASSUME_YES=true ;;
    --help|-h)
      sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) err "Unknown flag: $arg (supported: --purge, --dry-run, --yes)" ;;
  esac
done

OS="$(uname -s)"
case "$OS" in
  MINGW*|MSYS*|CYGWIN*)
    if [ -n "${NEXT_CODE_HOME:-}" ]; then
      NEXT_CODE_HOME_DIR="$NEXT_CODE_HOME"
    elif [ -n "${JCODE_HOME:-}" ]; then
      NEXT_CODE_HOME_DIR="$JCODE_HOME"
    else
      NEXT_CODE_HOME_DIR="${LOCALAPPDATA:?LOCALAPPDATA not set}/next-code"
    fi
    LEGACY_HOME_DIR="${LOCALAPPDATA}/jcode"
    LAUNCHER_DIR="${NEXT_CODE_INSTALL_DIR:-${JCODE_INSTALL_DIR:-$LOCALAPPDATA/next-code/bin}}"
    LAUNCHER="$LAUNCHER_DIR/next-code.exe"
    LEGACY_LAUNCHER="$LAUNCHER_DIR/jcode.exe"
    # Also check the old default install dir.
    LEGACY_LAUNCHER_DIR="${LOCALAPPDATA}/jcode/bin"
    LEGACY_LAUNCHER_ALT="$LEGACY_LAUNCHER_DIR/jcode.exe"
    BUILDS_DIR="$NEXT_CODE_HOME_DIR/builds"
    LEGACY_BUILDS_DIR="$LEGACY_HOME_DIR/builds"
    USER_DATA_DIR="$NEXT_CODE_HOME_DIR"
    LEGACY_USER_DATA_DIR="$LEGACY_HOME_DIR"
    ;;
  *)
    if [ -n "${NEXT_CODE_HOME:-}" ]; then
      NEXT_CODE_HOME_DIR="$NEXT_CODE_HOME"
    elif [ -n "${JCODE_HOME:-}" ]; then
      NEXT_CODE_HOME_DIR="$JCODE_HOME"
    else
      NEXT_CODE_HOME_DIR="$HOME/.next-code"
    fi
    LEGACY_HOME_DIR="$HOME/.jcode"
    LAUNCHER_DIR="${NEXT_CODE_INSTALL_DIR:-${JCODE_INSTALL_DIR:-$HOME/.local/bin}}"
    LAUNCHER="$LAUNCHER_DIR/next-code"
    LEGACY_LAUNCHER="$LAUNCHER_DIR/jcode"
    LEGACY_LAUNCHER_DIR=""
    LEGACY_LAUNCHER_ALT=""
    BUILDS_DIR="$NEXT_CODE_HOME_DIR/builds"
    LEGACY_BUILDS_DIR="$LEGACY_HOME_DIR/builds"
    USER_DATA_DIR="$NEXT_CODE_HOME_DIR"
    LEGACY_USER_DATA_DIR="$LEGACY_HOME_DIR"
    ;;
esac

# Collect removal targets.
TARGETS=()
[ -e "$LAUNCHER" ] || [ -L "$LAUNCHER" ] && TARGETS+=("$LAUNCHER (launcher)")
[ -e "$LEGACY_LAUNCHER" ] || [ -L "$LEGACY_LAUNCHER" ] && TARGETS+=("$LEGACY_LAUNCHER (legacy jcode launcher)")
if [ -n "$LEGACY_LAUNCHER_ALT" ] && { [ -e "$LEGACY_LAUNCHER_ALT" ] || [ -L "$LEGACY_LAUNCHER_ALT" ]; }; then
  TARGETS+=("$LEGACY_LAUNCHER_ALT (legacy Windows launcher)")
fi
[ -d "$BUILDS_DIR" ] && TARGETS+=("$BUILDS_DIR (installed binaries: stable/current/canary/versions)")
[ -d "$LEGACY_BUILDS_DIR" ] && [ "$LEGACY_BUILDS_DIR" != "$BUILDS_DIR" ] && \
  TARGETS+=("$LEGACY_BUILDS_DIR (legacy installed binaries)")
if [ "$PURGE" = true ]; then
  [ -d "$USER_DATA_DIR" ] && \
    TARGETS+=("$USER_DATA_DIR (ALL user data: config, auth, sessions, logs, memory)")
  [ -d "$LEGACY_USER_DATA_DIR" ] && [ "$LEGACY_USER_DATA_DIR" != "$USER_DATA_DIR" ] && \
    TARGETS+=("$LEGACY_USER_DATA_DIR (legacy ALL user data under ~/.jcode)")
fi

# Compatibility wrapper installed by some setups.
SELFDEV_WRAPPER="$HOME/.local/bin/selfdev"
if [ -f "$SELFDEV_WRAPPER" ] && grep -Eq 'jcode|next-code' "$SELFDEV_WRAPPER" 2>/dev/null; then
  TARGETS+=("$SELFDEV_WRAPPER (selfdev wrapper)")
fi

if [ ${#TARGETS[@]} -eq 0 ]; then
  info "Nothing to uninstall: no next-code installation found."
  exit 0
fi

info "The following will be removed:"
for t in "${TARGETS[@]}"; do
  printf '  - %s\n' "$t"
done
if [ "$PURGE" = false ]; then
  warn "User data in $USER_DATA_DIR is kept (config, auth, sessions, logs)."
  if [ -d "$LEGACY_USER_DATA_DIR" ] && [ "$LEGACY_USER_DATA_DIR" != "$USER_DATA_DIR" ]; then
    warn "Legacy user data in $LEGACY_USER_DATA_DIR is also kept."
  fi
  warn "Run with --purge for a full wipe."
fi

if [ "$DRY_RUN" = true ]; then
  info "Dry run: nothing was deleted."
  exit 0
fi

if [ "$ASSUME_YES" = false ]; then
  if [ -t 0 ]; then
    printf 'Proceed? [y/N] '
    read -r reply
    case "$reply" in
      y|Y|yes|YES) ;;
      *) info "Aborted."; exit 1 ;;
    esac
  else
    # Piped (curl | bash): require explicit --yes to avoid accidental deletion.
    err "stdin is not a terminal; re-run with --yes to confirm (e.g. curl ... | bash -s -- --yes)"
  fi
fi

# Stop any running next-code / legacy jcode server so files are not recreated mid-wipe.
if command -v pkill >/dev/null 2>&1; then
  pkill -f 'next-code( .*)? serve' 2>/dev/null || true
  pkill -f 'jcode( .*)? serve' 2>/dev/null || true
fi

remove() {
  local path="$1"
  if [ -e "$path" ] || [ -L "$path" ]; then
    rm -rf -- "$path"
    info "Removed $path"
  fi
}

remove "$LAUNCHER"
remove "$LEGACY_LAUNCHER"
[ -n "$LEGACY_LAUNCHER_ALT" ] && remove "$LEGACY_LAUNCHER_ALT"
if [ "$PURGE" = true ]; then
  remove "$USER_DATA_DIR"
  [ "$LEGACY_USER_DATA_DIR" != "$USER_DATA_DIR" ] && remove "$LEGACY_USER_DATA_DIR"
else
  remove "$BUILDS_DIR"
  [ "$LEGACY_BUILDS_DIR" != "$BUILDS_DIR" ] && remove "$LEGACY_BUILDS_DIR"
fi
if [ -f "$SELFDEV_WRAPPER" ] && grep -Eq 'jcode|next-code' "$SELFDEV_WRAPPER" 2>/dev/null; then
  remove "$SELFDEV_WRAPPER"
fi

info "next-code uninstalled."
if [ "$PURGE" = false ]; then
  info "Reinstall with: curl -fsSL https://raw.githubusercontent.com/quangdang46/next-code/master/scripts/install.sh | bash"
else
  info "All next-code data wiped. Reinstall with: curl -fsSL https://raw.githubusercontent.com/quangdang46/next-code/master/scripts/install.sh | bash"
fi
