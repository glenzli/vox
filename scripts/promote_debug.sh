#!/bin/sh
set -eu
script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_directory/.." && pwd)
local_build_root=${VOX_LOCAL_BUILD_ROOT:-"$(dirname -- "$repository_root")/.vox-local-build"}
source_app=${1:?"Pass the already validated Vox.app"}
test -x "$source_app/Contents/MacOS/Vox"
test -x "$source_app/Contents/MacOS/vox-cli"
case "$local_build_root" in "$repository_root"|"$repository_root"/*) exit 64;; esac
stamp=$(date +%Y%m%d-%H%M%S)-$$
release="$local_build_root/releases/$stamp"
mkdir -p "$local_build_root/releases"
mkdir "$release"
ditto "$source_app" "$release/Vox.app"
codesign --force --deep --sign - "$release/Vox.app"
plutil -lint "$release/Vox.app/Contents/Info.plist"
codesign --verify --deep --strict "$release/Vox.app"
ln -s "$release" "$local_build_root/.current-debug-$stamp"
mv -fh "$local_build_root/.current-debug-$stamp" "$local_build_root/current-debug"
echo "Vox debug app: $local_build_root/current-debug/Vox.app"
