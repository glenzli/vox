#!/bin/sh
set -eu
script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_directory/.." && pwd)
local_build_root=${VOX_LOCAL_BUILD_ROOT:-"$(dirname -- "$repository_root")/.vox-local-build"}
application="$local_build_root/current-debug/Vox.app"
test -x "$application/Contents/MacOS/Vox" || { echo "Build Vox first with scripts/build_and_promote_debug.sh" >&2; exit 69; }
if [ "${1:-}" = "--check" ];then echo "$application";exit 0;fi
if [ "$#" -gt 0 ];then open -n "$application" --args --project "$1";else open -n "$application";fi
