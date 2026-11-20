# PHP binding

The PHP binding exposes Bernard l'Édit's PDF and image APIs through an
[ext-php-rs](https://github.com/davidcole1340/ext-php-rs) extension, following the
[Python reference binding](../python/README.md) with idiomatic PHP naming. It is used by the
[Mindee PHP SDK](https://github.com/mindee/mindee-api-php).
Its API is documented in the [IDE / PHPStan stubs](bernard_ledit.stubs.php), regenerated with
`just php stubs`.

**Supported:** PHP 8.1 – 8.5, ZTS and NTS, on Linux (x86_64 and arm64, glibc), macOS
(arm64 and x86_64) and Windows (x86_64). The Linux glibc builds require glibc 2.35 or newer.
Bundled release archives include the pinned shared PDFium and libheif libraries. HEIC decoding
requires a separately installed [compatible decoder plugin](../../README.md#native-libheif-prerequisites).
The macOS archives require macOS 13 or newer.

## Installation

On Linux and macOS, extract the release archive and keep `bernard_ledit.so` together with its
`native/` directory; point `extension=` in `php.ini` at the extracted extension. PIE currently
copies only the extension on Unix, so it cannot install these bundled archives automatically.
On Windows, keep the DLLs together and ensure their directory is on `PATH` when PHP starts.
Normal loader overrides or previously loaded libraries can still affect resolution.

Until release archives are available, build from source. From the repository root:

```bash
# After installing PDFium and setting PDFIUM_DYNAMIC_LIB_PATH:
just php test-release
```

The extension
is then in `target/release/` (`libbernard_ledit_php.so`, `.dylib` on macOS, or
`bernard_ledit_php.dll` on Windows); load it with `extension=/path/to/it` in your `php.ini`.

On Windows, build with Rust nightly (`RUSTUP_TOOLCHAIN=nightly`, required by ext-php-rs) and with
the MSVC toolset your PHP was built with: 14.29 (v142) for PHP 8.1 – 8.3, 14.44 (v143) for 8.4
and 8.5.

For release packaging, build against the pinned libraries in a fresh Cargo target directory,
then run:

```bash
bash bindings/php/tools/pie-package.sh /path/to/extension v0.1.0 /path/to/output \
  /path/to/libheif /path/to/pdfium
```

On Linux this also requires `patchelf` and binutils (`readelf` and `objdump`); glibc
archives reject binaries requiring glibc newer than 2.35. On Windows, pass the vcpkg
`installed/x64-windows` directory for libheif and put its license at `libheif-LICENSE`
in that directory; `llvm-readobj` or `dumpbin` must also be on `PATH`. Archive contents
include the licenses for both bundled libraries;
libheif source for version 1.21.1 is available from
[its upstream release](https://github.com/strukturag/libheif/releases/tag/v1.21.1).

Document and image inputs are encoded bytes in PHP strings. The calling application handles
all file and stream I/O, including writing returned bytes.

## Image API

```php
<?php

use BernardLedit\Image\Image;
use function BernardLedit\Image\compressToJpeg;
use function BernardLedit\Image\guessFormat;

// --- load ---
$data = file_get_contents('photo.jpg');
$img = new Image($data);
[$width, $height] = $img->getSize();
echo $img->getFormat(); // JPEG

// --- transform (both return a new Image) ---
$cropped = $img->crop(0, 0, 960, 540);
$resized = $img->resize(640, 360); // exact size, Lanczos filter

// --- encode and save ---
$jpegBytes = $img->encode('JPEG', 85, true);
$pngBytes = $img->encode('PNG');
$pdfBytes = $img->encode('PDF'); // single-page PDF via DCTDecode
file_put_contents('out.jpg', $jpegBytes);
file_put_contents('out.png', $pngBytes);
file_put_contents('out.pdf', $pdfBytes);

// --- helpers ---
$format = guessFormat($data); // JPEG, PNG, TIFF, ...

// --- compress to JPEG ---
// Returns JPEG bytes and dimensions. Throws ImageException if the output
// is not smaller and the dimensions are unchanged.
[$compressed, $width, $height] = compressToJpeg($data, 85, true, 1024, 1024);

// --- crop by polygon ---
// Polygon points are normalized (x, y) coordinates (0..1).
$crop = $img->cropPolygon([[0.1, 0.1], [0.5, 0.1], [0.5, 0.4], [0.1, 0.4]]);
file_put_contents('crop.jpg', $crop->encode('JPEG'));
```

Supported encode formats: `JPEG`, `PNG`, `PDF`, `TIFF`, `WEBP`, `BMP`, `GIF`, `AVIF`, `ICO`, `QOI`,
`TGA`, `PNM`, `FARBFELD`, `OPENEXR`. HEIC support is decode-only; see
[image formats](../../docs/image-formats.md) for the scope and current status.

## PDF API

```php
<?php

use BernardLedit\Pdf\PdfDocument;
use function BernardLedit\Pdf\applyPageOptions;
use function BernardLedit\Pdf\compressToJpeg;
use function BernardLedit\Pdf\extractFromPageRanges;
use function BernardLedit\Pdf\fixPdf;

// --- load ---
$pdfBytes = file_get_contents('document.pdf');
$doc = new PdfDocument($pdfBytes);
echo $doc->getPageCount();
$page = $doc->getPage(0);
echo $page->getText();

// --- rasterise a page to an image ---
$image = $page->render(2.0);
$pngBytes = $image->encode('PNG');

// --- split / merge ---
$a = new PdfDocument(file_get_contents('a.pdf'));
$b = new PdfDocument(file_get_contents('b.pdf'));
$out = PdfDocument::create();
$out->importPages($a, [0, 1]);
$out->importPages($b, [0]);
file_put_contents('out.pdf', $out->toBytes());

// --- compress to JPEG ---
// Only pages without a text layer (scans) are rasterised and re-encoded as JPEG.
// Returns the input unchanged if the result would not be smaller.
$smaller = compressToJpeg($pdfBytes, 85, true, 150.0);
// The method form returns a new PdfDocument, or null when there is nothing to gain.
$result = $doc->compressToJpeg(85, true, 150.0);

// --- extract sub-documents ---
// The input is a PDF or an image (treated as a one-page document).
// Page indexes are 0-based; returns one PdfDocument per group.
$parts = extractFromPageRanges($pdfBytes, [[0, 1], [4]]);

// --- keep or remove pages ---
// Operation is KEEP_ONLY or REMOVE; negative indexes count from the end.
// Returns [document or null, skipped indexes]. Null means the PDF has fewer
// than the minimum number of pages, so nothing was done.
[$selected, $skipped] = applyPageOptions($pdfBytes, 'KEEP_ONLY', 0, [0, -1]);

// --- repair a broken header ---
// Drops every byte before the first %PDF- marker, if it sits before the maximum offset.
$fixed = fixPdf($pdfBytes, 500);

// Close documents when they are no longer needed.
$doc->close();
```

## Development

PHP 8.1+ with development headers (`php-config`), Composer and libclang are required, in addition to
the [shared development prerequisites](../../README.md#prerequisites). For example:

```bash
# Debian / Ubuntu
sudo apt install php-dev libclang-dev

# Fedora / RHEL
sudo dnf install php-devel clang-devel
```

Run these commands from the repository root:

```bash
just php build         # debug build
just php build-release # release build
just php test          # debug build
just php test-release  # release build
just php smoke         # checks that the extension loads
just php lint          # clippy on the PHP binding
just php stubs-check   # fails if the committed stubs are out of date
```

The test recipes ignore `php.ini` so they test the local build rather than an installed copy.
They load required PHP modules explicitly only if they are not built into PHP.
