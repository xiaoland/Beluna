#!/usr/bin/env bash
set -euo pipefail

workspace="${WORKSPACE_FOLDER:-$PWD}"

if pgrep -x quickwit >/dev/null 2>&1; then
  echo "quickwit is already running"
  echo "__QW_READY__"
  exit 0
fi

echo "quickwit is not running."
read -r -p "Start quickwit now? [y/N] " reply

if [[ "$reply" =~ ^[Yy]$ ]]; then
  echo "quickwit starting"
  exec /usr/bin/env bash "$workspace/scripts/run-quickwit-managed.sh" "$workspace"
fi

echo "skip starting quickwit"
echo "__QW_READY__"
