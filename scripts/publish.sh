#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
manifest_path="${repo_root}/Cargo.toml"

mode="dry-run"
package_args=()
publish_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --execute)
      mode="execute"
      ;;
    --allow-dirty)
      package_args+=("$1")
      publish_args+=("$1")
      ;;
    *)
      publish_args+=("$1")
      ;;
  esac
  shift
done

cd "${repo_root}"

echo "Packaging crate..."
cargo package --locked --manifest-path "${manifest_path}" "${package_args[@]}"

if [[ "${mode}" == "dry-run" ]]; then
  echo "Running cargo publish --dry-run..."
  cargo publish --dry-run --locked --manifest-path "${manifest_path}" "${publish_args[@]}"
else
  echo "Publishing crate..."
  cargo publish --locked --manifest-path "${manifest_path}" "${publish_args[@]}"
fi
