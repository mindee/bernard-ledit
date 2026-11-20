#!/usr/bin/env bash
set -euo pipefail

[ "$#" -eq 2 ] || { echo "usage: $0 <archive> <plugin-directory>" >&2; exit 1; }
archive="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
plugin="$(cd "$2" && pwd)"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
staging="$(mktemp -d)"
trap 'rm -rf "$staging"' EXIT
unzip -q "$archive" -d "$staging"
export HEIC_FIXTURE="$root/tests/data/file_types/receipt.heic"
unset LD_LIBRARY_PATH DYLD_LIBRARY_PATH

LIBHEIF_PLUGIN_PATH="$staging/no-plugins" php -n -d "extension=$staging/bernard_ledit.so" -r '
if (!extension_loaded("bernard_ledit")) { throw new Exception("extension not loaded"); }
if (BernardLedit\Pdf\PdfDocument::create()->getPageCount() !== 0) { throw new Exception("PDF failed"); }
try {
    new BernardLedit\Image\Image(file_get_contents(getenv("HEIC_FIXTURE")));
} catch (Throwable $error) {
    if (!str_contains($error->getMessage(), "no HEVC decoder")) { throw $error; }
    exit(0);
}
throw new Exception("HEIC decoded without a plugin");
'

export LIBHEIF_PLUGIN_PATH="$plugin"
export EXPECTED_HEIF="$staging/native/libheif.so.1"
if [ "$(uname -s)" = Darwin ]; then
    export DYLD_PRINT_LIBRARIES=1
fi
php -n -d "extension=$staging/bernard_ledit.so" -r '
$image = new BernardLedit\Image\Image(file_get_contents(getenv("HEIC_FIXTURE")));
if ($image->getSize() !== [800, 1066]) { throw new Exception("HEIC decoding failed"); }
if (PHP_OS_FAMILY === "Linux") {
    $libraries = [];
    foreach (file("/proc/self/maps") as $line) {
        if (preg_match("~(/[^ ]*/libheif\\.so\\.1)\\s*$~", $line, $match)) {
            $libraries[$match[1]] = true;
        }
    }
    if (count($libraries) !== 1 || !isset($libraries[realpath(getenv("EXPECTED_HEIF"))])) {
        throw new Exception("Expected exactly one bundled libheif: " . json_encode(array_keys($libraries)));
    }
}
' 2>"$staging/loader.log"
unset DYLD_PRINT_LIBRARIES
if [ "$(uname -s)" = Darwin ]; then
    grep -F "$staging/native/libheif.1.dylib" "$staging/loader.log"
    [ "$(grep -c 'libheif\.1\.dylib' "$staging/loader.log")" -eq 1 ]
fi
echo "PHP package and external HEVC plugin verified"
