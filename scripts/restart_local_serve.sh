#!/usr/bin/env bash
# Restart local jcode serve on the *current* installed binary.
# Use after scripts/install_release.sh so TUI work is not tested against a stale serve.
set -euo pipefail

current="$(readlink -f "${HOME}/.jcode/builds/current/jcode" 2>/dev/null \
  || readlink "${HOME}/.jcode/builds/current/jcode" 2>/dev/null \
  || true)"
if [[ -z "${current}" || ! -x "${current}" ]]; then
  echo "error: no executable at ~/.jcode/builds/current/jcode" >&2
  exit 1
fi

sock="${JCODE_SOCKET:-}"
if [[ -z "${sock}" ]]; then
  # Match common macOS temp default used by local serve.
  sock="${TMPDIR:-/tmp}/jcode.sock"
  sock="${sock%/}/jcode.sock"
  # Also try the path form without double jcode.sock
  if [[ ! -S "${sock}" ]]; then
    sock="${TMPDIR:-/tmp}jcode.sock"
  fi
fi

echo "binary: ${current}"
echo "socket: ${sock} (override with JCODE_SOCKET=...)"

# Kill only processes whose *executable path* is under builds/ (not this script).
while read -r pid cmd; do
  [[ -z "${pid}" ]] && continue
  case "${cmd}" in
    *"/jcode/builds/"*serve*|*"/builds/"*"jcode"*serve*)
      echo "stopping pid ${pid}"
      kill "${pid}" 2>/dev/null || true
      ;;
  esac
done < <(ps -ax -o pid=,command= 2>/dev/null | awk '/jcode/ && /serve/ {print $1, $0}')

sleep 1
# Force leftovers that still hold the socket.
if command -v lsof >/dev/null 2>&1 && [[ -S "${sock}" ]]; then
  lsof -t "${sock}" 2>/dev/null | while read -r pid; do
    echo "force-stop socket holder ${pid}"
    kill -9 "${pid}" 2>/dev/null || true
  done
fi

mkdir -p "$(dirname "${sock}")" 2>/dev/null || true
log="${TMPDIR:-/tmp}/jcode-serve-restart.log"
nohup "${current}" serve --socket "${sock}" >"${log}" 2>&1 &
new_pid=$!
sleep 1
if ! kill -0 "${new_pid}" 2>/dev/null; then
  echo "error: serve failed to start; see ${log}" >&2
  tail -20 "${log}" >&2 || true
  exit 1
fi
echo "started pid ${new_pid}"
echo "log: ${log}"
if command -v lsof >/dev/null 2>&1; then
  lsof -p "${new_pid}" 2>/dev/null | awk '/txt.*jcode/{print "mapped:", $NF; exit}'
fi
