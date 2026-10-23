#!/usr/bin/env bash
# Downloads the static PDFium library of one target from the repository's pdfium-static-<release>
# GitHub release, with the licenses of PDFium and of the code built into it. Checks both against
# their published SHA-256, saves the library under the name the linker looks for (libpdfium.a, or
# pdfium.lib for Windows targets) and the licenses as <directory>/licenses/.
#
# Usage: download.sh <target> <directory>
#   target     a release asset target, e.g. linux-x64-glibc, mac-arm64, win-x64-v143
#   directory  where to save them; pass it to the build as PDFIUM_STATIC_LIB_PATH
#
# Run it from inside the repository, with an authenticated `gh` (GH_TOKEN in CI).
set -euo pipefail

die() {
  echo "error: $*" >&2
  exit 1
}

# Prints the SHA-256 of a file with whichever tool the platform has (macOS lacks sha256sum).
# The file is read from stdin: given a name containing a backslash, as Windows paths do,
# sha256sum escapes it and prefixes the hash with a backslash.
sha256_of() {
  if command -v sha256sum >/dev/null; then
    sha256sum < "$1" | cut -d' ' -f1
  else
    shasum -a 256 < "$1" | cut -d' ' -f1
  fi
}

# Fails unless the downloaded asset matches its published .sha256, then removes the .sha256.
verify() {
  [ "$(sha256_of "$1")" = "$(cut -d' ' -f1 "$1.sha256")" ] ||
    die "$(basename "$1") does not match its published SHA-256"
  rm -f "$1.sha256"
}

main() {
  [ $# -eq 2 ] || die "usage: $0 <target> <directory>"
  local target="$1" dir="$2"
  local repo_root release asset linked_name licenses

  repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
  release="pdfium-static-$(tr -d '[:space:]' < "$repo_root/core/pdfium-release")"
  release="${release//\//_}"
  case "$target" in
    win-*) asset="libpdfium-$target.lib" linked_name="pdfium.lib" ;;
    *) asset="libpdfium-$target.a" linked_name="libpdfium.a" ;;
  esac
  licenses="libpdfium-$target-licenses.tar.gz"

  mkdir -p "$dir"
  gh release download "$release" --dir "$dir" --clobber \
    --pattern "$asset" --pattern "$asset.sha256" --pattern "$licenses" --pattern "$licenses.sha256"
  verify "$dir/$asset"
  verify "$dir/$licenses"

  mv -f "$dir/$asset" "$dir/$linked_name"
  rm -rf "$dir/licenses"
  (cd "$dir" && tar -xzf "$licenses" && rm -f "$licenses")
  echo "Saved $asset from $release as $dir/$linked_name, and its licenses in $dir/licenses"
}

main "$@"
