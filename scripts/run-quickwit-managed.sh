#!/usr/bin/env bash
set -euo pipefail

workspace="${1:-${WORKSPACE_FOLDER:-$PWD}}"
quickwit_home="$workspace/.o11y/quickwit-v0.8.2"
log_file="$workspace/logs/core/quickwit-task.log"

mkdir -p "$workspace/logs/core"
cd "$quickwit_home"

echo "__QW_BEGIN__"
./quickwit run --config ./config/quickwit.yaml >"$log_file" 2>&1 &
qw_pid=$!

cleanup() {
  if kill -0 "$qw_pid" 2>/dev/null; then
    kill "$qw_pid" 2>/dev/null || true
    wait "$qw_pid" 2>/dev/null || true
  fi
}

trap cleanup INT TERM

ready=0
for _ in $(seq 1 120); do
  if ! kill -0 "$qw_pid" 2>/dev/null; then
    echo "__QW_FAIL__"
    tail -n 30 "$log_file" || true
    wait "$qw_pid" || true
    exit 1
  fi

  if nc -z 127.0.0.1 7280 >/dev/null 2>&1; then
    ready=1
    break
  fi

  sleep 0.5
done

if [[ "$ready" -ne 1 ]]; then
  echo "__QW_FAIL__ timeout"
  tail -n 30 "$log_file" || true
  cleanup
  exit 1
fi

echo "__QW_READY__"

wait "$qw_pid"
