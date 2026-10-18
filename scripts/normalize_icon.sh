#!/bin/sh
set -eu
icon_forge_root=${1:?"Pass the Icon Forge checkout"}
script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_directory/.." && pwd)
work=$(mktemp -d "${TMPDIR:-/tmp}/vox-icon-forge.XXXXXX")
trap 'rm -rf "$work"' EXIT
node "$icon_forge_root/forge.js" -i "$repository_root/assets/branding/app-icon-source.png" -o "$work" --skip-rembg
cp "$work/tauri/icon.png" "$repository_root/assets/branding/app-icon.png"
cp "$work/tauri/icon.icns" "$repository_root/assets/branding/app-icon.icns"
cp "$work/tauri/icon.ico" "$repository_root/assets/branding/app-icon.ico"
