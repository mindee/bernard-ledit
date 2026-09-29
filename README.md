# Bernard l'Édit

<p align="center">
  <img src="docs/resources/bernard-ledit_logo.png" width="40%" alt="Bernard l'Édit">
</p>

> PDF, image, and geometry utilities for the Mindee SDK ecosystem.

Bernard l'Édit is a **Rust core library** with per-language bindings that gives every official Mindee Client Library a single, high-performance foundation for PDF manipulation, image processing, and geometry operations.  It replaces a patchwork of language-specific dependencies (pypdfium2/Pillow, PDFBox, SkiaSharp, Docnet, origami, Imagick, …) with one audited implementation.

> ⚠️ **Status:** under heavy development – APIs may change between minor versions.

---

## Table of contents

- [Features](#features)
- [Architecture](#architecture)
- [Python binding](#python-binding)
  - [Installation](#installation)
  - [Image API](#image-api)
  - [PDF API](#pdf-api)
  - [Geometry API](#geometry-api)
- [Development](#development)
  - [Prerequisites](#prerequisites)
  - [Build](#build)
  - [Test](#test)
  - [Lint](#lint)
- [Bindings](#bindings)
- [Thread safety](#thread-safety)
- [Third-party notices](#third-party-notices)
- [License](#license)

---

## Features

| Area         | Capabilities                                                                                                                       |
|--------------|------------------------------------------------------------------------------------------------------------------------------------|
| **Image**    | Decode (JPEG, PNG, TIFF, WebP, BMP, GIF, …), crop, resize (Lanczos, bicubic, nearest, …), encode, compress, save to path or buffer |
| **PDF**      | Load, split, merge, rasterise pages to JPEG, embed images, extract text and character metadata, add text overlays                  |
| **Geometry** | `Point` and `Polygon` primitives with centroid, bounding-box helpers                                                               |

---

## Architecture

```
bernard-ledit/
├── core/                   # Rust library (bernard-ledit crate)
│   └── src/
│       ├── image/          # Image decoding, encoding, compression
│       ├── pdf/            # PDF manipulation via pdfium
│       └── geometry/       # Point / Polygon primitives
└── bindings/
    ├── python/             # PyO3 extension (maturin)
    ├── dotnet/
    ├── java/
    ├── nodejs/
    ├── php/
    └── ruby/
```

The Rust core is the single source of truth.  Language bindings are thin wrappers that expose the same API idiomatically in each language.

---

## Python binding

### Installation

```bash
pip install bernard-ledit
```

The wheel bundles the compiled Rust extension and pdfium – no system dependencies required.

### Image API

```python
from bernard_ledit.image import decode, guess_format, compress

# --- decode ---
# Accepts bytes, bytearray, or any file-like object (BytesIO, BufferedReader, …)
img = decode(open("photo.jpg", "rb"))
img = decode(b"\xff\xd8\xff...")
print(img)           # Image(1920x1080, format:JPEG)
print(img.size)      # (1920, 1080)
print(img.format)    # 'JPEG'

# --- transform ---
cropped  = img.crop(left=0, top=0, right=960, bottom=540)
resized  = img.resize(640, 360)                    # Lanczos by default
resized  = img.resize(640, 360, filter="bicubic")  # nearest | triangle | catmullrom | gaussian | lanczos

# --- encode ---
jpeg_bytes = img.encode("JPEG", quality=85)        # optimize=True by default (mozjpeg)
png_bytes  = img.encode("PNG")
pdf_bytes  = img.encode("PDF")                     # single-page PDF via DCTDecode

# --- save ---
img.save("out.jpg")                    # format inferred from extension
img.save(pathlib.Path("out.png"))
img.save(buffer, format="JPEG")        # write to any writable buffer
img.save("out.pdf", format="PDF")      # force PDF regardless of extension

# --- helpers ---
fmt   = guess_format(data)             # "JPEG" | "PNG" | "TIFF" | …
jpeg, w, h = compress(
    data,
    quality=85,
    max_width=1024,    # aspect-preserving downscale, never upscales
    max_height=1024,
)
```

Supported resize filters: `nearest`, `triangle`, `catmullrom` (alias `bicubic`), `gaussian`, `lanczos` (default).

Supported encode formats: `JPEG`, `PNG`, `PDF`, `TIFF`, `WEBP`, `BMP`, `GIF`, `AVIF`, `ICO`, `QOI`, `TGA`, `PNM`, `FARBFELD`, `OPENEXR`.

### PDF API

```python
from bernard_ledit.pdf import PdfDocument

# --- load ---
doc = PdfDocument(open("document.pdf", "rb"))  # also accepts bytes or path
doc = PdfDocument(b"...")
doc = PdfDocument("/path/to/file.pdf")

# context-manager (auto-close)
with PdfDocument("document.pdf") as doc:
    print(len(doc))           # page count
    page = doc[0]             # or doc.get_page(0)
    print(page.get_size())    # PageSize(width=595.0, height=842.0)
    print(page.text())        # extracted text
    print(page.chars())       # list[TextChar] with font/bounds metadata
    print(page.rotation())    # 0 | 90 | 180 | 270
    bitmap = page.render(scale=2.0)      # PdfBitmap at 2× scale
    png_bytes = bitmap.to_png_bytes()

# --- rasterise a page to JPEG ---
jpeg_bytes = doc.rasterize_page(page_index=0, quality=85)

# --- split / merge ---
a = PdfDocument("a.pdf")
b = PdfDocument("b.pdf")
out = PdfDocument.new()
out.import_pages(a, [0, 1])
out.import_pages(b, [0])

buf = io.BytesIO()
out.save(buf)
out.save_to_file("merged.pdf")

# --- embed images ---
doc = PdfDocument.new()
doc.append_jpeg_page(jpeg_bytes)
doc.append_multiple_jpeg_pages([jpeg1, jpeg2])

# --- overlay text ---
from bernard_ledit.pdf import TextChar
chars = [TextChar(char="A", font_name="Helvetica", font_size=12.0,
                  font_weight=400, bounds=(50.0, 100.0, 62.0, 112.0))]
doc.add_text(page_idx=0, chars=chars)
```

### Geometry API [DEPRECATED]

```python
from bernard_ledit.geometry import Point, Polygon

p = Point(0.5, 0.75)
print(p.x, p.y)    # 0.5  0.75

poly = Polygon([(0, 0), (1, 0), (1, 1), (0, 1)])
print(poly.centroid())   # Point(0.5, 0.5)
print(len(poly))         # 4
```

---

## Development

### Prerequisites

| Tool                                                       | Purpose                              |
|------------------------------------------------------------|--------------------------------------|
| Rust ≥ 1.88 (`rust-toolchain.toml` pins the exact version) | Core + all bindings                  |
| [just](https://github.com/casey/just)                      | Task runner                          |
| [maturin](https://github.com/PyO3/maturin)                 | Build Python wheels                  |
| A C compiler + cmake                                       | mozjpeg (bundled, built from source) |
| Python ≥ 3.9 + virtualenv                                  | Python binding tests                 |

### Build

```bash
# Rust core (debug)
just build

# Python extension (installs into the active virtualenv)
just python build
```

### Test

```bash
# Rust core
just test

# Python binding (builds extension first if needed)
just python test
```

> **Note:** Rust tests run single-threaded (`RUST_TEST_THREADS=1` via `.cargo/config.toml`) because pdfium's renderer is not thread-safe.

### Lint

```bash
just lint          # fmt-check + clippy + cargo-deny + udeps
just check         # everything: fmt + lint + build + test
```

---

## Bindings

| Language | Crate / Package        | Status         |
|----------|------------------------|----------------|
| Python   | `bernard-ledit` (PyPI) | ✅ Active       |
| .NET     | `bindings/dotnet`      | 🚧 In progress |
| Java     | `bindings/java`        | 🚧 In progress |
| Node.js  | `bindings/nodejs`      | 🚧 In progress |
| PHP      | `bindings/php`         | 🚧 In progress |
| Ruby     | `bindings/ruby`        | 🚧 In progress |

---

## Thread safety

All pdfium rendering and mozjpeg encoding calls are serialized behind process-wide mutexes.
The Python GIL additionally protects single-threaded callers.

**Safe:** single-threaded Python, Python `threading` (GIL held in Rust), separate `multiprocessing` workers.  
**Avoid:** `os.fork()` after a `PdfDocument` has been opened in the parent process.

---

## Third-party notices

This distribution bundles mozjpeg (libjpeg-turbo + IJG).  See [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES) for the full attribution text required by their licenses.

---

## License

Apache 2.0 – see [LICENSE](LICENSE).

