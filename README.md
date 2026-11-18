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
- [PDFium: dynamic and static builds](#pdfium-dynamic-and-static-builds)
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
│   ├── build.rs            # Fetches the dynamic PDFium for development builds
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
│   └── pdfium-static/      # Builds and downloads static PDFium libraries
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

The always-on HEIC decoder dynamically links **system libheif 1.17 or newer with the
libde265 HEVC decoder**, including for applications that do not decode HEIC. This applies to
Rust builds and released Python and PHP bindings; native libheif and its plugins are not bundled.
An HEVC encoder is not required. Build environments need development headers and library discovery;
deployment environments need the compatible shared library and decoder, including any separately
packaged plugin, discoverable by the OS loader. Matching CPU architecture, libc and ABI matter.

- Ubuntu 24.04 build: `sudo apt-get install pkg-config libheif-dev libheif-plugin-libde265`.
  For deployment install the corresponding runtime library and decoder plugin from the same
  compatible distribution. Older distributions may not offer libheif 1.17; the extension's
  glibc 2.35 baseline does not make Ubuntu 24.04's libheif usable on glibc 2.35. Such targets
  need a separately built or backported, ABI-compatible libheif >= 1.17 and libde265; stock
  Ubuntu 22.04 packages are too old. The CI extension symbol-version check enforces its 2.35
  baseline, but does not check the glibc requirements of the separately installed libraries.
- Alpine: install `libheif-dev` for builds and the matching `libheif` runtime; newer Alpine
  packages may also require `libheif-libde265`. Check the package's decoder configuration.
- macOS: `brew install libheif`; ensure the shared library and decoder are available at the
  install names recorded in the extension on the deployment machine.
- Windows x64 MSVC: install vcpkg's dynamic `libheif[core,libde265]:x64-windows` port, set
  `VCPKG_ROOT` and `VCPKGRS_DYNAMIC=1` when building, and make its DLLs available to the loader
  when running. The extension's MSVC runtime and the system libraries must be compatible.

Do not use wheel repair or packaging steps that vendor libheif or its codec libraries. This
system-dependency policy needs verification for each wheel/PIE target and deployment: a wheel
tag or successful build alone does not establish portability, loader paths, or plugin availability.
No Python wheel publishing workflow exists in this repository. See
[image formats](docs/image-formats.md) for the decoder scope and licensing/patent caveats.

---

## PDFium: dynamic and static builds

PDFium comes from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries), at the release pinned in [
`core/pdfium-release`](core/pdfium-release). Builds use it in one of two ways:

|                   | Dynamic (default)                                                                                                       | Static                                                                                                                                        |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| **Used for**      | Development and tests                                                                                                   | Released packages                                                                                                                             |
| **How**           | `core/build.rs` downloads the shared library into `core/.pdfium_cache`; the binding loads it at runtime (`PDFIUM_PATH`) | PDFium is linked into the binding at build time: `PDFIUM_STATIC_LIB_PATH` points to the library, and a static feature selects the C++ runtime |
| **Cargo feature** | none                                                                                                                    | `static-pdfium` (Linux, libstdc++), `static-pdfium-libcxx` (macOS, libc++), `static-pdfium-msvc` (Windows)                                    |

Static libraries are prebuilt by [`build-static-pdfium.yml`](.github/workflows/build-static-pdfium.yml) and published,
each with a `.sha256` checksum, to the `pdfium-static-<release>` GitHub release of this repository (e.g.
`pdfium-static-chromium_8076`). Each target's archive is published with its corresponding license bundle.

| Target              | For                                        |
| ------------------- | ------------------------------------------ |
| `linux-x64-glibc`   | Linux x86_64, glibc 2.35+                  |
| `linux-arm64-glibc` | Linux arm64, glibc 2.35+                   |
| `linux-x64-musl`    | Alpine x86_64                              |
| `linux-arm64-musl`  | Alpine arm64                               |
| `mac-arm64`         | macOS, Apple Silicon                       |
| `mac-x64`           | macOS, Intel                               |
| `win-x64-v142`      | Windows x86_64, MSVC 14.29 (PHP 8.1 – 8.3) |
| `win-x64-v143`      | Windows x86_64, MSVC 14.44 (PHP 8.4, 8.5)  |
| `win-arm64-v143`    | Windows arm64, MSVC 14.44                  |

```bash
# Download a published library (needs an authenticated `gh`)
bash tools/pdfium-static/download.sh <target> <directory>

# Or build one yourself (Linux: needs Docker; Windows: also set MSVC_TOOLSET, e.g. 14.44)
PDFIUM_OS=linux PDFIUM_CPU=x64 bash tools/pdfium-static/build.sh
```

`build.sh` documents its other settings (musl, work and output directories) in its header. The target list lives in [
`tools/pdfium-static/targets.json`](tools/pdfium-static/targets.json).

---

## Development

### Prerequisites

| Tool                                                     | Purpose                                     |
| -------------------------------------------------------- | ------------------------------------------- |
| Rust 1.99.0 (pinned in `rust-toolchain.toml`)            | Core + all bindings                         |
| [just](https://github.com/casey/just)                    | Task runner                                 |
| A C compiler + cmake + nasm                              | Building static PDFium libraries            |
| System libheif ≥ 1.17 + libde265, development files     | HEIC decoder in core and bindings           |
| [GitHub CLI](https://cli.github.com/)                    | Downloading static PDFium libraries         |
| Docker                                                   | Building static PDFium for Linux            |

### Build

```bash
# Rust core (release)
just core build
```

### Test

```bash
# Rust core
just core test
just core hack-test        # every feature combination, except the static-linking ones
```

> **Note:** Rust tests run single-threaded (`RUST_TEST_THREADS=1` via `.cargo/config.toml`) because pdfium's renderer is
> not thread-safe.

### Lint

```bash
just core lint             # fmt-check + clippy + cargo-deny + udeps + feature-powerset check
just core check-all        # everything: fmt + lint + build + test
```

### CI

| Workflow                           | What it does                                                                                                              |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `pull-request.yml`, `main-dev.yml` | Entry points: run the unit tests (including `receipt.heic`), binding tests and linters                                    |
| `build-static-pdfium.yml`          | Builds and publishes any missing static PDFium library, then runs `_php-static-check.yml` against the published libraries |
| `_php-static-check.yml`            | Builds and tests the PHP extension with static PDFium: PHP 8.1 – 8.5, ZTS and NTS, on every target with a PHP build       |

---

## Thread safety

PDFium rendering calls are serialized by a process-wide mutex.
See the [Python binding](bindings/python/README.md#thread-safety) for Python-specific guidance.

---

## License

Apache 2.0 – see [LICENSE](LICENSE).
