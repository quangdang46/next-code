#!/usr/bin/env bash
# Restart local next-code serve on the *current* installed binary.
# Use after scripts/install_release.sh so TUI work is not tested against a stale serve.
set -euo pipefail

# Locate the currently installed next-code binary.
current=""
for candidate in \
  "${HOME}/.next-code/builds/current/next-code"; do
  if [[ -e "${candidate}" ]]; then
    resolved="$(readlink -f "${candidate}" 2>/dev/null \
      || readlink "${candidate}" 2>/dev/null \
      || printf '%s' "${candidate}")"
    if [[ -n "${resolved}" && -x "${resolved}" ]]; then
      current="${resolved}"
      break
    fi
  fi
done

if [[ -z "${current}" || ! -x "${current}" ]]; then
  if command -v next-code >/dev/null 2>&1; then
    current="$(command -v next-code)"
  else
    echo "error: no executable at ~/.next-code/builds/current/next-code" >&2
    exit 1
  fi
fi

sock="${NEXT_CODE_SOCKET:-}"
if [[ -z "${sock}" ]]; then
  # Match common macOS temp default used by local serve.
  base="${TMPDIR:-/tmp}"
  base="${base%/}"
  for try in \
    "${base}/next-code.sock" \
    "${base}/next-code.sock"; do
    if [[ -S "${try}" ]]; then
      sock="${try}"
      break
    fi
  done
  if [[ -z "${sock}" ]]; then
    sock="${base}/next-code.sock"
  fi
fi

echo "binary: ${current}"
echo "socket: ${sock} (override with NEXT_CODE_SOCKET=...)"

# Kill only processes whose *executable path* is under builds/ (not this script).
while read -r pid cmd; do
  [[ -z "${pid}" ]] && continue
  case "${cmd}" in
    */next-code/builds/*serve*|*"/builds/"*"next-code"*serve*)
      echo "stopping pid ${pid}"
      kill "${pid}" 2>/dev/null || true
      ;;
  esac
done < <(ps -ax -o pid=,command= 2>/dev/null | awk '/next-code/ && /serve/ {print $1, $0}')

sleep 1
# Force leftovers that still hold the socket.
if command -v lsof >/dev/null 2>&1 && [[ -S "${sock}" ]]; then
  lsof -t "${sock}" 2>/dev/null | while read -r pid; do
    echo "force-stop socket holder ${pid}"
    kill -9 "${pid}" 2>/dev/null || true
  done
fi

mkdir -p "$(dirname "${sock}")" 2>/dev/null || true
log="${TMPDIR:-/tmp}/next-code-serve-restart.log"
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
  lsof -p "${new_pid}" 2>/dev/null | awk '/txt.*next-code/{print "mapped:", $NF; exit}'
fi
