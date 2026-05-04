#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <rust-target-triple> <dist-dir>" >&2
  exit 1
fi

target_triple="$1"
dist_dir="$2"

caller_dir="$PWD"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
core_dir="$repo_root/core"
binary_name="beluna"
binary_file="$binary_name"
archive_name="beluna-core-${target_triple}.tar.gz"
checksum_name="SHA256SUMS"

case "$target_triple" in
  *windows*)
    binary_file="${binary_name}.exe"
    ;;
esac

if [[ "$dist_dir" != /* ]]; then
  dist_dir="$caller_dir/$dist_dir"
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "missing jq for Cargo metadata parsing" >&2
  exit 1
fi

mkdir -p "$dist_dir"
cd "$repo_root"

target_dir="$(cargo metadata \
  --manifest-path "$core_dir/Cargo.toml" \
  --locked \
  --format-version 1 \
  --no-deps \
  | jq -r '.target_directory')"

cargo build \
  --manifest-path "$core_dir/Cargo.toml" \
  --locked \
  --release \
  --target "$target_triple"

built_binary="$target_dir/${target_triple}/release/${binary_file}"
if [[ ! -x "$built_binary" ]]; then
  echo "expected built executable at $built_binary" >&2
  exit 1
fi

stage_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$stage_dir"
}
trap cleanup EXIT

cp "$built_binary" "$stage_dir/$binary_file"

archive_path="$dist_dir/$archive_name"
checksum_path="$dist_dir/$checksum_name"

tar -C "$stage_dir" -czf "$archive_path" "$binary_file"

if command -v shasum >/dev/null 2>&1; then
  checksum="$(shasum -a 256 "$archive_path" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
  checksum="$(sha256sum "$archive_path" | awk '{print $1}')"
else
  echo "missing shasum/sha256sum for checksum generation" >&2
  exit 1
fi

printf '%s  %s\n' "$checksum" "$archive_name" > "$checksum_path"

echo "packaged $archive_path"
echo "wrote $checksum_path"
