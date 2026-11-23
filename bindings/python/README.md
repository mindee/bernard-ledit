# Python binding

The Python binding exposes Bernard l'Édit's PDF and image APIs through a PyO3 extension.

## Installation

```bash
pip install bernard-ledit
```

The wheel bundles the compiled Rust extension with PDFium statically linked. HEIC decoding requires
system libheif and either its libde265 or FFmpeg HEVC decoder plugin; see the
[native libheif prerequisites](../../README.md#native-libheif-prerequisites).
Document and image inputs must be `bytes`, not paths or file-like objects. The calling application
handles all file and stream I/O, including writing returned bytes.

## Image API

```python
import io
from pathlib import Path

from bernard_ledit.common import format_from_extension
from bernard_ledit.image import Image, guess_format

# --- load ---
# Accepts bytes only; the calling application handles file I/O.
img = Image(Path("photo.jpg").read_bytes())
img = Image(b"\xff\xd8\xff...")
print(img)  # Image(1920x1080, format:JPEG)
print(img.size)  # (1920, 1080)
print(img.format)  # 'JPEG'

# --- transform (both return a new Image) ---
cropped = img.crop(left=0, top=0, right=960, bottom=540)
resized = img.resize(640, 360)  # exact size, Lanczos filter

# --- encode ---
jpeg_bytes = img.encode("JPEG", jpeg_quality=85, jpeg_optimize=True)
png_bytes = img.encode("PNG")
pdf_bytes = img.encode("PDF")  # single-page PDF via DCTDecode
# JPEG Huffman tables are optimized by default; pass jpeg_optimize=False to disable.

# --- save ---
Path("out.jpg").write_bytes(img.encode(format_from_extension("jpg")))
Path("out.png").write_bytes(img.encode(format_from_extension("png")))
buffer = io.BytesIO()
buffer.write(img.encode("JPEG"))
Path("out.pdf").write_bytes(img.encode(format_from_extension("pdf")))

# --- helpers ---
fmt = guess_format(data)  # "JPEG" | "PNG" | "TIFF" | …

# --- compress to JPEG ---
# Re-encodes as JPEG. If dimensions are unchanged and the output isn't smaller
# than the source, raises ImageError.
out = img.compress_to_jpeg(
    quality=85,
    optimize=True,
    max_width=1024,  # aspect-preserving downscale, never upscales
    max_height=1024,
)

# --- crop by polygon ---
# Polygon points are normalized (x, y) coordinates (0..1).
crop = img.crop_polygon([(0.1, 0.1), (0.5, 0.1), (0.5, 0.4), (0.1, 0.4)])
Path("crop.jpg").write_bytes(crop.encode(format_from_extension("jpg")))
```

Supported encode formats: `JPEG`, `PNG`, `PDF`, `TIFF`, `WEBP`, `BMP`, `GIF`, `AVIF`, `ICO`, `QOI`,
`TGA`, `PNM`, `FARBFELD`, `OPENEXR`. HEIC support is decode-only; see
[image formats](../../docs/image-formats.md) for the scope and current status.

## PDF API

```python
import io
from pathlib import Path

from bernard_ledit.pdf import PdfDocument

# --- load ---
doc = PdfDocument(Path("document.pdf").read_bytes())
doc = PdfDocument(b"...")

# context-manager (auto-close)
with PdfDocument(Path("document.pdf").read_bytes()) as doc:
    print(len(doc))  # page count
    page = doc[0]  # or doc.get_page(0)
    print(page.text())  # extracted text
    image = page.render(scale=2.0)  # Image at 2× scale
    png_bytes = image.encode("PNG")

# --- rasterise a page to an image ---
rasterized_image = doc.get_page(0).render()
jpeg_bytes = rasterized_image.encode("JPEG", jpeg_quality=85)

# --- split / merge ---
a = PdfDocument(Path("a.pdf").read_bytes())
b = PdfDocument(Path("b.pdf").read_bytes())
out = PdfDocument.new()
out.import_pages(a, [0, 1])
out.import_pages(b, [0])

buf = io.BytesIO()
buf.write(out.to_bytes())
```

```python
from bernard_ledit.pdf import (
    PdfDocument,
    compress_to_jpeg,
    extract_from_page_ranges,
    fix_pdf,
    apply_page_options,
)

# --- compress to JPEG ---
# Only pages without a text layer (scans) are rasterised and re-encoded as JPEG.
# Returns the input unchanged if the result would not be smaller.
smaller = compress_to_jpeg(pdf_bytes, quality=85, optimize=True, dpi=150.0)
# The method form returns a new PdfDocument, or None when there is nothing to gain.
with PdfDocument(pdf_bytes) as doc:
    result = doc.compress_to_jpeg(quality=85, optimize=True, dpi=150.0)

# --- extract sub-documents ---
# `data` is a PDF or an image (treated as a one-page document).
# Page indexes are 0-based; returns one PdfDocument per group.
parts = extract_from_page_ranges(pdf_bytes, [[0, 1], [4]])

# --- keep or remove pages ---
# `operation` is "KEEP_ONLY" or "REMOVE". Negative indexes count from the end.
# Returns (document or None, skipped_indexes). None means the PDF has fewer
# than `on_min_pages` pages, so nothing was done.
doc, skipped = apply_page_options(
    pdf_bytes, operation="KEEP_ONLY", on_min_pages=0, page_indexes=[0, -1]
)

# --- repair a broken header ---
# Drops every byte before the first `%PDF-` marker, if it sits before maximum_offset.
fixed = fix_pdf(raw_bytes, maximum_offset=500)
```

## Development

Python 3.9+ and an active virtualenv are required, in addition to the
[shared development prerequisites](../../README.md#prerequisites).
Download the static PDFium archive for your platform as described in
[PDFium builds and targets](../../README.md#pdfium-builds). From the repository root:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install "maturin>=1.0,<2.0"
just python build        # statically links PDFium and installs the extension
just python wheel        # builds a distributable static wheel
just python test         # builds the extension before running tests
just python lint-check   # ruff and mypy
just python format
```

## Thread safety

PDFium rendering calls are serialized by a process-wide mutex.
