#!/usr/bin/env bash
# Build the pinned shared libheif without bundling any codec.
# Set LIBHEIF_PLUGIN_PATH at runtime to a directory of compatible user-provided plugins.
set -euo pipefail

[ "$#" -ge 1 ] && [ "$#" -le 2 ] || {
    echo "usage: $0 <installation-directory> [--test-plugin]" >&2
    exit 1
}
libde265=OFF
if [ "$#" -eq 2 ]; then
    [ "$2" = "--test-plugin" ] || { echo "unknown option: $2" >&2; exit 1; }
    libde265=ON
fi
prefix="$(mkdir -p "$1" && cd "$1" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

if [ "$(uname -s)" = Darwin ]; then
    export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-13.0}"
fi

version=1.21.1
sha256=9799b4b1c19006f052bcf399c761cc147e279762683cefaf16871dbb9b4ea2a1
curl -fsSL "https://github.com/strukturag/libheif/releases/download/v$version/libheif-$version.tar.gz" -o "$work/source.tar.gz"
if command -v sha256sum >/dev/null; then
    actual="$(sha256sum "$work/source.tar.gz" | cut -d' ' -f1)"
else
    actual="$(shasum -a 256 "$work/source.tar.gz" | cut -d' ' -f1)"
fi
[ "$actual" = "$sha256" ] || { echo "libheif source checksum mismatch" >&2; exit 1; }
mkdir "$work/source"
tar -xzf "$work/source.tar.gz" -C "$work/source" --strip-components=1

cmake -S "$work/source" -B "$work/build" \
    -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$prefix" -DCMAKE_INSTALL_LIBDIR=lib \
    -DBUILD_SHARED_LIBS=ON -DBUILD_TESTING=OFF -DBUILD_DOCUMENTATION=OFF \
    -DENABLE_PLUGIN_LOADING=ON -DPLUGIN_DIRECTORY=/usr/local/lib/libheif \
    -DPLUGIN_INSTALL_DIRECTORY="$prefix/plugins" \
    -DWITH_EXAMPLES=OFF -DWITH_GDK_PIXBUF=OFF -DWITH_LIBSHARPYUV=OFF \
    -DWITH_LIBDE265="$libde265" -DWITH_LIBDE265_PLUGIN="$libde265" \
    -DWITH_FFMPEG_DECODER=OFF -DWITH_X265=OFF -DWITH_X264=OFF \
    -DWITH_OpenH264_DECODER=OFF -DWITH_AOM_DECODER=OFF -DWITH_AOM_ENCODER=OFF \
    -DWITH_DAV1D=OFF -DWITH_RAV1E=OFF -DWITH_SvtEnc=OFF -DWITH_KVAZAAR=OFF \
    -DWITH_JPEG_DECODER=OFF -DWITH_JPEG_ENCODER=OFF \
    -DWITH_OpenJPEG_DECODER=OFF -DWITH_OpenJPEG_ENCODER=OFF -DWITH_OPENJPH_ENCODER=OFF \
    -DWITH_UVG266=OFF -DWITH_VVDEC=OFF -DWITH_VVENC=OFF
cmake --build "$work/build" --config Release --parallel 4
cmake --install "$work/build" --config Release
cp "$work/source/COPYING" "$prefix/libheif-LICENSE"
