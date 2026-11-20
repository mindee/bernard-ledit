# Image file formats and encodings

This guide separates a file format from the encoding stored inside it. A format may be a
codec itself (for example, JPEG), or a container that can hold images encoded in several
ways (for example, TIFF, HEIF, and PDF).

The status below describes this repository's current `image` crate configuration and
decoder-only `libheif-rs` integration.

## Formats in the current image pipeline

The core depends on `image` with its default formats enabled. The `Image` API handles a
single decoded raster (`DynamicImage`), not animation frames, multipage documents, or
arbitrary image metadata.

| Format | Common extensions | What it can contain / common encoding | Decode in `Image::decode` | Encode with `Image::encode` |
| --- | --- | --- | --- | --- |
| BMP | `.bmp` | Raster pixels; commonly uncompressed, with some BMP variants using RLE | Yes | Yes |
| DDS | `.dds` | GPU texture surfaces; may use block compression such as BC/DXT | No through the current generic image path | No |
| GIF | `.gif` | Palette-based image data encoded with LZW; may contain animation | Yes, as one raster image | Yes, as one image; animation is not preserved by `Image` |
| HDR (Radiance) | `.hdr` | RGBE pixels; may use run-length encoding | Yes | Yes |
| ICO | `.ico` | Icon container with one or more BMP- or PNG-encoded images | Yes | Yes |
| JPEG | `.jpg`, `.jpeg` | Lossy DCT-based JPEG image | Yes | Yes; uses `jpeg-encoder`, with the API's quality option |
| OpenEXR | `.exr` | High-dynamic-range image; supports compression choices such as ZIP, PIZ, and RLE | Yes | Yes |
| PNG | `.png` | Lossless pixels using DEFLATE compression and per-row filters | Yes | Yes |
| PNM | `.pbm`, `.pgm`, `.ppm`, `.pam`, `.pnm` | Netpbm bitmap/graymap/pixmap/anymap; ASCII or binary pixel representations | Yes | Yes |
| QOI | `.qoi` | Lossless pixels using QOI run, index, difference, and literal chunks | Yes | Yes |
| TGA | `.tga` | Raster pixels; commonly raw or RLE | Yes | Yes |
| TIFF | `.tif`, `.tiff` | Container; supports multiple images and compression methods such as uncompressed, LZW, DEFLATE, PackBits, or JPEG | Yes, one raster image | Yes |
| WebP | `.webp` | VP8 lossy or VP8L lossless image data; can also carry animation | Yes, as one raster image | Yes, lossless encoding only |
| Farbfeld | `.ff` | Uncompressed 16-bit RGBA pixels | Yes | Yes |
| HEIC / HEIF (HEVC) | `.heic`, `.heif` | HEVC-coded primary image in a HEIF container | Yes, via pinned shared libheif 1.21.1 and a compatible external HEVC decoder plugin | No; HEIC encoding is unsupported |
| AVIF | `.avif` | AV1-coded image in the AVIF/HEIF container | Not with the current features: AV1 decoding requires `image`'s `avif-native` feature, which is not enabled | Yes, using `image`'s AVIF encoder |

These rows cover every active format in this project's `image::ImageFormat` path, the
HEIC path, and
the common encodings relevant to the project; some container standards permit additional
compression modes. `image::ImageFormat` does not support both reading and writing for
every format. The `DDS` codec exists in `image`, but the crate's generic
`ImageReader`/`ImageFormat` path does not currently enable it for this API. `image` also
has a hidden, deprecated `PCX` enum variant; it is not supported by this project's
generic decode or encode path.

The core's `OutputFormat::Pdf` is separate from raster formats. It creates a single-page
PDF with the image embedded as a JPEG stream (`DCTDecode`). PDF input and rendering use
PDFium; a PDF can contain text, vector graphics, and images encoded with different PDF
stream filters, such as Flate, DCT (JPEG), JPX (JPEG 2000), CCITT, and JBIG2.
`SourceFormat` includes `Image(ImageFormat)`, `Heic`, `Heif`, and `Pdf`, with canonical
names `HEIC`, `HEIF`, and `PDF` for the latter three. PDF remains a document input, not
a raster `Image::decode` format.

## HEIF, HEIC, and AVIF

HEIF is a container, not one particular image codec. HEIC and AVIF are related formats
that pair the container with different image codecs.

| Name | Relationship | Typical image codec | Extensions | Repository status |
| --- | --- | --- | --- | --- |
| HEIF | ISO Base Media File Format-based image container | Can carry HEVC, AVC, AV1, JPEG, JPEG 2000, uncompressed data, and other codecs, depending on the implementation | `.heif`, `.heic` | Decode support is limited to HEVC-coded images; no HEIF encoding |
| HEIC | HEIF with HEVC-coded images; "HEIC" commonly identifies HEVC image files | HEVC / H.265 | `.heic` (sometimes `.heif`) | Decoder-only support through `libheif-rs` |
| AVIF | An AV1-based image format using the HEIF container family | AV1 | `.avif` | AVIF encoding remains available through `image`; decoding is not enabled |

H.264 and H.265 are codec names, not file-format names:

| Codec name | Also called | Common use here |
| --- | --- | --- |
| H.264 | AVC | Can be stored in HEIF, but is not supported here |
| H.265 | HEVC | Used by typical HEIC files |
| AV1 | AV1 | Used by AVIF |

The `libheif` project lists the following codec backends. They are possibilities in the
library, not a guarantee that every installed system build contains each one.

| HEIF image type | Codec stored | `libheif` decoder backends | `libheif` encoder backends | This project's support |
| --- | --- | --- | --- | --- |
| HEIC | HEVC / H.265 | libde265, FFmpeg | x265, kvazaar | Decode only |
| AVIF | AV1 | libaom, dav1d, FFmpeg | libaom, rav1e, SVT-AV1 | No change: encoding via `image`; decoding not enabled |
| AVC-in-HEIF | AVC / H.264 | OpenH264, FFmpeg | x264 | Not planned |
| JPEG-in-HEIF | JPEG | libjpeg-turbo, FFmpeg | libjpeg-turbo | Not planned |
| JPEG 2000-in-HEIF | JPEG 2000 | OpenJPEG, FFmpeg | OpenJPEG | Not planned |
| VVC-in-HEIF | VVC / H.266 | vvdec, FFmpeg | vvenc, uvg266 | Not planned |
| HTJ2K-in-HEIF | High-throughput JPEG 2000 | OpenJPEG, FFmpeg | OpenJPH | Not planned |
| Uncompressed HEIF | ISO/IEC 23001-17 uncompressed image data | Built into `libheif` | Built into `libheif` | Not planned |

The integration decodes **HEVC-coded HEIC/HEIF only**: the primary image is
converted to RGB8, compositing alpha over white. Other images, animation, and metadata
are not preserved. HEIC encoding is explicitly unsupported; neither an HEVC encoder nor
support for every codec in the HEIF family is required. AVIF behavior does not change.

## System dependency and deployment

The decoder is always on and uses dynamically linked libheif 1.21.1, even for applications
that do not use HEIC. The Python wheel and PHP PIE asset bundle this shared library, but not
an HEVC decoder plugin. Development builds use the same pinned libheif version; build
environments need CMake and a C++ compiler, and need libde265 development files to build
the test plugin. Runtime environments need a compatible libde265 or FFmpeg plugin visible
to libheif. Architecture, libc, ABI, library install names, and plugin paths must match the
target.
If libheif is present but no HEVC decoder is available, decoding returns an error
directing the user to install either the libde265 or FFmpeg decoder plugin. If multiple
HEVC decoders are available, the integration tries them in priority order.
See the [README prerequisites](../README.md#native-libheif-prerequisites) for platform
examples and packaging limitations.

## Software license and patent notes

- The `libheif-rs` wrapper is MIT-licensed, but that does not determine the licenses of
  the native `libheif` library or its codec backends. Check the bundled libheif license
  and the exact plugin installed in each deployment environment.
- Decoder-only use avoids calling H.264/H.265 encoder APIs and should not require encoder
  plugins for the intended runtime configuration. It does **not** by itself resolve
  possible patent obligations for HEVC decoding; a system package may also include
  encoder plugins, so check its composition.
- Software copyright licenses and codec patent rights are separate questions. Review
  applicable terms for the actual codecs, regions, distribution, and backend use before
  production deployment; this guide is not legal advice.
- Some distribution packages include additional encoders or codecs even though this
  project requires neither; inspect the actual package and plugin composition.

## Where this is reflected in the code

- [`core/Cargo.toml`](../core/Cargo.toml) declares the `image` and JPEG encoder dependencies.
- [`core/src/common.rs`](../core/src/common.rs) defines output formats, including PDF;
  the source format type also includes PDF.
- [`core/src/image/image_data.rs`](../core/src/image/image_data.rs) implements image format
  parsing, decoding, and encoding.
- [`core/src/pdf/document.rs`](../core/src/pdf/document.rs) handles PDF operations.

## References

- [`image` 0.25.10 codec support](https://docs.rs/image/0.25.10/image/codecs/index.html)
- [`image::ImageFormat`](https://docs.rs/image/0.25.10/image/enum.ImageFormat.html)
- [`libheif` supported formats and codec backends](https://github.com/strukturag/libheif#supported-features)
- [`libheif-rs` system requirements and `image` integration](https://docs.rs/libheif-rs/latest/libheif_rs/)
