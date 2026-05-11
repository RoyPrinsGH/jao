#!/usr/bin/env bash
set -euo pipefail

toolchain_args=()

while (($# > 0)); do
	case "$1" in
		--nightly)
			toolchain_args=(+nightly)
			shift
			;;
		*)
			echo "usage: ${0##*/} [--nightly]" >&2
			exit 2
			;;
	esac
done

cargo "${toolchain_args[@]}" fmt --manifest-path ../Cargo.toml