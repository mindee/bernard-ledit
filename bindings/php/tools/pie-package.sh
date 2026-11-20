#!/usr/bin/env bash
# Packages a built bernard_ledit extension as the release asset PIE downloads for the PHP that runs
# this script. The PHP version, thread safety, architecture, OS, libc and, on Windows, compiler are
# read from that PHP the way PIE reads them, so the asset name matches what PIE looks for.
#
# Usage: pie-package.sh <extension> <tag> <output-dir> <libheif-prefix> <pdfium-prefix>
#   extension        the built library: libbernard_ledit_php.so, .dylib, or bernard_ledit_php.dll
#   tag              the release tag, exactly as written: PIE names assets after it
#   output-dir       where to write the archive
#   libheif-prefix   installation of codec-free libheif 1.21.1
#   pdfium-prefix    installation of the pinned PDFium release
set -euo pipefail

readonly EXTENSION_NAME="bernard_ledit"

die() {
  echo "error: $*" >&2
  exit 1
}

# Runs a PHP snippet without php.ini and prints its output. PHP's own warnings go to stderr, so they
# can't end up in the output.
php_eval() {
  php -n -d display_errors=stderr -r "$1"
}

# Prints a path in the form PHP and 7-Zip accept: Windows programs need Windows paths in Git Bash.
native_path() {
  if command -v cygpath >/dev/null; then
    cygpath -w "$1"
  else
    printf '%s\n' "$1"
  fi
}

# Fails unless PHP loads the extension under the name PIE checks after installing it.
check_extension_name() {
  php -n -d "extension=$(native_path "$1")" -r "exit(extension_loaded('$EXTENSION_NAME') ? 0 : 1);" ||
    die "$1 does not load as the '$EXTENSION_NAME' extension"
}

# Fails unless the extension reports the release's version: the one Composer compares
# ext-bernard_ledit constraints against.
check_extension_version() {
  local reported
    reported="$(php -n -d display_errors=stderr -d "extension=$(native_path "$1")" -r "echo phpversion('$EXTENSION_NAME');")"
  [ "$reported" = "${2#v}" ] || die "$1 reports version '$reported', but the tag $2 means '${2#v}'"
}

# Prints the architecture under PIE's name for it: x86_64 or arm64.
architecture() {
  local machine
  machine="$(php_eval 'echo php_uname("m");')"
  case "$machine" in
    x64 | x86_64 | AMD64 | amd64) echo x86_64 ;;
    arm64 | aarch64 | ARM64) echo arm64 ;;
    *) die "unsupported architecture '$machine'" ;;
  esac
}

# Prints the Linux libc as PIE detects it: musl when `ls` links against musl, glibc otherwise.
linux_libc() {
  if ldd "$(command -v ls)" 2>/dev/null | grep -q musl; then
    echo musl
  else
    echo glibc
  fi
}

# Prints the compiler PHP was built with on Windows (vs16, vs17), from "PHP Extension Build".
windows_compiler() {
  local compiler
  compiler="$(php -n -i | grep '^PHP Extension Build' | grep -oE 'VS[0-9]+' || true)"
  [ -n "$compiler" ] || die "could not read the compiler from 'php -i'"
  printf '%s\n' "$compiler" | tr '[:upper:]' '[:lower:]'
}

check_glibc_baseline() {
  local binary="$1" symbols newest
  symbols="$(objdump -T "$binary")" || die "could not inspect $binary"
  newest="$(printf '%s\n' "$symbols" | grep -oE 'GLIBC_[0-9]+(\.[0-9]+)*' | sed 's/GLIBC_//' | sort -Vu | tail -n 1 || true)"
  [ -n "$newest" ] || die "could not determine the glibc requirement of $binary"
  [ "$(printf '%s\n' "$newest" 2.35 | sort -V | tail -n 1)" = 2.35 ] ||
    die "$binary requires glibc $newest, newer than 2.35"
}

# Zips the contents of a directory, keeping its subdirectories, into the archive named first.
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
  [ $# -eq 5 ] || die "usage: $0 <extension> <tag> <output-dir> <libheif-prefix> <pdfium-prefix>"
  local extension="$1" tag="$2" output_dir="$3" heif="$4" pdfium="$5"
  local repo_root php_version thread_safety arch os base debug bundled_extension python

  [ -f "$extension" ] || die "no such file: $extension"
  [[ "$tag" =~ ^[A-Za-z0-9._+-]+$ ]] || die "unexpected characters in tag '$tag'"
  debug="$(php_eval 'var_export(PHP_DEBUG);')"
  [ "$debug" = 0 ] || [ "$debug" = false ] ||
    die "PHP_DEBUG is '$debug': PIE expects -debug assets for a debug build of PHP"
  repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
  php_version="$(php_eval 'echo PHP_MAJOR_VERSION, ".", PHP_MINOR_VERSION;')"
  thread_safety="$(php_eval 'echo PHP_ZTS ? "zts" : "nts";')"
  arch="$(architecture)"
  os="$(php_eval 'echo PHP_OS_FAMILY;')"

  staging="$(mktemp -d)"
  trap 'rm -rf "${staging:-}"' EXIT
  mkdir -p "$output_dir"
  cp "$repo_root/LICENSE" "$staging/"

  case "$os" in
    Windows)
      base="php_$EXTENSION_NAME-$tag-$php_version-${thread_safety#z}-$(windows_compiler)-$arch"
      base="$(printf '%s' "$base" | tr '[:upper:]' '[:lower:]')"
      cp "$extension" "$staging/$base.dll"
      bundled_extension="$staging/$base.dll"
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
      bundled_extension="$staging/$EXTENSION_NAME.so"
      ;;
    *)
      die "unsupported OS family '$os'"
      ;;
  esac

  if [ "$os" = Linux ] && [ "$libc" = musl ]; then
    die "musl/Alpine PHP release packages are unsupported; use a glibc-based distribution"
  fi

  if [ "$os" = Windows ]; then
    python=python
  else
    python=python3
  fi
  "$python" "$repo_root/tools/bundle_native.py" "$bundled_extension" "$heif" "$pdfium" --windows-flat
  if [ "$os" = Linux ] && [ "$libc" = glibc ]; then
    check_glibc_baseline "$bundled_extension"
    check_glibc_baseline "$staging/native/libheif.so.1"
    check_glibc_baseline "$staging/native/libpdfium.so"
  fi
  if [ "$os" = Windows ]; then
    export PATH="$staging:$PATH"
  else
    unset LD_LIBRARY_PATH DYLD_LIBRARY_PATH
  fi
  check_extension_name "$bundled_extension"
  check_extension_version "$bundled_extension" "$tag"

  rm -f "$output_dir/$base.zip"
  create_zip "$(cd "$output_dir" && pwd)/$base.zip" "$staging"
  echo "Packaged $output_dir/$base.zip"
}

main "$@"
