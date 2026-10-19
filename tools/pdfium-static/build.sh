#!/usr/bin/env bash
# Builds a static, JPEG-namespaced libpdfium.a with bblanchon/pdfium-binaries, inside Docker.
#
# Env (all optional):
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
WORK_DIR="${WORK_DIR:-/tmp/pdfium-static}"
OUT_DIR="${OUT_DIR:-$repo_root/target/pdfium-static/${PDFIUM_TAG//\//_}/$PDFIUM_OS-$PDFIUM_CPU}"
namespacer="$script_dir/namespace_pdfium_jpeg.py"

# Fail before the long build, not after it.
[ -f "$namespacer" ] || { echo "missing $namespacer" >&2; exit 1; }
command -v docker >/dev/null || { echo "docker is required" >&2; exit 1; }
command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 1; }

mkdir -p "$WORK_DIR" && cd "$WORK_DIR"
if [ ! -d pdfium-binaries/.git ]; then
  git clone https://github.com/bblanchon/pdfium-binaries.git
fi
cd pdfium-binaries
git fetch --tags --quiet
# Use the tag matching the pdfium release, NOT main: main's patch set targets today's pdfium
# revision and its patches fail to apply against older chromium/<N> checkouts.
# Discard the local patches below first, so a re-run on a reused checkout can switch tags.
git checkout --quiet -- .
git checkout --quiet "$PDFIUM_TAG"

# Leftovers from a failed gclient sync would make the next sync fail.
sudo rm -rf _bad_scm 2>/dev/null || rm -rf _bad_scm

# Link against libstdc++ like core's pdfium-render feature; upstream static defaults to libc++.
grep -Fq 'echo "pdf_is_complete_lib = true"' steps/05-configure.sh
sed -i '/echo "pdf_is_complete_lib = true"/a\    echo "use_custom_libcxx = false" # bernard-ledit: static libstdc++' steps/05-configure.sh

# Fewer parallel fetches: googlesource throttles bursts from CI IPs (HTTP 503).
sed -i 's/gclient sync /gclient sync --jobs 4 /' steps/02-checkout.sh

# Skip step 9 (self-test linking with system gcc, fails with recent Clang's CREL/eh_frame)
# and step 10 (release tarball). Step 7 has already staged the libpdfium.a we use.
sed -i '/09-test.sh/d; /10-pack.sh/d' build.sh
cd ..

# ubuntu:22.04 (glibc 2.35): the .a, and so our extension, inherits the builder's glibc
# symbol versions. `yes |` with `-i` answers install-build-deps' bare apt prompts; debconf
# is preseeded because `yes` can pick wrong entries in debconf dialogs, and sudo strips
# DEBIAN_FRONTEND. Flags must precede the os/cpu positionals (build.sh uses getopts).
yes | docker run --rm -i \
  -v "$PWD/pdfium-binaries:/w" -w /w \
  -e PDFIUM_TAG="$PDFIUM_TAG" -e PDFIUM_OS="$PDFIUM_OS" -e PDFIUM_CPU="$PDFIUM_CPU" \
  ubuntu:22.04 bash -eu -c '
    export DEBIAN_FRONTEND=noninteractive
    echo "debconf debconf/frontend select Noninteractive" | debconf-set-selections
    apt-get update
    apt-get install -y git curl python3 lsb-release sudo build-essential pkg-config file
    # -b sets PDFium_BRANCH for gclient; without it, gclient syncs pdfium main.
    ./build.sh -s -b "$PDFIUM_TAG" "$PDFIUM_OS" "$PDFIUM_CPU"
  ' || [ "${PIPESTATUS[1]}" -eq 0 ]

if [ "${SKIP_NAMESPACE:-0}" = 1 ]; then
  echo "Raw build staged in $WORK_DIR/pdfium-binaries/staging"
  exit 0
fi

# Keep the staged original; Cargo must link the namespaced copy.
mkdir -p "$OUT_DIR"
python3 "$namespacer" "$WORK_DIR/pdfium-binaries" "$OUT_DIR"
echo "Namespaced static PDFium in $OUT_DIR"