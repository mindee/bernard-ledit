#!/usr/bin/env bash
# Downloads the static PDFium library of one target from the repository's pdfium-static-<release>
# GitHub release, checks it against its published SHA-256, and saves it under the name the linker
# looks for: libpdfium.a, or pdfium.lib for Windows targets.
#
# Usage: download.sh <target> <directory>
#   target     a release asset target, e.g. linux-x64-glibc, mac-arm64, win-x64-v143
#   directory  where to save the library; pass it to the build as PDFIUM_STATIC_LIB_PATH
#
# Run it from inside the repository, with an authenticated `gh` (GH_TOKEN in CI).
set -euo pipefail

die() {
  echo "error: $*" >&2
  exit 1
}

# Prints the SHA-256 of a file with whichever tool the platform has (macOS lacks sha256sum).
sha256_of() {
  if command -v sha256sum >/dev/null; then
    sha256sum < "$1" | cut -d' ' -f1
  else
    shasum -a 256 < "$1" | cut -d' ' -f1
  fi
}

main() {
  [ $# -eq 2 ] || die "usage: $0 <target> <directory>"
  local target="$1" dir="$2"
  local repo_root release asset linked_name

  repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
  release="pdfium-static-$(tr -d '[:space:]' < "$repo_root/core/pdfium-release")"
  release="${release//\//_}"
  case "$target" in
    win-*) asset="libpdfium-$target.lib" linked_name="pdfium.lib" ;;
    *) asset="libpdfium-$target.a" linked_name="libpdfium.a" ;;
  esac

  mkdir -p "$dir"
  gh release download "$release" --pattern "$asset" --pattern "$asset.sha256" --dir "$dir" --clobber
  [ "$(sha256_of "$dir/$asset")" = "$(cut -d' ' -f1 "$dir/$asset.sha256")" ] ||
    die "$asset does not match its published SHA-256"
  mv -f "$dir/$asset" "$dir/$linked_name"
  rm -f "$dir/$asset.sha256"
  echo "Saved $asset from $release as $dir/$linked_name"
}

main "$@"
