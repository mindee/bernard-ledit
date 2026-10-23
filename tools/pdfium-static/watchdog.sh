#!/usr/bin/env bash
# Watches the log of a running build. Once the log has stayed the same size for <seconds>, prints
# the processes started since the watchdog did, so the job log shows what the build is stuck on,
# then kills Ninja and everything it started, so the build fails instead of hanging until the
# job's time limit. Keeps watching afterwards, in case the build is stuck outside Ninja.
#
# Usage: watchdog.sh <log> <seconds> &
#   log      the file the build's output goes to
#   seconds  how long the log may stay unchanged
# Run it in the background, and kill it once the build is over.
set -uo pipefail

readonly POLL_SECONDS="${WATCHDOG_POLL_SECONDS:-60}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

# Prints the processes started since the given UTC time (on Linux and macOS, all of them but the
# kernel's), with their CPU time: a stuck process whose CPU time keeps rising loops, one whose CPU
# time doesn't waits, e.g. on an error dialog.
list_processes() {
  if [ "${OS:-}" = Windows_NT ]; then
    powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$script_dir/list-processes.ps1")" -Since "$1"
  else
    ps -eo pid,ppid,etime,time,args | awk 'NR == 1 || ($1 != 2 && $2 != 2)'
  fi
}

# Kills Ninja and the commands it runs; the Linux build runs them as root, in its container.
kill_ninja() {
  if [ "${OS:-}" = Windows_NT ]; then
    taskkill //F //T //IM ninja.exe
  else
    sudo -n pkill -x ninja 2>/dev/null || pkill -x ninja
  fi
}

main() {
  [ $# -eq 2 ] || { echo "usage: $0 <log> <seconds>" >&2; exit 1; }
  local log="$1" limit="$2" started size last_size="" unchanged=0
  started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  # sleep's output goes nowhere: a sleep outliving this script would otherwise hold the job log open.
  while sleep "$POLL_SECONDS" > /dev/null 2>&1; do
    size="$(wc -c < "$log" 2>/dev/null | tr -d ' ')"
    if [ "$size" != "$last_size" ]; then
      last_size="$size"
      unchanged=0
      continue
    fi
    unchanged=$((unchanged + POLL_SECONDS))
    if [ "$unchanged" -ge "$limit" ]; then
      echo "::error::The build printed nothing for $((unchanged / 60)) minutes. The processes started since it began:"
      list_processes "$started"
      echo "Killing Ninja and the commands it runs."
      kill_ninja || true
      unchanged=0
    fi
  done
}

main "$@"
