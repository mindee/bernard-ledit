# Bernard l'Édit

<p align="center">
  <img src="docs/resources/bernard-ledit_logo.png" width="40%" alt="Bernard l'Édit">
</p>

> PDF and image utilities with multiple languages bindings.

> ⚠️ **Status:** under heavy development – APIs may change between minor versions.

---

## Table of contents

- [Features](#features)
- [Architecture](#architecture)
- [Overview](#overview)
- [Bindings](#bindings)
- [Native libheif prerequisites](#native-libheif-prerequisites)
- [Native PDFium prerequisite](#native-pdfium-prerequisite)
- [Development](#development)
  - [Prerequisites](#prerequisites)
  - [Build](#build)
  - [Test](#test)
  - [Lint](#lint)
  - [CI](#ci)
- [Thread safety](#thread-safety)
- [License](#license)

---

## Features

| Area         | Capabilities                                                                                                                                                                                        |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Image**    | Decode (JPEG, PNG, TIFF, WebP, BMP, GIF, HEIC, HEIF …), crop, resize, encode, compress to JPEG, extract polygon crops from images and PDF pages                                         |
| **PDF**      | Load, split, merge, rasterise pages to images, embed images, extract text and character metadata, compress scans, extract sub-documents, apply page options, repair broken headers |

---

## Architecture

```
bernard-ledit/
├── core/                   # Rust library (bernard-ledit crate)
│   ├── pdfium-release      # Pinned PDFium release (pdfium-binaries tag)
│   ├── build.rs            # Links the system PDFium shared library
│   └── src/
│       ├── image/          # Image decoding, encoding, compression, polygon crops
│       ├── pdf/            # PDF manipulation via PDFium
│       └── utils/
├── bindings/
│   ├── python/             # PyO3 extension (maturin)
│   ├── php/                # PHP extension (ext-php-rs)
│   ├── java/
│   ├── dotnet/
│   ├── nodejs/
│   └── ruby/
├── tools/
│   └── download-pdfium.sh  # Downloads shared PDFium for development and CI
└── .github/workflows/      # CI, see below
```

---

## Overview

Bernard-Ledit uses:
1. pdfium for PDF manipulation
2. jpeg-encoder to compress using JPEG
3. libheif for HEIC/HEIF image decoding
4. image for general image decoding, encoding, and manipulation

Supported encode formats: `JPEG`, `PNG`, `PDF`, `TIFF`, `WEBP`, `BMP`, `GIF`, `AVIF`, `ICO`, `QOI`, `TGA`, `PNM`,
`FARBFELD`, `OPENEXR`.

HEIC support is decode-only; HEIC encoding is unsupported. See
[image formats](docs/image-formats.md) for the scope and current status.

## Bindings

The Python binding is the reference for the other bindings, which follow its API and documentation
structure while using conventions appropriate to their language.

| Language | Crate / Package        | Status                                             |
| -------- | ---------------------- | -------------------------------------------------- |
| [Python](bindings/python/README.md) | `bernard-ledit` (PyPI) | ✅ Active |
| [PHP](bindings/php/README.md) | `bindings/php` | ✅ Active, used by the PHP SDK; PIE release coming |
| [Java](bindings/java/README.md) | `bindings/java` | 🚧 In progress |
| [.NET](bindings/dotnet/README.md) | `bindings/dotnet` | 🚧 In progress |
| [Node.js](bindings/nodejs/README.md) | `bindings/nodejs` | 🚧 In progress |
| [Ruby](bindings/ruby/README.md) | `bindings/ruby` | 🚧 In progress |

---

## Native libheif prerequisites

Python wheels, PHP release archives, and development builds use the same pinned shared
libheif **1.21.1**. The library is built without codecs. HEIC decoding requires a separately
installed libde265 or FFmpeg decoder plugin compatible with libheif 1.21.1; plugins built
for older libheif versions may not work. Set `LIBHEIF_PLUGIN_PATH` to the plugin directory
before starting the application.

For local development on Linux or macOS, install CMake, a C++ compiler, and pkg-config.
Install libde265's development files too if you need HEIC decoding in tests.

| Platform | Build dependencies |
| --- | --- |
| Ubuntu | `sudo apt-get install build-essential cmake pkg-config` (add `libde265-dev` for the test plugin) |
| macOS | `brew install cmake pkgconf` (add `libde265` for the test plugin) |

Build the pinned shared library:

```bash
libheif_prefix="$HOME/.cache/bernard-ledit/libheif-1.21.1"
bash tools/build-libheif.sh "$libheif_prefix"
export LIBHEIF_PREFIX="$libheif_prefix"
export PKG_CONFIG_PATH="$libheif_prefix/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
case "$(uname -s)" in
  Darwin) export DYLD_LIBRARY_PATH="$libheif_prefix/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" ;;
  *) export LD_LIBRARY_PATH="$libheif_prefix/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ;;
esac
```

For HEIC decoding in local tests, build a matching plugin separately:

```bash
plugin_prefix="$HOME/.cache/bernard-ledit/heif-test-plugin-1.21.1"
bash tools/build-libheif.sh "$plugin_prefix" --test-plugin
export LIBHEIF_PLUGIN_PATH="$plugin_prefix/plugins"
```

On Windows x64 MSVC, install the pinned port and plugin with
`vcpkg install "libheif[libde265]:x64-windows" --overlay-ports=tools/vcpkg`, then set
`VCPKG_ROOT`, `VCPKGRS_DYNAMIC=1`, `LIBHEIF_PREFIX` to
`<vcpkg-root>/installed/x64-windows`, and `LIBHEIF_PLUGIN_PATH` to
`<vcpkg-root>/installed/x64-windows/plugins/libheif`. Make
`<vcpkg-root>/installed/x64-windows/bin` available on `PATH`.

The plugin and its libde265 dependency are not included in release packages. libheif's
source is available from the
[1.21.1 upstream release](https://github.com/strukturag/libheif/releases/tag/v1.21.1).

---

## Native PDFium prerequisite

Bundled Python wheels and PHP release archives contain the shared PDFium build matching
[`core/pdfium-release`](core/pdfium-release). Development builds and other bindings need
the matching [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) shared build
installed separately for the OS, architecture and libc. It must be visible to the system
loader when the extension loads, even without PDF operations.

For development, download the pinned release and point the linker at its library directory:

```bash
# Example for Apple Silicon; use linux-x64, linux-arm64, linux-musl-x64,
# linux-musl-arm64, mac-x64, or win-x64 for the other supported targets.
bash tools/download-pdfium.sh mac-arm64 /path/to/pdfium
export PDFIUM_DYNAMIC_LIB_PATH=/path/to/pdfium/lib
```

On Linux, install `libpdfium.so` in a loader search directory or configure `LD_LIBRARY_PATH`
before starting the application. On macOS, install `libpdfium.dylib` in `/usr/local/lib` or
`/opt/homebrew/lib`; the download script sets its install name to `@rpath/libpdfium.dylib`.
On Windows, the build needs `lib/pdfium.lib` (the script copies the provided import library to
that name); the application must find `bin/pdfium.dll` at runtime. Python on Windows may require
placing the DLL beside `python.exe` or using `os.add_dll_directory()` before importing the package.
The loader's search rules and ABI must match the installed binary. The old `PDFIUM_PATH` and PHP
`bernard_ledit.pdfium_path` settings are no longer used.

---

## Development

### Prerequisites

| Tool                                                     | Purpose                                     |
| -------------------------------------------------------- | ------------------------------------------- |
| Rust 1.99.0 (pinned in `rust-toolchain.toml`)            | Core + all bindings                         |
| [just](https://github.com/casey/just)                    | Task runner                                 |
| Pinned libheif 1.21.1 development build; matching plugin for HEIC tests | HEIC support in core and bindings |
| System shared PDFium and `PDFIUM_DYNAMIC_LIB_PATH` | PDF operations in core and bindings |


### Build

```bash
# Rust core (release)
just core build
```

### Test

```bash
# Rust core
just core test
just core hack-test        # every feature combination
```

### Lint

```bash
just core lint             # fmt-check + clippy + cargo-deny + udeps + feature-powerset check
just core check-all        # everything: fmt + lint + build + test
```

---

## Thread safety

PDFium rendering calls are serialized by a process-wide mutex.
See the [Python binding](bindings/python/README.md#thread-safety) for Python-specific guidance.

---

## License

Apache 2.0 – see [LICENSE](LICENSE).