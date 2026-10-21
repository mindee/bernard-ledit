#!/usr/bin/env bash
# Builds a static, JPEG-namespaced PDFium library with bblanchon/pdfium-binaries:
# inside Docker on Linux, natively on macOS and Windows.
#
# Env (all optional):
#   PDFIUM_ENV   PDFium environment.
#   PDFIUM_TAG   pdfium-binaries tag, default: contents of core/pdfium-release (e.g. chromium/8076)
#   PDFIUM_OS    build.sh os,  default: linux
#   PDFIUM_CPU   build.sh cpu, default: x64
#   OUT_DIR      namespaced output, default: target/pdfium-static/<tag>/<os>-<cpu>
#   WORK_DIR     pdfium-binaries checkout + pdfium sources, default: /tmp/pdfium-static
#   SKIP_NAMESPACE=1  stop after the raw build (for iterating on the namespacer)
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"

PDFIUM_TAG="${PDFIUM_TAG:-$(tr -d '[:space:]' < "$repo_root/core/pdfium-release")}"
PDFIUM_OS="${PDFIUM_OS:-linux}"
PDFIUM_CPU="${PDFIUM_CPU:-x64}"
PDFIUM_ENV="${PDFIUM_ENV:-}"   # "musl" for musl builds, empty otherwise
sedi() { local f="${*: -1}"; sed -i.bak "$@" && rm -f "$f.bak"; }
WORK_DIR="${WORK_DIR:-/tmp/pdfium-static}"
OUT_DIR="${OUT_DIR:-$repo_root/target/pdfium-static/${PDFIUM_TAG//\//_}/$PDFIUM_OS-$PDFIUM_CPU${PDFIUM_ENV:+-$PDFIUM_ENV}}"
namespacer="$script_dir/namespace_pdfium_jpeg.py"

# Fail before the long build, not after it.
[ -f "$namespacer" ] || { echo "missing $namespacer" >&2; exit 1; }
if [ "$PDFIUM_OS" = linux ]; then
  command -v docker >/dev/null || { echo "docker is required" >&2; exit 1; }
fi
if [ "$PDFIUM_OS" = win ]; then
  PYTHON="$(command -v python || true)"
else
  PYTHON="$(command -v python3 || command -v python || true)"
fi
[ -n "$PYTHON" ] || { echo "python is required" >&2; exit 1; }

mkdir -p "$WORK_DIR/pdfium-binaries" && cd "$WORK_DIR/pdfium-binaries"
# Init in place rather than clone: a restored source cache may already have put pdfium/ here.
if [ ! -d .git ]; then
  git init --quiet
  git remote add origin https://github.com/bblanchon/pdfium-binaries.git
fi
git fetch --tags --quiet origin
# Use the tag matching the pdfium release, NOT main: main's patch set targets today's pdfium
# revision and its patches fail to apply against older chromium/<N> checkouts.
# -f discards our local patches from a previous run, so a reused checkout can switch tags.
git checkout --quiet -f "$PDFIUM_TAG"

# Leftovers from a failed gclient sync would make the next sync fail.
sudo rm -rf _bad_scm 2>/dev/null || rm -rf _bad_scm

# Link against libstdc++ like core's pdfium-render feature; upstream static defaults to libc++.
if [ "$PDFIUM_OS" = linux ]; then
  grep -Fq 'echo "pdf_is_complete_lib = true"' steps/05-configure.sh
  sedi '/echo "pdf_is_complete_lib = true"/a\
    echo "use_custom_libcxx = false" # bernard-ledit: static libstdc++
' steps/05-configure.sh
fi

sedi 's/gclient sync /gclient sync --jobs 4 /' steps/02-checkout.sh
sedi '/09-test.sh/d; /10-pack.sh/d' build.sh
# Windows: link PDFium against the DLL C runtime (/MD) like Rust and PHP, not Chromium's
# default static runtime (/MT) for non-component builds. Runs after pdfium-binaries' own patches.
if [ "$PDFIUM_OS" = win ]; then
  crt_gn="$WORK_DIR/pdfium-binaries/pdfium/build/config/win/BUILD.gn"
  cat >> steps/03-patch.sh <<EOF

# bernard-ledit: dynamic CRT
grep -qE '":(static|dynamic)_crt"' "$crt_gn" || { echo "CRT config not found in $crt_gn" >&2; exit 1; }
sed -i 's/configs = \[ ":static_crt" \]/configs = [ ":dynamic_crt" ]/' "$crt_gn"
EOF

  if [ -n "${MSVC_TOOLSET:-}" ]; then
    toolchain_py="$WORK_DIR/pdfium-binaries/pdfium/build/toolchain/win/setup_toolchain.py"
    cat >> steps/03-patch.sh <<EOF

# bernard-ledit: compile with MSVC toolset $MSVC_TOOLSET (vcvarsall.bat -vcvars_ver)
if ! grep -q "vcvars_ver=$MSVC_TOOLSET" "$toolchain_py"; then
  if ! grep -q "_LoadEnvFromBat(args)" "$toolchain_py"; then
    echo "vcvarsall call not found in $toolchain_py; relevant code:" >&2
    grep -n -B2 -A12 -i "vcvarsall" "$toolchain_py" >&2 || true
    exit 1
  fi
    sed -i "/def _LoadEnvFromBat/!s/_LoadEnvFromBat(args)/_LoadEnvFromBat(args + ['-vcvars_ver=$MSVC_TOOLSET'])/" "$toolchain_py"
fi
EOF
  fi
fi
cd ..

# ubuntu:22.04 (glibc 2.35): the .a, and so our extension, inherits the builder's glibc
# symbol versions. `yes |` with `-i` answers install-build-deps' bare apt prompts; debconf
# is preseeded because `yes` can pick wrong entries in debconf dialogs, and sudo strips
# DEBIAN_FRONTEND. Flags must precede the os/cpu positionals (build.sh uses getopts).
if [ "$PDFIUM_OS" = mac ] || [ "$PDFIUM_OS" = win ]; then
  ( cd pdfium-binaries && ./build.sh -s -b "$PDFIUM_TAG" "$PDFIUM_OS" "$PDFIUM_CPU" )
  if [ "$PDFIUM_OS" = win ]; then
    lib="pdfium-binaries/pdfium/out/obj/pdfium.lib"
    if grep -aqi 'defaultlib:"\{0,1\}libcmt' "$lib"; then
      echo "pdfium.lib still requests the static C runtime (libcmt); the /MD patch did not apply" >&2
      exit 1
    fi
  fi
else
  yes | docker run --rm -i \
    -v "$PWD/pdfium-binaries:/w" -w /w \
    -e PDFIUM_TAG="$PDFIUM_TAG" -e PDFIUM_OS="$PDFIUM_OS" -e PDFIUM_CPU="$PDFIUM_CPU" -e PDFIUM_ENV="$PDFIUM_ENV" \
    -e MUSL_URL="${MUSL_URL:-https://github.com/mindee/bernard-ledit/releases/download/musl-toolchains}" \
    ubuntu:22.04 bash -eu -c '
      export DEBIAN_FRONTEND=noninteractive
      echo "debconf debconf/frontend select Noninteractive" | debconf-set-selections
      apt-get update
      apt-get install -y git curl python3 lsb-release sudo build-essential pkg-config file
      # The environment ("musl") is build.sh optional third positional argument.
      env_arg=""; [ "$PDFIUM_ENV" = musl ] && env_arg="musl"
      ./build.sh -s -b "$PDFIUM_TAG" "$PDFIUM_OS" "$PDFIUM_CPU" $env_arg
    ' || [ "${PIPESTATUS[1]}" -eq 0 ]
fi

if [ "${SKIP_NAMESPACE:-0}" = 1 ]; then
  echo "Raw build staged in $WORK_DIR/pdfium-binaries/staging"
  exit 0
fi

# Keep the staged original; Cargo must link the namespaced copy.
mkdir -p "$OUT_DIR"
"$PYTHON" "$namespacer" "$WORK_DIR/pdfium-binaries" "$OUT_DIR"
echo "Namespaced static PDFium in $OUT_DIR"