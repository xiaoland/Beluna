#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
sweep_days="${BELUNA_CARGO_SWEEP_DAYS:-30}"

usage() {
  cat <<'USAGE'
usage: scripts/rust-storage-maintenance.sh <command>

commands:
  scan             show Rust-related local storage hot spots
  sweep-dry-run    preview age-based cargo-sweep cleanup for active and legacy targets
  sweep            run age-based cargo-sweep cleanup for active and legacy targets
  sweep-all-dry-run
                   preview cargo-sweep --all cleanup for active and legacy targets
  sweep-all        run cargo-sweep --all cleanup for active and legacy targets

environment:
  BELUNA_CARGO_SWEEP_DAYS   artifact age threshold in days, default: 30
USAGE
}

print_size() {
  local path="$1"

  if [[ -e "$path" ]]; then
    du -sh "$path"
  else
    printf '0B\t%s (missing)\n' "$path"
  fi
}

scan() {
  echo "repo: $repo_root"
  echo
  echo "Cargo target directories:"
  print_size "$repo_root/target"
  print_size "$repo_root/core/target"
  print_size "$repo_root/cli/target"
  print_size "$repo_root/moira/src-tauri/target"
  echo
  echo "Moira frontend artifacts:"
  print_size "$repo_root/moira/node_modules"
  print_size "$repo_root/moira/dist"
  echo
  echo "Cargo and Rust global state:"
  print_size "${CARGO_HOME:-$HOME/.cargo}"
  print_size "${RUSTUP_HOME:-$HOME/.rustup}"
  print_size "${SCCACHE_DIR:-$HOME/.cache/sccache}"
}

sweep_dry_run() {
  if ! cargo sweep --version >/dev/null 2>&1; then
    echo "missing cargo-sweep; install with: cargo install cargo-sweep" >&2
    exit 1
  fi

  echo "previewing active target cleanup for artifacts older than ${sweep_days} days"
  cargo sweep --recursive --dry-run --time "$sweep_days" "$repo_root"
  echo
  echo "previewing legacy nested target cleanup for artifacts older than ${sweep_days} days"
  sweep_legacy_targets 1 --time "$sweep_days"
}

sweep_by_age() {
  if ! cargo sweep --version >/dev/null 2>&1; then
    echo "missing cargo-sweep; install with: cargo install cargo-sweep" >&2
    exit 1
  fi

  echo "cleaning active target artifacts older than ${sweep_days} days"
  cargo sweep --recursive --time "$sweep_days" "$repo_root"
  echo
  echo "cleaning legacy nested target artifacts older than ${sweep_days} days"
  sweep_legacy_targets 0 --time "$sweep_days"
}

sweep_all_dry_run() {
  if ! cargo sweep --version >/dev/null 2>&1; then
    echo "missing cargo-sweep; install with: cargo install cargo-sweep" >&2
    exit 1
  fi

  echo "previewing cargo-sweep --all cleanup of active target"
  cargo sweep --recursive --dry-run --all "$repo_root"
  echo
  echo "previewing cargo-sweep --all cleanup of legacy nested targets"
  sweep_legacy_targets 1 --all
}

sweep_all() {
  if ! cargo sweep --version >/dev/null 2>&1; then
    echo "missing cargo-sweep; install with: cargo install cargo-sweep" >&2
    exit 1
  fi

  echo "cleaning cargo-sweep --all artifacts from active target"
  cargo sweep --recursive --all "$repo_root"
  echo
  echo "cleaning cargo-sweep --all artifacts from legacy nested targets"
  sweep_legacy_targets 0 --all
}

sweep_legacy_targets() {
  local dry_run="$1"
  shift
  local args=("$@")
  local projects=(
    "$repo_root/core"
    "$repo_root/cli"
    "$repo_root/moira/src-tauri"
  )
  local project
  local target_dir

  for project in "${projects[@]}"; do
    target_dir="$project/target"
    if [[ -d "$target_dir" ]]; then
      echo "$target_dir"
      if [[ "$dry_run" == "1" ]]; then
        CARGO_TARGET_DIR="$target_dir" cargo sweep --dry-run "${args[@]}" "$project"
      else
        CARGO_TARGET_DIR="$target_dir" cargo sweep "${args[@]}" "$project"
      fi
    fi
  done
}

command="${1:-scan}"

case "$command" in
  scan)
    scan
    ;;
  sweep-dry-run)
    sweep_dry_run
    ;;
  sweep)
    sweep_by_age
    ;;
  sweep-all-dry-run|reset-dry-run)
    sweep_all_dry_run
    ;;
  sweep-all|reset)
    sweep_all
    ;;
  -h|--help|help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
