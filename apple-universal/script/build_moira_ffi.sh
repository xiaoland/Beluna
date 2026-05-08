#!/usr/bin/env bash
set -euo pipefail

if [[ "${PLATFORM_NAME:-}" != "macosx" ]]; then
    echo "Moira FFI: skipping for PLATFORM_NAME=${PLATFORM_NAME:-unknown}"
    exit 0
fi

project_dir="${PROJECT_DIR:?PROJECT_DIR is required}"
target_build_dir="${TARGET_BUILD_DIR:?TARGET_BUILD_DIR is required}"
frameworks_folder_path="${FRAMEWORKS_FOLDER_PATH:?FRAMEWORKS_FOLDER_PATH is required}"
configuration="${CONFIGURATION:-Debug}"

repo_root="$(cd "${project_dir}/.." && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo is required to build Moira FFI" >&2
    exit 1
fi

cargo_args=(build -p moira-ffi --lib --locked)
cargo_profile="debug"
if [[ "${configuration}" == "Release" ]]; then
    cargo_args+=(--release)
    cargo_profile="release"
fi

echo "Moira FFI: building ${cargo_profile} dylib"
(
    cd "${repo_root}"
    cargo "${cargo_args[@]}"
)

destination_dir="${target_build_dir}/${frameworks_folder_path}"
source_dylib="${repo_root}/target/${cargo_profile}/libmoira_ffi.dylib"
source_duckdb_dylib="${repo_root}/target/${cargo_profile}/deps/libduckdb.dylib"

if [[ ! -f "${source_dylib}" ]]; then
    echo "error: Moira FFI dylib was not produced at ${source_dylib}" >&2
    exit 1
fi

if [[ ! -f "${source_duckdb_dylib}" ]]; then
    echo "error: DuckDB dylib was not produced at ${source_duckdb_dylib}" >&2
    exit 1
fi

mkdir -p "${destination_dir}"

sign_dylib() {
    local dylib_path="$1"

    if [[ "${CODE_SIGNING_ALLOWED:-NO}" != "YES" ]]; then
        return
    fi

    echo "Moira FFI: signing $(basename "${dylib_path}") with identity ${code_sign_identity}"
    codesign --force --sign "${code_sign_identity}" --timestamp=none "${dylib_path}"
}

code_sign_identity="${EXPANDED_CODE_SIGN_IDENTITY:-}"
if [[ -z "${code_sign_identity}" ]]; then
    code_sign_identity="-"
fi

destination_moira_dylib="${destination_dir}/libmoira_ffi.dylib"
destination_duckdb_dylib="${destination_dir}/libduckdb.dylib"

cp "${source_dylib}" "${destination_moira_dylib}"
install_name_tool -id "@rpath/libmoira_ffi.dylib" "${destination_moira_dylib}"
sign_dylib "${destination_moira_dylib}"

cp "${source_duckdb_dylib}" "${destination_duckdb_dylib}"
sign_dylib "${destination_duckdb_dylib}"

echo "Moira FFI: bundled ${destination_moira_dylib}"
echo "Moira FFI: bundled ${destination_duckdb_dylib}"
