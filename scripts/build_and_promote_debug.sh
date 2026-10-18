#!/bin/sh
set -eu
script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_directory/.." && pwd)
local_build_root=${VOX_LOCAL_BUILD_ROOT:-"$(dirname -- "$repository_root")/.vox-local-build"}
build_directory=${VOX_BUILD_DIR:-"$local_build_root/build"}
cargo_directory=${VOX_CARGO_TARGET_DIR:-"$local_build_root/cargo"}
case "$local_build_root" in "$repository_root"|"$repository_root"/*) echo "Build output must stay outside the checkout" >&2; exit 64;; esac
cd "$repository_root"
python3 scripts/check_translations.py
CARGO_TARGET_DIR="$cargo_directory" cargo fmt --all -- --check
CARGO_TARGET_DIR="$cargo_directory" cargo clippy --offline --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR="$cargo_directory" cargo test --offline --workspace
cmake --preset desktop-dev -B "$build_directory" -DVOX_CARGO_TARGET_DIR="$cargo_directory"
cmake --build "$build_directory" --parallel "${VOX_BUILD_JOBS:-6}"
ctest --test-dir "$build_directory" --output-on-failure
"$script_directory/promote_debug.sh" "$build_directory/Vox.app"
