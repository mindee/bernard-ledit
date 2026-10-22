#!/usr/bin/env bash
# Builds a static, JPEG-namespaced PDFium library for one target with bblanchon/pdfium-binaries:
# inside an ubuntu:22.04 container on Linux, natively on macOS and Windows.
#
# Environment (all optional unless stated):
#   PDFIUM_OS       target OS: linux (default), mac or win
#   PDFIUM_CPU      target CPU: x64 (default) or arm64
#   PDFIUM_ENV      "musl" for a Linux musl build, empty otherwise
#   PDFIUM_TAG      pdfium-binaries tag; default: the contents of core/pdfium-release
#   MSVC_TOOLSET    required on Windows: MSVC toolset to compile with, e.g. 14.29
#   WORK_DIR        checkout and build directory; default: /tmp/pdfium-static
#   OUT_DIR         output directory for the namespaced library;
#                   default: target/pdfium-static/<tag>/<os>-<cpu>[-<env>]
#   MUSL_URL        mirror of the musl cross toolchains (musl.cc is unreliable)
#   SKIP_NAMESPACE  set to 1 to stop after the raw PDFium build
set -euo pipefail

readonly PDFIUM_BINARIES_URL="https://github.com/bblanchon/pdfium-binaries.git"
readonly DEFAULT_MUSL_URL="https://github.com/mindee/bernard-ledit/releases/download/musl-toolchains-gcc11.2.1-musl1.2.2-gb76f37fd"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"
namespacer="$script_dir/namespace_pdfium_jpeg.py"

PDFIUM_OS="${PDFIUM_OS:-linux}"
PDFIUM_CPU="${PDFIUM_CPU:-x64}"
PDFIUM_ENV="${PDFIUM_ENV:-}"
PDFIUM_TAG="${PDFIUM_TAG:-$(tr -d '[:space:]' < "$repo_root/core/pdfium-release")}"
WORK_DIR="${WORK_DIR:-/tmp/pdfium-static}"
OUT_DIR="${OUT_DIR:-$repo_root/target/pdfium-static/${PDFIUM_TAG//\//_}/$PDFIUM_OS-$PDFIUM_CPU${PDFIUM_ENV:+-$PDFIUM_ENV}}"

die() {
  echo "error: $*" >&2
  exit 1
}

# In-place sed for both GNU sed (Linux, Git Bash) and BSD sed (macOS).
sedi() {
  local file="${*: -1}"
  sed -i.bak "$@" && rm -f "$file.bak"
}

# Removes a directory, using sudo for files left behind by the root-owned build container.
remove_dir() {
  rm -rf "$1" 2>/dev/null || sudo rm -rf "$1"
}

# Checks the target and the tools before the long build, and sets PYTHON.
# Windows ships a python3 stub that opens the Microsoft Store, so it must use `python`.
validate_inputs() {
  case "$PDFIUM_OS-$PDFIUM_ENV" in
    linux- | linux-musl | mac- | win-) ;;
    *) die "unsupported target: PDFIUM_OS='$PDFIUM_OS', PDFIUM_ENV='$PDFIUM_ENV'" ;;
  esac
  [ -f "$namespacer" ] || die "missing $namespacer"
  if [ "$PDFIUM_OS" = linux ]; then
    command -v docker >/dev/null || die "docker is required"
  fi
  if [ "$PDFIUM_OS" = win ]; then
    [ -n "${MSVC_TOOLSET:-}" ] || die "MSVC_TOOLSET is required on Windows"
    PYTHON="$(command -v python || true)"
  else
    PYTHON="$(command -v python3 || command -v python || true)"
  fi
  [ -n "$PYTHON" ] || die "python is required"
}

# Checks out the pdfium-binaries tag matching the PDFium release: its patches target that exact
# PDFium revision. Initialises in place because a restored source cache may already contain pdfium/,
# and `checkout -f` discards our patches from a previous run. Leftovers from a failed gclient sync
# (_bad_scm) would break the next sync.
checkout_pdfium_binaries() {
  mkdir -p "$binaries"
  if [ ! -d "$binaries/.git" ]; then
    git -C "$binaries" init --quiet
    git -C "$binaries" remote add origin "$PDFIUM_BINARIES_URL"
  fi
  git -C "$binaries" fetch --tags --quiet origin
  git -C "$binaries" checkout --quiet -f "$PDFIUM_TAG"
  remove_dir "$binaries/_bad_scm"
}

# Lowers gclient's parallel fetches (googlesource answers CI bursts with HTTP 503), and drops the
# self-test and packaging steps: the library we need is already staged by then.
patch_build_steps() {
  sedi 's/gclient sync /gclient sync --jobs 4 /' "$binaries/steps/02-checkout.sh"
  sedi '/09-test.sh/d; /10-pack.sh/d' "$binaries/build.sh"
}

# Windows: appends two patches to pdfium-binaries' patch step. pdfium-binaries' checkout step
# resets these files beforehand, so the patches always apply to pristine sources.
# The first links against the DLL C runtime (/MD), like Rust and PHP, instead of Chromium's static
# runtime (/MT). The second compiles with MSVC_TOOLSET through vcvarsall.bat's -vcvars_ver, so the
# library links with the toolset of the PHP versions it targets.
patch_windows_toolchain() {
  local crt_gn="$binaries/pdfium/build/config/win/BUILD.gn"
  local toolchain_py="$binaries/pdfium/build/toolchain/win/setup_toolchain.py"
  cat >> "$binaries/steps/03-patch.sh" <<EOF

# bernard-ledit: link against the DLL C runtime (/MD)
grep -q 'configs = \[ ":static_crt" \]' "$crt_gn" || { echo "static CRT config not found in $crt_gn" >&2; exit 1; }
sed -i 's/configs = \[ ":static_crt" \]/configs = [ ":dynamic_crt" ]/' "$crt_gn"

# bernard-ledit: compile with MSVC toolset $MSVC_TOOLSET
sed -i "/def _LoadEnvFromBat/!s/_LoadEnvFromBat(args)/_LoadEnvFromBat(args + ['-vcvars_ver=$MSVC_TOOLSET'])/" "$toolchain_py"
grep -q "vcvars_ver=$MSVC_TOOLSET" "$toolchain_py" || { echo "vcvarsall call not found in $toolchain_py" >&2; exit 1; }
EOF
}

# Runs pdfium-binaries' build on the host (macOS, Windows).
# Flags must precede the positional arguments: build.sh parses them with getopts.
build_natively() {
  (cd "$binaries" && ./build.sh -s -b "$PDFIUM_TAG" "$PDFIUM_OS" "$PDFIUM_CPU")
}

# Runs pdfium-binaries' build inside ubuntu:22.04, whose glibc 2.35 becomes the oldest glibc the
# extension supports. Debconf is preseeded so that no package install opens a dialog, including
# those run through sudo, which drops DEBIAN_FRONTEND. PDFIUM_ENV, when set, is build.sh's optional
# third positional argument.
build_in_container() {
  docker run --rm -v "$binaries:/w" -w /w \
    -e PDFIUM_TAG="$PDFIUM_TAG" -e PDFIUM_OS="$PDFIUM_OS" -e PDFIUM_CPU="$PDFIUM_CPU" \
    -e PDFIUM_ENV="$PDFIUM_ENV" -e MUSL_URL="${MUSL_URL:-$DEFAULT_MUSL_URL}" \
    ubuntu:22.04 bash -eu -c '
      export DEBIAN_FRONTEND=noninteractive
      echo "debconf debconf/frontend select Noninteractive" | debconf-set-selections
      apt-get update
      apt-get install -y git curl python3 lsb-release sudo build-essential pkg-config file
      ./build.sh -s -b "$PDFIUM_TAG" "$PDFIUM_OS" "$PDFIUM_CPU" $PDFIUM_ENV
    '
}

# Windows: fails if the library still requests the static C runtime, i.e. the /MD patch didn't apply.
verify_windows_crt() {
  local lib="$binaries/pdfium/out/obj/pdfium.lib"
  [ -f "$lib" ] || die "the PDFium build produced no $lib"
  if grep -Eaqi 'defaultlib:"?libcmt' "$lib"; then
    die "pdfium.lib requests the static C runtime (libcmt); the /MD patch did not apply"
  fi
}

# Windows: fetches Chromium's llvm-nm, which matches the compiler that built the objects. The
# namespacer falls back to the LLVM on PATH if this download fails.
fetch_windows_llvm_tools() {
  (cd "$binaries/pdfium" && "$PYTHON" tools/clang/scripts/update.py --package=objdump) ||
    echo "warning: could not fetch Chromium's LLVM tools; the namespacer will use the LLVM on PATH" >&2
}

main() {
  validate_inputs
  mkdir -p "$WORK_DIR"
  binaries="$(cd "$WORK_DIR" && pwd)/pdfium-binaries"

  checkout_pdfium_binaries
  patch_build_steps
  case "$PDFIUM_OS" in
    linux)
      build_in_container
      ;;
    mac)
      build_natively
      ;;
    win)
      patch_windows_toolchain
      build_natively
      verify_windows_crt
      fetch_windows_llvm_tools
      ;;
  esac

  if [ "${SKIP_NAMESPACE:-0}" = 1 ]; then
    echo "Raw PDFium build left in $binaries"
    return
  fi
  mkdir -p "$OUT_DIR"
  "$PYTHON" "$namespacer" "$binaries" "$OUT_DIR"
}

main "$@"
