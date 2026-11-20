#!/usr/bin/env bash
# Download the pinned shared PDFium build for CI or local development.
set -euo pipefail

[ "$#" -eq 2 ] || { echo "usage: $0 <target> <directory>" >&2; exit 1; }
target="$1"
destination="$2"
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
release="$(tr -d '[:space:]' < "$repo_root/core/pdfium-release")"
mkdir -p "$destination"
curl -fsSL "https://github.com/bblanchon/pdfium-binaries/releases/download/${release//\//%2F}/pdfium-$target.tgz" | tar -xz -C "$destination"

case "$target" in
  win-*)
    cp "$destination/lib/pdfium.dll.lib" "$destination/lib/pdfium.lib"
    ;;
  mac-*)
    install_name_tool -id @rpath/libpdfium.dylib "$destination/lib/libpdfium.dylib"
    ;;
esac
