use crate::common::{Format, OutputFormat, guess_format};
use crate::image::ImageError;
use ::image::imageops::FilterType;
use ::image::{DynamicImage, ImageFormat};
use image::{ImageDecoder, ImageReader};
use jpeg_encoder::{ColorType as JpegColorType, Encoder as JpegEncoder};
use std::io::Cursor;

/// A decoded raster image plus the format it was decoded from (if known).
pub struct Image {
    pub(super) inner: DynamicImage,
    pub(super) format: Option<Format>,
    /// corresponds to length of encoded image data if format is not `None`.
    /// or length of raw pixel data if format is `None`.
    pub(super) source_len: Option<usize>,
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

impl Image {
    /// Decode any supported raster and records the sniffed format.
    /// # Errors
    /// Returns an error if the image data is invalid or the format is not supported.
    pub fn decode(data: &[u8]) -> Result<Self, ImageError> {
        if let Ok(format @ (Format::Heic | Format::Heif)) = guess_format(data) {
            return Ok(Self {
                inner: super::heif::decode(data)?,
                format: Some(format),
                source_len: Some(data.len()),
            });
        }
        let inner = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .map_err(|e| ImageError::Decode(e.to_string()))?;
        let source_len = Some(data.len());
        let format = inner.format().map(Format::Image);
        let mut decoder = inner
            .into_decoder()
            .map_err(|e| ImageError::Decode(e.to_string()))?;
        let orientation = decoder
            .orientation()
            .map_err(|e| ImageError::Decode(e.to_string()))?;
        let mut raster =
            DynamicImage::from_decoder(decoder).map_err(|e| ImageError::Decode(e.to_string()))?;
        raster.apply_orientation(orientation);
        Ok(Self {
            inner: raster,
            format,
            source_len,
        })
    }

    /// Return bytes as an u8 vec.
    /// The bytes can correspond to the original encoded image data.
    /// or to the raw pixel data of the decoded image if the original format is not available.
    /// # Errors
    /// This operation currently never returns an error.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ImageError> {
        Ok(self.inner.as_bytes().to_vec())
    }
}

impl Image {
    /// Creates an `ImageData` from a `DynamicImage`.
    pub(crate) fn from_dynamic(inner: DynamicImage) -> Self {
        let source_len = Some(inner.as_bytes().len());
        Self {
            inner,
            format: None,
            source_len,
        }
    }

    /// (width, height).
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    /// Format this image was decoded from, if known.
    #[must_use]
    pub const fn format(&self) -> Option<Format> {
        self.format
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
            source_len: self.source_len,
        })
    }

    /// Aspect-preserving downscale.
    ///
    /// Fits the image within `max_width`/`max_height` while preserving aspect
    /// ratio (Pillow `thumbnail` semantics). Bounds are clamped to the current
    /// dimensions so the image is never upscaled. Uses `Lanczos3`.
    #[must_use]
    pub fn downscale_to_fit(&self, max_width: Option<u32>, max_height: Option<u32>) -> Self {
        let (width, height) = self.size();
        let bound_w = max_width.map_or(width, |w| w.clamp(1, width));
        let bound_h = max_height.map_or(height, |h| h.clamp(1, height));

        if bound_w == width && bound_h == height {
            return Self {
                inner: self.inner.clone(),
                format: self.format,
                source_len: self.source_len,
            };
        }

        Self {
            inner: self.inner.resize(bound_w, bound_h, FilterType::Lanczos3),
            format: self.format,
            source_len: self.source_len,
        }
    }

    /// Encode to `format` at `jpeg_quality` (`jpeg_quality` and `jpeg_optimize` affect JPEG only).
    ///
    /// Accepts any raster [`ImageFormat`] or [`OutputFormat::Pdf`]. Choosing PDF
    /// produces a single-page document that embeds the image as a JPEG.
    /// `jpeg_quality` and `jpeg_optimize` only affect JPEG encoding and PDF encoding
    /// because the image is embedded as a JPEG in the PDF.
    /// # Errors
    /// Returns `ImageError::Encode` if encoding fails or `ImageError::InvalidDimensions`
    /// if the dimensions are invalid.
    pub fn encode(
        &self,
        format: impl Into<OutputFormat>,
        jpeg_quality: u8,
        jpeg_optimize: bool,
    ) -> Result<Vec<u8>, ImageError> {
        match format.into() {
            OutputFormat::Pdf => self.encode_pdf(jpeg_quality, jpeg_optimize),
            OutputFormat::Image(image_format) => {
                self.encode_raster(image_format, jpeg_quality, jpeg_optimize)
            }
        }
    }

    /// Encode to a raster `format` at `jpeg_quality` (`jpeg_quality` and `jpeg_optimize` only affect JPEG).
    fn encode_raster(
        &self,
        format: ImageFormat,
        jpeg_quality: u8,
        jpeg_optimize: bool,
    ) -> Result<Vec<u8>, ImageError> {
        if format == ImageFormat::Jpeg {
            if !(1..=100).contains(&jpeg_quality) {
                return Err(ImageError::Encode(format!(
                    "invalid jpeg_quality: {jpeg_quality} (expected 1..=100)"
                )));
            }
            let rgb_image = self.inner.to_rgb8();
            let width = u16::try_from(rgb_image.width()).map_err(|_| {
                ImageError::Encode(format!(
                    "JPEG width exceeds maximum dimension of {}",
                    u16::MAX
                ))
            })?;
            let height = u16::try_from(rgb_image.height()).map_err(|_| {
                ImageError::Encode(format!(
                    "JPEG height exceeds maximum dimension of {}",
                    u16::MAX
                ))
            })?;
            let mut jpeg = Vec::new();
            let mut encoder = JpegEncoder::new(&mut jpeg, jpeg_quality);
            encoder.set_optimized_huffman_tables(jpeg_optimize);
            encoder
                .encode(rgb_image.as_raw(), width, height, JpegColorType::Rgb)
                .map_err(|e| ImageError::Encode(e.to_string()))?;
            Ok(jpeg)
        } else {
            let mut cursor = Cursor::new(Vec::new());
            self.inner
                .write_to(&mut cursor, format)
                .map_err(|e| ImageError::Encode(e.to_string()))?;
            Ok(cursor.into_inner())
        }
    }

    /// Encode as a single-page PDF that embeds the image as a JPEG via
    /// `/Filter /DCTDecode`. The page maps one image pixel to one PDF point.
    fn encode_pdf(&self, jpeg_quality: u8, jpeg_optimize: bool) -> Result<Vec<u8>, ImageError> {
        let jpeg_bytes = self.encode_raster(ImageFormat::Jpeg, jpeg_quality, jpeg_optimize)?;
        let document = crate::pdf::document::Document::from_jpeg_autosize(&jpeg_bytes)
            .map_err(|e| ImageError::Encode(e.to_string()))?;
        document
            .to_bytes()
            .map_err(|e| ImageError::Encode(e.to_string()))
    }

    /// Compresses this image to JPEG until the target JPEG quality is reached.
    /// Returns (`jpeg_bytes`, `width`, `height`).
    /// # Errors
    /// Returns an error if the image cannot be decoded, resized, or encoded.
    pub fn compress_to_jpeg(
        &self,
        quality: u8,
        optimize: bool,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> Result<(Vec<u8>, u32, u32), ImageError> {
        if quality < 5 {
            return Err(ImageError::ProgressiveQualityTooLow {
                jpeg_quality: quality,
            });
        }
        let scaled = self.downscale_to_fit(max_width, max_height);
        let (w, h) = scaled.size();
        let budget = if (w, h) == self.size() {
            self.source_len
        } else {
            None
        };

        let mut q = quality;
        loop {
            let jpeg = scaled.encode(ImageFormat::Jpeg, q, optimize)?;
            if let Some(initial_size) = budget.filter(|&b| jpeg.len() >= b) {
                if !optimize || q < 10 {
                    return Err(ImageError::SizeIncreased {
                        compressed_size: jpeg.len(),
                        initial_size,
                    });
                }
                q -= 5;
            } else {
                return Ok((jpeg, w, h));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{format_name, parse_output_format};
    use ::image::{ImageFormat, RgbImage};
    use std::io::Cursor;

    fn compress_to_jpeg(
        data: &[u8],
        quality: u8,
        optimize: bool,
        mw: Option<u32>,
        mh: Option<u32>,
    ) -> Result<(Vec<u8>, u32, u32), ImageError> {
        let img = Image::decode(data)?;
        img.compress_to_jpeg(quality, optimize, mw, mh)
    }

    /// Build deterministic raster bytes of a solid-colour image in `format`,
    /// so tests can assert exact dimensions without depending on fixture files.
    fn synth_bytes(width: u32, height: u32, format: ImageFormat) -> Vec<u8> {
        let img = RgbImage::from_pixel(width, height, image::Rgb([120, 200, 40]));
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
        assert_eq!(
            guess_format(&bytes).unwrap(),
            Format::Image(ImageFormat::Png)
        );
    }

    #[test]
    fn guess_format_jpeg_from_fixture() {
        let bytes = test_data_bytes!("file_types/receipt.jpg");
        assert_eq!(
            guess_format(bytes).unwrap(),
            Format::Image(ImageFormat::Jpeg)
        );
    }

    #[test]
    fn guess_format_tiff_from_fixture() {
        let bytes = test_data_bytes!("file_types/receipt.tiff");
        assert_eq!(
            guess_format(bytes).unwrap(),
            Format::Image(ImageFormat::Tiff)
        );
    }

    #[test]
    fn guess_format_garbage_errors() {
        assert!(matches!(
            guess_format(&[0x00, 0x01, 0x02, 0x03]),
            Err(ImageError::UnknownFormat)
        ));
    }

    #[test]
    fn guess_format_empty_errors() {
        assert!(guess_format(&[]).is_err());
    }

    #[test]
    fn format_name_known_formats() {
        assert_eq!(format_name(ImageFormat::Jpeg.into()), Some("JPEG"));
        assert_eq!(format_name(ImageFormat::Png.into()), Some("PNG"));
        assert_eq!(format_name(ImageFormat::WebP.into()), Some("WEBP"));
        assert_eq!(format_name(ImageFormat::Tiff.into()), Some("TIFF"));
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
    fn decode_png_records_size_and_format() {
        let bytes = synth_bytes(20, 10, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        assert_eq!(img.size(), (20, 10));
        assert_eq!(img.format(), Some(ImageFormat::Png.into()));
    }

    #[test]
    fn decode_jpeg_fixture() {
        let bytes = test_data_bytes!("file_types/receipt.jpg");
        let img = Image::decode(bytes).unwrap();
        let (w, h) = img.size();
        assert!(w > 0 && h > 0);
        assert_eq!(img.format(), Some(ImageFormat::Jpeg.into()));
    }

    #[test]
    fn decode_heic_to_rgb_and_reencode_as_png() {
        let bytes = test_data_bytes!("file_types/receipt.heic");
        let image = Image::decode(bytes).unwrap();
        assert_eq!(image.size(), (800, 1066));
        assert_eq!(image.format(), Some(Format::Heic));
        assert_eq!(image.inner.color(), image::ColorType::Rgb8);
        assert_eq!(image.source_len, Some(bytes.len()));
        let resized = image.resize(80, 107, FilterType::Lanczos3).unwrap();
        assert_eq!(resized.format(), Some(Format::Heic));
        let png = resized.encode(ImageFormat::Png, 85, true).unwrap();
        assert_eq!(Image::decode(&png).unwrap().size(), (80, 107));
    }

    #[test]
    fn decode_heic_grid_composites_alpha_on_white() {
        // Synthetic 128x64 lossless HEVC grid, two 64x64 tiles with alpha strips.
        let bytes = include_bytes!("../../tests/fixtures/alpha-grid.heic");
        let image = Image::decode(bytes).unwrap();
        assert_eq!(image.size(), (128, 64));
        let rgb = image.inner.as_rgb8().unwrap();
        assert_eq!(rgb.get_pixel(10, 10).0, [255, 255, 255]);
        assert_eq!(rgb.get_pixel(40, 10).0, [227, 177, 127]);
        assert_eq!(rgb.get_pixel(90, 10).0, [200, 100, 0]);
    }

    #[test]
    fn decode_hevc_with_generic_heif_brand() {
        let mut bytes = test_data_bytes!("file_types/receipt.heic").to_vec();
        let ftyp_len = usize::try_from(u32::from_be_bytes(bytes[..4].try_into().unwrap())).unwrap();
        for brand in bytes[8..ftyp_len].as_chunks_mut::<4>().0 {
            if brand == b"heic" {
                brand.copy_from_slice(b"mif1");
            }
        }
        let image = Image::decode(&bytes).unwrap();
        assert_eq!(image.format(), Some(Format::Heif));
        assert_eq!(image.size(), (800, 1066));
        assert_eq!(image.inner.color(), image::ColorType::Rgb8);
    }

    #[test]
    fn decode_heic_applies_container_rotation_once() {
        // Synthetic red/blue 64x32 image with a 90-degree clockwise irot property.
        let bytes = include_bytes!("../../tests/fixtures/rotated.heic");
        let image = Image::decode(bytes).unwrap();
        assert_eq!(image.size(), (32, 64));
        let rgb = image.inner.as_rgb8().unwrap();
        assert!(rgb.get_pixel(16, 8).0[0] > 240);
        assert!(rgb.get_pixel(16, 56).0[2] > 240);
    }

    #[test]
    fn decode_truncated_heic_and_unsupported_heif_codec_fail() {
        let bytes = test_data_bytes!("file_types/receipt.heic");
        assert!(matches!(
            Image::decode(&bytes[..40]),
            Err(ImageError::Decode(_))
        ));
        let jpeg_heif = test_data_bytes!("file_types/receipt.jpg.heif");
        assert!(matches!(
            Image::decode(jpeg_heif),
            Err(ImageError::Decode(_))
        ));
    }

    #[test]
    fn decode_garbage_errors() {
        assert!(matches!(
            Image::decode(&[0xDE, 0xAD, 0xBE, 0xEF]),
            Err(ImageError::Decode(_))
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
    fn encode_jpeg_optimize_flag_reduces_jpeg_size() {
        let bytes = synth_bytes(256, 256, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let unoptimized = img.encode(ImageFormat::Jpeg, 85, false).unwrap();
        let optimized = img.encode(ImageFormat::Jpeg, 85, true).unwrap();
        assert!(
            optimized.len() <= unoptimized.len(),
            "optimized JPEG ({} bytes) should be <= unoptimized ({} bytes)",
            optimized.len(),
            unoptimized.len(),
        );
        assert_eq!(Image::decode(&optimized).unwrap().size(), (256, 256));
    }

    #[test]
    fn encode_jpeg_rejects_dimensions_over_jpeg_limit() {
        let image = Image::from_dynamic(DynamicImage::ImageRgb8(RgbImage::new(65_536, 1)));
        assert!(matches!(
            image.encode(ImageFormat::Jpeg, 85, true),
            Err(ImageError::Encode(_))
        ));
    }

    #[test]
    fn encode_png_has_magic_bytes() {
        let bytes = synth_bytes(16, 16, ImageFormat::Jpeg);
        let img = Image::decode(&bytes).unwrap();
        let out = img.encode(ImageFormat::Png, 85, false).unwrap();
        assert_eq!(&out[0..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn encode_jpeg_roundtrips_to_decodable() {
        let bytes = synth_bytes(24, 12, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let jpeg = img.encode(ImageFormat::Jpeg, 90, false).unwrap();
        let reloaded = Image::decode(&jpeg).unwrap();
        assert_eq!(reloaded.size(), (24, 12));
        assert_eq!(reloaded.format(), Some(ImageFormat::Jpeg.into()));
    }

    #[test]
    fn encode_pdf_has_magic_bytes_and_embeds_jpeg() {
        crate::pdf::pdfium();
        let bytes = synth_bytes(24, 12, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let pdf = img.encode(OutputFormat::Pdf, 85, false).unwrap();
        assert!(pdf.starts_with(b"%PDF-"), "PDF header missing");
        let document = crate::pdf::document::Document::decode(&pdf).unwrap();
        assert_eq!(document.page_count().unwrap(), 1);
        assert_eq!(document.page(0).unwrap().size(), (24.0, 12.0));
    }

    #[test]
    fn encode_pdf_media_box_matches_image_size() {
        crate::pdf::pdfium();
        let bytes = synth_bytes(40, 25, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let pdf = img.encode(OutputFormat::Pdf, 85, false).unwrap();
        let document = crate::pdf::document::Document::decode(&pdf).unwrap();
        assert_eq!(document.page(0).unwrap().size(), (40.0, 25.0));
    }

    #[test]
    fn encode_pdf_via_string_format() {
        crate::pdf::pdfium();
        let bytes = synth_bytes(16, 16, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let format = parse_output_format("pdf").unwrap();
        assert_eq!(format, OutputFormat::Pdf);
        let pdf = img.encode(format, 85, false).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
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
    fn downscale_never_upscales_when_bounds_larger() {
        let bytes = synth_bytes(50, 50, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = img.downscale_to_fit(Some(500), Some(500));
        assert_eq!(out.size(), (50, 50));
    }

    #[test]
    fn downscale_none_bounds_keeps_size() {
        let bytes = synth_bytes(50, 40, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = img.downscale_to_fit(None, None);
        assert_eq!(out.size(), (50, 40));
    }

    #[test]
    fn downscale_preserves_aspect_ratio() {
        let bytes = synth_bytes(200, 100, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = img.downscale_to_fit(Some(100), Some(100));
        assert_eq!(out.size(), (100, 50));
    }

    #[test]
    fn downscale_only_width_bound() {
        let bytes = synth_bytes(200, 100, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        // scale = 50/200 = 0.25
        let out = img.downscale_to_fit(Some(50), None);
        assert_eq!(out.size(), (50, 25));
    }

    #[test]
    fn downscale_floors_dimensions_to_at_least_one() {
        let bytes = synth_bytes(100, 4, ImageFormat::Png);
        let img = Image::decode(&bytes).unwrap();
        let out = img.downscale_to_fit(Some(2), None);
        let (_, h) = out.size();
        assert!(h >= 1);
    }

    #[test]
    fn compress_to_jpeg_returns_jpeg_and_dimensions() {
        let bytes = synth_bytes(200, 100, ImageFormat::Png);
        let (jpeg, w, h) = compress_to_jpeg(&bytes, 85, false, Some(100), Some(100)).unwrap();
        assert_eq!(&jpeg[0..3], &[0xFF, 0xD8, 0xFF]);
        assert_eq!((w, h), (100, 50));
    }
    #[test]
    fn compress_to_jpeg_errors_when_jpeg_is_not_smaller() {
        let bytes = synth_bytes(30, 30, ImageFormat::Png); // tiny flat PNG
        assert!(matches!(
            compress_to_jpeg(&bytes, 85, false, None, None),
            Err(ImageError::SizeIncreased { .. })
        ));
    }

    #[test]
    fn compress_to_jpeg_with_bounds_returns_jpeg_even_if_larger() {
        // a requested downscale is always honored, regardless of byte count
        let bytes = synth_bytes(200, 100, ImageFormat::Png);
        let (_, w, h) = compress_to_jpeg(&bytes, 85, false, Some(100), Some(100)).unwrap();
        assert_eq!((w, h), (100, 50));
    }
    #[test]
    fn compress_to_jpeg_garbage_errors() {
        assert!(compress_to_jpeg(&[0x00, 0x01], 85, false, None, None).is_err());
    }

    #[test]
    fn compress_fixture_jpeg() {
        let bytes = test_data_bytes!("file_types/receipt.jpg");
        let (jpeg, w, h) = compress_to_jpeg(bytes, 80, false, Some(512), Some(512)).unwrap();
        assert_eq!(&jpeg[0..3], &[0xFF, 0xD8, 0xFF]);
        assert!(w <= 512 && h <= 512);
    }
}
