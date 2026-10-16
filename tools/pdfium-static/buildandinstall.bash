#!/usr/bin/env bash
set -eu
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p /tmp/pdfium-static && cd /tmp/pdfium-static
if [ ! -d pdfium-binaries/.git ]; then
  git clone https://github.com/bblanchon/pdfium-binaries.git
fi
cd pdfium-binaries
# Check out the tag matching PDFIUM_RELEASE in core/build.rs, NOT main. pdfium-binaries tags
# its own repo per-release (patches/ + build.sh included), and main's patch set is written
# against whatever pdfium/build revision *today's* HEAD targets — it drifts from older
# chromium/<N> source checkouts and its patches will fail to apply (seen in practice: hunk
# failures in patches/clang_rt.patch against build/config/clang/BUILD.gn). Checking out the
# matching tag keeps the pdfium revision and the patch set from the same point in time.
git checkout chromium/8076
# Match core's pdfium-render/libstdc++ feature; the upstream static default uses libc++.
if ! grep -Fq '# bernard-ledit: static libstdc++' steps/05-configure.sh; then
  grep -Fq 'echo "pdf_is_complete_lib = true"' steps/05-configure.sh
  sed -i '/echo "pdf_is_complete_lib = true"/a\    echo "use_custom_libcxx = false" # bernard-ledit: static libstdc++' steps/05-configure.sh
fi
# Skip build.sh's own internal self-test (step 9) and release-packaging (step 10): step 9
# hardcodes system gcc/g++ to link a throwaway example.c against libpdfium.a, which fails on
# stock Ubuntu with the recent-Clang CREL/eh_frame incompatibility (see plan doc §7.1 for
# full explanation) — but by the time step 9 runs, step 7 has ALREADY staged the libpdfium.a
# we actually need, so this failure doesn't affect our deliverable. Step 10 just tars up a
# release artifact we don't use either (we read staging/lib/ directly).
sed -i '/09-test.sh/d; /10-pack.sh/d' build.sh
cd ..
# Build on ubuntu:22.04 (glibc 2.35), NOT 24.04: the .a inherits the glibc symbol versions of
# the builder, and so will our .so. 22.04 = what bblanchon's own runners produce today.
#
# `yes |` + `-i` (not `-it`): pdfium's own build/install-build-deps.sh (run internally by
# steps/04-install-extras.sh, invoked by build.sh — we don't call it directly, so we can't
# just pass it a flag) calls `apt-get install` WITHOUT `-y`/`--assume-yes` unless its own
# `--no-prompt` flag is set, which nothing in this chain sets. That leaves a bare
# "Do you want to continue? [Y/n]" reading from stdin partway through the build, which hangs
# forever under a plain `-it` run once you walk away from the terminal. Piping `yes` answers
# that (and any other stray Y/n prompt) automatically; `-it` would fight with piped stdin
# ("the input device is not a TTY"), so use `-i` only.
#
# ⚠️ `yes` is NOT enough on its own — it only answers plain stdin y/n reads. Installing
# `keyboard-configuration`/`console-setup` (pulled in transitively) triggers a *debconf*
# dialog (whiptail-style keyboard-layout picker), which reads input completely differently.
# Piping `yes` into that is actively unsafe for automation: instead of "continuing", it can
# select a garbage/wrong menu entry and leave a bogus config baked into the image — it won't
# necessarily fail loudly. Pre-seeding debconf's answer in its own database (below) makes it
# pick its *default* silently and never prompt at all, which is what we actually want.
# `DEBIAN_FRONTEND=noninteractive` exported in the shell is *not* sufficient by itself either:
# the installs run via `sudo apt-get install ...` inside install-build-deps.py, and `sudo`
# resets the environment by default (`env_reset` in sudoers) — an exported env var in our
# shell isn't guaranteed to survive that `sudo` call. debconf-set-selections writes straight
# to debconf's own database, so it survives both the env reset and the sudo boundary.
yes | docker run --rm -i -v "$PWD/pdfium-binaries:/w" -w /w ubuntu:22.04 bash -eu -c '
  export DEBIAN_FRONTEND=noninteractive
  echo "debconf debconf/frontend select Noninteractive" | debconf-set-selections
  apt-get update
  apt-get install -y git curl python3 lsb-release sudo build-essential pkg-config file
  # -b IS still needed: it sets PDFium_BRANCH, which steps/02-checkout.sh uses for
  # `gclient sync -r origin/${PDFium_BRANCH:-main}` against pdfium.googlesource.com itself.
  # Checking out the pdfium-binaries tag above only pins *this* repos patches/build scripts;
  # it does not touch PDFium_BRANCH, which defaults to "main" and would silently fetch
  # todays pdfium source instead of the one chromium/8076 actually refers to
  # (verified: pdfium.googlesource.com has a matching `refs/heads/chromium/8076` branch).
  # Flags must come BEFORE the positional os/cpu args: build.sh uses bash getopts, which stops
  # parsing options at the first non-flag argument. `-s linux x64 -b chromium/8076` leaves
  # `-b` and `chromium/8076` as two stray positional args -> "Too many arguments".
  ./build.sh -s -b chromium/8076 linux x64
'
# Preserve the staged original; Cargo must use the namespaced copy.
python3 "$script_dir/namespace_pdfium_jpeg.py" \
  /tmp/pdfium-static/pdfium-binaries "$script_dir/pdfium-static-namespaced"
