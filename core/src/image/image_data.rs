use crate::image::ImageError;
use ::image::imageops::FilterType;
use ::image::{DynamicImage, ImageFormat};
use image::ImageReader;
use log::warn;
use std::io::Cursor;

/// A decoded raster image plus the format it was decoded from (if known).
pub struct Image {
    inner: DynamicImage,
    format: Option<ImageFormat>,
}

/// Thin wrapper for resize filters, so the string->enum parse lives in core.
pub struct Filter(pub FilterType);

/// A target encode format: any raster [`ImageFormat`], or PDF.
///
/// PDF is not a raster format the `image` crate knows about, so it lives here
/// alongside the raster formats to give [`Image::encode`] a single entry point
/// for every output the SDK can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// A raster format handled by the `image` crate (PNG, JPEG, WEBP, ...).
    Image(ImageFormat),
    /// A single-page PDF embedding the image as a JPEG.
    Pdf,
}

impl From<ImageFormat> for OutputFormat {
    fn from(format: ImageFormat) -> Self {
        Self::Image(format)
    }
}

/// Magic-byte sniff. Backs the public `guess_format`.
/// # Errors
/// Returns an error if the image format cannot be determined.
pub fn guess_format(data: &[u8]) -> Result<ImageFormat, ImageError> {
    image::guess_format(data).map_err(|e| ImageError::UnsupportedFormat(e.to_string()))
}

/// Stable uppercase name: JPEG, PNG, GIF, WEBP, TIFF, BMP, ICO, ... .
/// Centralizes the `ImageFormat`->&str mapping used by both `guess_format`
/// and `Image::format`.
#[must_use]
pub const fn format_name(format: ImageFormat) -> Option<&'static str> {
    match format {
        ImageFormat::Avif => Some("AVIF"),
        ImageFormat::Bmp => Some("BMP"),
        ImageFormat::Dds => Some("DDS"),
        ImageFormat::Farbfeld => Some("FARBFELD"),
        ImageFormat::Gif => Some("GIF"),
        ImageFormat::Hdr => Some("HDR"),
        ImageFormat::Ico => Some("ICO"),
        ImageFormat::Jpeg => Some("JPEG"),
        ImageFormat::OpenExr => Some("OPENEXR"),
        ImageFormat::Png => Some("PNG"),
        ImageFormat::Pnm => Some("PNM"),
        ImageFormat::Qoi => Some("QOI"),
        ImageFormat::Tga => Some("TGA"),
        ImageFormat::Tiff => Some("TIFF"),
        ImageFormat::WebP => Some("WEBP"),
        _ => None,
    }
}

/// Parse an uppercase/lowercase format string into an `ImageFormat`.
/// # Errors
/// Returns `ImageError::UnsupportedFormat` on miss.
pub fn parse_format(name: &str) -> Result<ImageFormat, ImageError> {
    match name {
        s if s.eq_ignore_ascii_case("AVIF") => Ok(ImageFormat::Avif),
        s if s.eq_ignore_ascii_case("BMP") => Ok(ImageFormat::Bmp),
        s if s.eq_ignore_ascii_case("DDS") => Ok(ImageFormat::Dds),
        s if s.eq_ignore_ascii_case("FARBFELD") => Ok(ImageFormat::Farbfeld),
        s if s.eq_ignore_ascii_case("GIF") => Ok(ImageFormat::Gif),
        s if s.eq_ignore_ascii_case("HDR") => Ok(ImageFormat::Hdr),
        s if s.eq_ignore_ascii_case("ICO") => Ok(ImageFormat::Ico),
        s if s.eq_ignore_ascii_case("JPEG") => Ok(ImageFormat::Jpeg),
        s if s.eq_ignore_ascii_case("OPENEXR") => Ok(ImageFormat::OpenExr),
        s if s.eq_ignore_ascii_case("PNG") => Ok(ImageFormat::Png),
        s if s.eq_ignore_ascii_case("PNM") => Ok(ImageFormat::Pnm),
        s if s.eq_ignore_ascii_case("QOI") => Ok(ImageFormat::Qoi),
        s if s.eq_ignore_ascii_case("TGA") => Ok(ImageFormat::Tga),
        s if s.eq_ignore_ascii_case("TIFF") => Ok(ImageFormat::Tiff),
        s if s.eq_ignore_ascii_case("WEBP") => Ok(ImageFormat::WebP),
        _ => Err(ImageError::UnsupportedFormat(format!(
            "Unknown format: {name}"
        ))),
    }
}

/// Parse a format string into an [`OutputFormat`], accepting `"PDF"` in addition
/// to every raster name understood by [`parse_format`].
/// # Errors
/// Returns `ImageError::UnsupportedFormat` on miss.
pub fn parse_output_format(name: &str) -> Result<OutputFormat, ImageError> {
    if name.eq_ignore_ascii_case("PDF") {
        return Ok(OutputFormat::Pdf);
    }
    parse_format(name).map(OutputFormat::Image)
}
impl std::str::FromStr for Filter {
    type Err = ImageError;

    /// "lanczos" | "lanczos3" -> Lanczos3, "nearest", "triangle",
    /// "catmullrom"/"bicubic", "gaussian". Default caller passes "lanczos".
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            v if v.eq_ignore_ascii_case("lanczos") || v.eq_ignore_ascii_case("lanczos3") => {
                Ok(Self(FilterType::Lanczos3))
            }
            v if v.eq_ignore_ascii_case("nearest") => Ok(Self(FilterType::Nearest)),
            v if v.eq_ignore_ascii_case("triangle") => Ok(Self(FilterType::Triangle)),
            v if v.eq_ignore_ascii_case("catmullrom") || v.eq_ignore_ascii_case("bicubic") => {
                Ok(Self(FilterType::CatmullRom))
            }
            v if v.eq_ignore_ascii_case("gaussian") => Ok(Self(FilterType::Gaussian)),
            _ => Err(ImageError::UnknownFilter(format!("Unknown filter: {s}"))),
        }
    }
}

impl Image {
    /// Decode any supported raster and records the sniffed format.
    /// # Errors
    /// Returns an error if the image data is invalid or the format is not supported.
    pub fn decode(data: &[u8]) -> Result<Self, ImageError> {
        let inner = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .map_err(|e| ImageError::Decode(e.to_string()))?;
        let format = inner.format();
        let decoded = inner
            .decode()
            .map_err(|e| ImageError::Decode(e.to_string()))?;
        Ok(Self {
            inner: decoded,
            format,
        })
    }

    /// (width, height).
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    /// Format this image was decoded from, if known.
    #[must_use]
    pub const fn format(&self) -> Option<ImageFormat> {
        self.format
    }

    /// PIL-style crop by pixel box.
    /// # Errors
    /// Returns `ImageError::InvalidCrop` if the crop is invalid.
    pub fn crop(&self, left: u32, top: u32, right: u32, bottom: u32) -> Result<Self, ImageError> {
        let (img_width, img_height) = self.size();
        if left >= right || top >= bottom || right > img_width || bottom > img_height {
            return Err(ImageError::InvalidCrop {
                left,
                top,
                right,
                bottom,
                width: img_width,
                height: img_height,
            });
        }
        let crop_width = right - left;
        let crop_height = bottom - top;
        let cropped_inner = self.inner.crop_imm(left, top, crop_width, crop_height);
        Ok(Self {
            inner: cropped_inner,
            format: self.format,
        })
    }

    /// Exact resize to (width, height) with the given filter.
    /// Uses `DynamicImage::resize_exact`. Preserves `format`.
    /// # Errors
    /// Returns `ImageError::InvalidDimensions` if the dimensions are invalid.
    pub fn resize(&self, width: u32, height: u32, filter: FilterType) -> Result<Self, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::InvalidDimensions { width, height });
        }
        let resized_inner = self.inner.resize_exact(width, height, filter);
        Ok(Self {
            inner: resized_inner,
            format: self.format,
        })
    }

    /// Encode to `format` at `quality` (quality only affects JPEG).
    ///
    /// Accepts any raster [`ImageFormat`] or [`OutputFormat::Pdf`]. Choosing PDF
    /// produces a single-page document that embeds the image as a JPEG, so no
    /// separate trip through the `pdf` module is needed.
    /// TODO: `optimize` is accepted at the language layer but ignored.
    /// # Errors
    /// Returns `ImageError::Encode` if encoding fails or `ImageError::InvalidDimensions`
    /// if the dimensions are invalid.
    pub fn encode(
        &self,
        format: impl Into<OutputFormat>,
        quality: u8,
        optimize: bool,
    ) -> Result<Vec<u8>, ImageError> {
        if optimize {
            warn!("optimize=true ignored for Image::encode");
        }
        match format.into() {
            OutputFormat::Pdf => self.encode_pdf(quality),
            OutputFormat::Image(image_format) => self.encode_raster(image_format, quality),
        }
    }

    /// Encode to a raster `format` at `quality` (quality only affects JPEG).
    fn encode_raster(&self, format: ImageFormat, quality: u8) -> Result<Vec<u8>, ImageError> {
        let mut cursor = Cursor::new(Vec::new());
        match format {
            ImageFormat::Jpeg => {
                if !(1..=100).contains(&quality) {
                    return Err(ImageError::Encode(format!(
                        "invalid JPEG quality: {quality} (expected 1..=100)"
                    )));
                }
                let rgb_image = self.inner.to_rgb8();
                let mut encoder =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
                encoder
                    .encode(
                        rgb_image.as_raw(),
                        rgb_image.width(),
                        rgb_image.height(),
                        image::ExtendedColorType::Rgb8,
                    )
                    .map_err(|e| ImageError::Encode(e.to_string()))?;
            }
            _ => {
                self.inner
                    .write_to(&mut cursor, format)
                    .map_err(|e| ImageError::Encode(e.to_string()))?;
            }
        }
        Ok(cursor.into_inner())
    }

    /// Encode as a single-page PDF that embeds the image as a JPEG via
    /// `/Filter /DCTDecode`. The page maps one image pixel to one PDF point.
    fn encode_pdf(&self, quality: u8) -> Result<Vec<u8>, ImageError> {
        let jpeg_bytes = self.encode_raster(ImageFormat::Jpeg, quality)?;
        crate::pdf::jpeg::build_jpeg_pdf_auto_size(&jpeg_bytes)
            .map_err(|e| ImageError::Encode(e.to_string()))
    }
}

/// Aspect-preserving downscale.
///
/// Fits the image within `max_width`/`max_height` while preserving aspect
/// ratio (Pillow `thumbnail` semantics). Bounds are clamped to the current
/// dimensions so the image is never upscaled. Uses `Lanczos3`.
#[must_use]
pub fn downscale_to_fit(img: &Image, max_width: Option<u32>, max_height: Option<u32>) -> Image {
    let (width, height) = img.size();
    // Clamp each bound to the current size so `resize` (which fits within the
    // box using the smaller of the two ratios) can only ever scale down.
    let bound_w = max_width.map_or(width, |w| w.clamp(1, width));
    let bound_h = max_height.map_or(height, |h| h.clamp(1, height));

    if bound_w == width && bound_h == height {
        return Image {
            inner: img.inner.clone(),
            format: img.format,
        };
    }

    Image {
        inner: img.inner.resize(bound_w, bound_h, FilterType::Lanczos3),
        format: img.format,
    }
}

/// Compresses image data by decoding, optionally downscaling, and re-encoding as JPEG.
/// Returns (`jpeg_bytes`, `width`, `height`).
/// # Errors
/// Returns an error if the image cannot be decoded, resized, or encoded.
pub fn compress(
    data: &[u8],
    quality: u8,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> Result<(Vec<u8>, u32, u32), ImageError> {
    let img = Image::decode(data)?;
    let scaled = downscale_to_fit(&img, max_width, max_height);
    let jpeg_bytes = scaled.encode(ImageFormat::Jpeg, quality, false)?;
    let (w, h) = scaled.size();
    Ok((jpeg_bytes, w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::{ImageFormat, RgbImage};
    use std::io::Cursor;
    use std::str::FromStr;

    /// Build deterministic raster bytes of a solid-colour image in `format`,
    /// so tests can assert exact dimensions without depending on fixture files.
    fn synth_bytes(width: u32, height: u32, format: ImageFormat) -> Vec<u8> {
        let img = RgbImage::from_pixel(width, height, ::image::Rgb([120, 200, 40]));
        let dynamic = DynamicImage::ImageRgb8(img);
        let mut cursor = Cursor::new(Vec::new());
        dynamic
            .write_to(&mut cursor, format)
            .expect("failed to encode synthetic image");
        cursor.into_inner()
    }

    #[test]
    fn guess_format_png() {
        let bytes = synth_bytes(4, 4, ImageFormat::Png);
        assert_eq!(guess_format(&bytes).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn guess_format_jpeg_from_fixture() {
        let bytes = test_data_bytes!("file_types/receipt.jpg");
        assert_eq!(guess_format(bytes).unwrap(), ImageFormat::Jpeg);
    }

    #[test]
    fn guess_format_tiff_from_fixture() {
        let bytes = test_data_bytes!("file_types/receipt.tiff");
        assert_eq!(guess_format(bytes).unwrap(), ImageFormat::Tiff);
    }

    #[test]
    fn guess_format_garbage_errors() {
        assert!(matches!(
            guess_format(&[0x00, 0x01, 0x02, 0x03]),
            Err(ImageError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn guess_format_empty_errors() {
        assert!(guess_format(&[]).is_err());
    }

    #[test]
    fn format_name_known_formats() {
        assert_eq!(format_name(ImageFormat::Jpeg), Some("JPEG"));
        assert_eq!(format_name(ImageFormat::Png), Some("PNG"));
        assert_eq!(format_name(ImageFormat::WebP), Some("WEBP"));
        assert_eq!(format_name(ImageFormat::Tiff), Some("TIFF"));
    }

    #[test]
    fn parse_format_case_insensitive() {
        assert_eq!(parse_format("jpeg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(parse_format("JPEG").unwrap(), ImageFormat::Jpeg);
        assert_eq!(parse_format("Png").unwrap(), ImageFormat::Png);
        assert_eq!(parse_format("webp").unwrap(), ImageFormat::WebP);
    }

    #[test]
    fn parse_format_unknown_errors() {
        assert!(matches!(
            parse_format("not-a-format"),
            Err(ImageError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn filter_lanczos_maps_to_lanczos3() {
        assert!(matches!(
            Filter::from_str("lanczos").unwrap().0,
            FilterType::Lanczos3
        ));
        assert!(matches!(
            Filter::from_str("LANCZOS3").unwrap().0,
            FilterType::Lanczos3
        ));
    }

    #[test]
    fn filter_all_known_variants() {
        assert!(matches!(
            Filter::from_str("nearest").unwrap().0,
            FilterType::Nearest
        ));
        assert!(matches!(
            Filter::from_str("triangle").unwrap().0,
            FilterType::Triangle
        ));
        assert!(matches!(
            Filter::from_str("catmullrom").unwrap().0,
            FilterType::CatmullRom
        ));
        assert!(matches!(
            Filter::from_str("bicubic").unwrap().0,
            FilterType::CatmullRom
        ));
        assert!(matches!(
            Filter::from_str("gaussian").unwrap().0,
            FilterType::Gaussian
        ));
    }

    #[test]
    fn filter_unknown_errors() {
        assert!(matches!(
            Filter::from_str("sinc"),
            Err(ImageError::UnknownFilter(_))
        ));
    }

    #[test]
    fn decode_png_records_size_and_format() {
        let bytes = synth_bytes(20, 10, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        assert_eq!(img.size(), (20, 10));
        assert_eq!(img.format(), Some(ImageFormat::Png));
    }

    #[test]
    fn decode_jpeg_fixture() {
        let bytes = test_data_bytes!("file_types/receipt.jpg");
        let img = Image::decode(bytes).unwrap();
        let (w, h) = img.size();
        assert!(w > 0 && h > 0);
        assert_eq!(img.format(), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn decode_garbage_errors() {
        assert!(matches!(
            Image::decode(&[0xDE, 0xAD, 0xBE, 0xEF]),
            Err(ImageError::Decode(_))
        ));
    }

    #[test]
    fn crop_valid_box() {
        let bytes = synth_bytes(100, 80, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let cropped = img.crop(10, 20, 60, 70).unwrap();
        assert_eq!(cropped.size(), (50, 50));
    }

    #[test]
    fn crop_preserves_format() {
        let bytes = synth_bytes(40, 40, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let cropped = img.crop(0, 0, 10, 10).unwrap();
        assert_eq!(cropped.format(), Some(ImageFormat::Png));
    }

    #[test]
    fn crop_out_of_bounds_errors() {
        let bytes = synth_bytes(30, 30, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        assert!(matches!(
            img.crop(0, 0, 31, 10),
            Err(ImageError::InvalidCrop { .. })
        ));
    }

    #[test]
    fn crop_inverted_box_errors() {
        let bytes = synth_bytes(30, 30, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        assert!(matches!(
            img.crop(20, 5, 10, 25),
            Err(ImageError::InvalidCrop { .. })
        ));
    }

    #[test]
    fn crop_zero_size_box_errors() {
        let bytes = synth_bytes(30, 30, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        assert!(matches!(
            img.crop(5, 5, 5, 20),
            Err(ImageError::InvalidCrop { .. })
        ));
    }

    #[test]
    fn resize_exact_dimensions() {
        let bytes = synth_bytes(40, 40, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let resized = img.resize(80, 20, FilterType::Lanczos3).unwrap();
        assert_eq!(resized.size(), (80, 20));
    }

    #[test]
    fn resize_zero_width_errors() {
        let bytes = synth_bytes(40, 40, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        assert!(matches!(
            img.resize(0, 20, FilterType::Lanczos3),
            Err(ImageError::InvalidDimensions { .. })
        ));
    }

    #[test]
    fn resize_zero_height_errors() {
        let bytes = synth_bytes(40, 40, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        assert!(matches!(
            img.resize(20, 0, FilterType::Lanczos3),
            Err(ImageError::InvalidDimensions { .. })
        ));
    }

    #[test]
    fn encode_jpeg_has_magic_bytes() {
        let bytes = synth_bytes(16, 16, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = img.encode(ImageFormat::Jpeg, 85, false).unwrap();
        assert_eq!(&out[0..3], &[0xFF, 0xD8, 0xFF]);
    }

    #[test]
    fn encode_png_has_magic_bytes() {
        let bytes = synth_bytes(16, 16, ImageFormat::Jpeg);
        let img = Image::decode(&bytes).unwrap();
        let out = img.encode(ImageFormat::Png, 85, false).unwrap();
        assert_eq!(&out[0..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn encode_optimize_flag_is_noop() {
        let bytes = synth_bytes(16, 16, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let a = img.encode(ImageFormat::Jpeg, 85, true).unwrap();
        let b = img.encode(ImageFormat::Jpeg, 85, false).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn encode_jpeg_roundtrips_to_decodable() {
        let bytes = synth_bytes(24, 12, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let jpeg = img.encode(ImageFormat::Jpeg, 90, false).unwrap();
        let reloaded = Image::decode(&jpeg).unwrap();
        assert_eq!(reloaded.size(), (24, 12));
        assert_eq!(reloaded.format(), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn encode_pdf_has_magic_bytes_and_embeds_jpeg() {
        let bytes = synth_bytes(24, 12, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let pdf = img.encode(OutputFormat::Pdf, 85, false).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.4\n"), "PDF header missing");
        assert!(pdf.ends_with(b"%%EOF\n"), "PDF %%EOF marker missing");
        assert!(
            String::from_utf8_lossy(&pdf).contains("/Filter /DCTDecode"),
            "embedded JPEG (DCTDecode) missing"
        );
    }

    #[test]
    fn encode_pdf_media_box_matches_image_size() {
        let bytes = synth_bytes(40, 25, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let pdf = img.encode(OutputFormat::Pdf, 85, false).unwrap();
        assert!(
            String::from_utf8_lossy(&pdf).contains("/MediaBox [0 0 40 25]"),
            "PDF page size should map 1 pixel to 1 point"
        );
    }

    #[test]
    fn encode_pdf_via_string_format() {
        let bytes = synth_bytes(16, 16, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let format = parse_output_format("pdf").unwrap();
        assert_eq!(format, OutputFormat::Pdf);
        let pdf = img.encode(format, 85, false).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.4\n"));
    }

    #[test]
    fn encode_pdf_rejects_invalid_quality() {
        let bytes = synth_bytes(16, 16, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        assert!(matches!(
            img.encode(OutputFormat::Pdf, 0, false),
            Err(ImageError::Encode(_))
        ));
    }

    #[test]
    fn parse_output_format_pdf_case_insensitive() {
        assert_eq!(parse_output_format("PDF").unwrap(), OutputFormat::Pdf);
        assert_eq!(parse_output_format("Pdf").unwrap(), OutputFormat::Pdf);
        assert_eq!(
            parse_output_format("jpeg").unwrap(),
            OutputFormat::Image(ImageFormat::Jpeg)
        );
    }

    #[test]
    fn parse_output_format_unknown_errors() {
        assert!(matches!(
            parse_output_format("not-a-format"),
            Err(ImageError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn downscale_never_upscales_when_bounds_larger() {
        let bytes = synth_bytes(50, 50, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = downscale_to_fit(&img, Some(500), Some(500));
        assert_eq!(out.size(), (50, 50));
    }

    #[test]
    fn downscale_none_bounds_keeps_size() {
        let bytes = synth_bytes(50, 40, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = downscale_to_fit(&img, None, None);
        assert_eq!(out.size(), (50, 40));
    }

    #[test]
    fn downscale_preserves_aspect_ratio() {
        let bytes = synth_bytes(200, 100, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = downscale_to_fit(&img, Some(100), Some(100));
        assert_eq!(out.size(), (100, 50));
    }

    #[test]
    fn downscale_only_width_bound() {
        let bytes = synth_bytes(200, 100, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        // scale = 50/200 = 0.25
        let out = downscale_to_fit(&img, Some(50), None);
        assert_eq!(out.size(), (50, 25));
    }

    #[test]
    fn downscale_floors_dimensions_to_at_least_one() {
        let bytes = synth_bytes(100, 4, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = downscale_to_fit(&img, Some(2), None);
        let (_, h) = out.size();
        assert!(h >= 1);
    }

    #[test]
    fn compress_returns_jpeg_and_dimensions() {
        let bytes = synth_bytes(200, 100, ImageFormat::Png);
        let (jpeg, w, h) = compress(&bytes, 85, Some(100), Some(100)).unwrap();
        assert_eq!(&jpeg[0..3], &[0xFF, 0xD8, 0xFF]);
        assert_eq!((w, h), (100, 50));
    }

    #[test]
    fn compress_does_not_upscale() {
        let bytes = synth_bytes(30, 30, ImageFormat::Png);
        let (_, w, h) = compress(&bytes, 85, Some(500), Some(500)).unwrap();
        assert_eq!((w, h), (30, 30));
    }

    #[test]
    fn compress_no_bounds_keeps_dimensions() {
        let bytes = synth_bytes(64, 48, ImageFormat::Png);
        let (_, w, h) = compress(&bytes, 85, None, None).unwrap();
        assert_eq!((w, h), (64, 48));
    }

    #[test]
    fn compress_garbage_errors() {
        assert!(compress(&[0x00, 0x01], 85, None, None).is_err());
    }

    #[test]
    fn compress_fixture_jpeg() {
        let bytes = test_data_bytes!("file_types/receipt.jpg");
        let (jpeg, w, h) = compress(bytes, 80, Some(512), Some(512)).unwrap();
        assert_eq!(&jpeg[0..3], &[0xFF, 0xD8, 0xFF]);
        assert!(w <= 512 && h <= 512);
    }
}
