#!/usr/bin/env bash
# Create a PIE release archive for a built bernard_ledit PHP extension.
# Get the PHP version, thread safety, architecture and OS from PHP.
# Get the Linux libc from ldd and the Windows compiler version from PHP.
# Use these values to make the PIE asset name.
#
# Usage: pie-package.sh <extension> <tag> <output-dir> <pdfium-licenses>
#   extension        The built library: libbernard_ledit_php.so, libbernard_ledit_php.dylib
#                    or bernard_ledit_php.dll.
#   tag              The release tag. PIE uses this value in the asset name.
#   output-dir       The directory for the archive.
#   pdfium-licenses  The licenses/ directory from the static PDFium download.
#                    The archive includes these licenses.
set -euo pipefail

readonly EXTENSION_NAME="bernard_ledit"

die() {
  echo "error: $*" >&2
  exit 1
}

# Run PHP without php.ini. Send PHP warnings to stderr.
php_eval() {
  php -n -d display_errors=stderr -r "$1"
}

# Convert the path for PHP and 7-Zip on Windows.
native_path() {
  if command -v cygpath >/dev/null; then
    cygpath -w "$1"
  else
    printf '%s\n' "$1"
  fi
}

# Check that PHP loads the extension with the name that PIE uses.
check_extension_name() {
  php -n -d "extension=$(native_path "$1")" -r "exit(extension_loaded('$EXTENSION_NAME') ? 0 : 1);" ||
    die "$1 does not load as the '$EXTENSION_NAME' extension"
}

# Check that the extension version matches the release tag.
check_extension_version() {
  local reported
    reported="$(php -n -d display_errors=stderr -d "extension=$(native_path "$1")" -r "echo phpversion('$EXTENSION_NAME');")"
  [ "$reported" = "${2#v}" ] || die "$1 reports version '$reported', but the tag $2 means '${2#v}'"
}

# Get the architecture name that PIE uses: x86_64 or arm64.
architecture() {
  local machine
  machine="$(php_eval 'echo php_uname("m");')"
  case "$machine" in
    x64 | x86_64 | AMD64 | amd64) echo x86_64 ;;
    arm64 | aarch64 | ARM64) echo arm64 ;;
    *) die "unsupported architecture '$machine'" ;;
  esac
}

# Identify Linux libc as PIE does. Check if ls links to musl.
linux_libc() {
  if ldd "$(command -v ls)" 2>/dev/null | grep -q musl; then
    echo musl
  else
    echo glibc
  fi
}

# Get the Windows compiler version from PHP Extension Build.
windows_compiler() {
  local compiler
  compiler="$(php -n -i | grep '^PHP Extension Build' | grep -oE 'VS[0-9]+' || true)"
  [ -n "$compiler" ] || die "could not read the compiler from 'php -i'"
  printf '%s\n' "$compiler" | tr '[:upper:]' '[:lower:]'
}

# Create a ZIP archive. Include the subdirectories.
create_zip() {
  local archive="$1" dir="$2"
  if command -v zip >/dev/null; then
    (cd "$dir" && zip -q -r -X "$archive" ./*)
  elif command -v 7z >/dev/null; then
    (cd "$dir" && 7z a -tzip -bso0 -bsp0 "$(native_path "$archive")" ./*)
  else
    die "zip or 7z is required"
  fi
}

main() {
  [ $# -eq 4 ] || die "usage: $0 <extension> <tag> <output-dir> <pdfium-licenses>"
  local extension="$1" tag="$2" output_dir="$3" pdfium_licenses="$4"
  local repo_root php_version thread_safety arch os base debug

  [ -f "$extension" ] || die "no such file: $extension"
  [ -f "$pdfium_licenses/pdfium.txt" ] || die "no PDFium licenses in $pdfium_licenses"
  [[ "$tag" =~ ^[A-Za-z0-9._+-]+$ ]] || die "unexpected characters in tag '$tag'"
  debug="$(php_eval 'var_export(PHP_DEBUG);')"
  [ "$debug" = 0 ] || [ "$debug" = false ] ||
    die "PHP_DEBUG is '$debug': PIE expects -debug assets for a debug build of PHP"
  check_extension_name "$extension"
  check_extension_version "$extension" "$tag"

  repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
  php_version="$(php_eval 'echo PHP_MAJOR_VERSION, ".", PHP_MINOR_VERSION;')"
  thread_safety="$(php_eval 'echo PHP_ZTS ? "zts" : "nts";')"
  arch="$(architecture)"
  os="$(php_eval 'echo PHP_OS_FAMILY;')"

  staging="$(mktemp -d)"
  trap 'rm -rf "${staging:-}"' EXIT
  mkdir -p "$output_dir"
  cp "$repo_root/LICENSE" "$staging/"
  cp -R "$pdfium_licenses" "$staging/licenses"

  case "$os" in
    Windows)
      base="php_$EXTENSION_NAME-$tag-$php_version-${thread_safety#z}-$(windows_compiler)-$arch"
      base="$(printf '%s' "$base" | tr '[:upper:]' '[:lower:]')"
      cp "$extension" "$staging/$base.dll"
      if [ -f "${extension%.*}.pdb" ]; then
        cp "${extension%.*}.pdb" "$staging/$base.pdb"
      fi
      ;;
    Linux | Darwin)
      local libc=bsdlibc
      if [ "$os" = Linux ]; then
        libc="$(linux_libc)"
      fi
      base="php_$EXTENSION_NAME-${tag}_php$php_version-$arch-$os-$libc-$thread_safety"
      base="$(printf '%s' "$base" | tr '[:upper:]' '[:lower:]')"
      cp "$extension" "$staging/$EXTENSION_NAME.so"
      ;;
    *)
      die "unsupported OS family '$os'"
      ;;
  esac

  rm -f "$output_dir/$base.zip"
  create_zip "$(cd "$output_dir" && pwd)/$base.zip" "$staging"
  echo "Packaged $output_dir/$base.zip"
}

main "$@"
