//! Types shared across core modules.

use crate::image::ImageError;
use ::image::ImageFormat;

/// The format of an encoded input, including formats outside the `image` crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// A raster format handled by the `image` crate.
    Image(ImageFormat),
    /// A HEIF file advertising HEVC-coded still images.
    Heic,
    /// A generic HEIF file; its image codec is checked during decoding.
    Heif,
    /// A PDF document.
    Pdf,
}

impl From<ImageFormat> for Format {
    fn from(format: ImageFormat) -> Self {
        Self::Image(format)
    }
}

/// Detects the source format from bytes, without decoding the image.
///
/// Recognizing a container does not guarantee its codec is supported.
/// # Errors
/// Returns [`ImageError::UnknownFormat`] for unrecognized or truncated headers.
pub fn guess_format(data: &[u8]) -> Result<Format, ImageError> {
    if data.starts_with(b"%PDF-") {
        return Ok(Format::Pdf);
    }
    if let Some(format) = heif_format(data) {
        return Ok(format);
    }
    ::image::guess_format(data)
        .map(Format::Image)
        .map_err(|_| ImageError::UnknownFormat)
}

fn heif_format(data: &[u8]) -> Option<Format> {
    // ISO BMFF file-type box: [size: u32 BE]["ftyp"][optional extended size: u64 BE]
    // [major brand: 4 bytes][minor version: u32 BE][compatible brands: 4 bytes each].
    // Size includes the header; 1 selects an extended size, while 0 extends to EOF.
    // Both the major and compatible brands identify formats; the minor version is skipped.
    if data.get(4..8)? != b"ftyp" {
        return None;
    }
    let size = u32::from_be_bytes(data.get(..4)?.try_into().ok()?);
    let (size, header) = match size {
        0 => (data.len(), 8),
        1 => (
            usize::try_from(u64::from_be_bytes(data.get(8..16)?.try_into().ok()?)).ok()?,
            16,
        ),
        size => (usize::try_from(size).ok()?, 8),
    };
    let payload = data.get(header..size)?;
    if payload.len() < 8 || payload.len() % 4 != 0 {
        return None;
    }
    let brands = || {
        payload[..4]
            .as_chunks::<4>()
            .0
            .iter()
            .chain(payload[8..].as_chunks::<4>().0.iter())
    };
    if brands().any(|brand| matches!(brand, b"avif" | b"avis")) {
        return Some(Format::Image(ImageFormat::Avif));
    }
    if brands().any(|brand| matches!(brand, b"heic" | b"heix" | b"heim" | b"heis")) {
        return Some(Format::Heic);
    }
    if brands().any(|brand| matches!(brand, b"mif1" | b"mif2")) {
        return Some(Format::Heif);
    }
    None
}

/// Returns the canonical uppercase source-format name.
#[must_use]
pub const fn format_name(format: Format) -> Option<&'static str> {
    match format {
        Format::Heic => Some("HEIC"),
        Format::Heif => Some("HEIF"),
        Format::Pdf => Some("PDF"),
        Format::Image(format) => match format {
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
        },
    }
}

/// A target encode format: any raster [`ImageFormat`], or PDF.
///
/// PDF is not a raster format the `image` crate knows about, so it lives here
/// alongside the raster formats to give image encoding a single entry point
/// for every output the SDK can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// A raster format handled by the `image` crate (PNG, JPEG, WEBP, ...).
    Image(ImageFormat),
    /// A single-page PDF embedding the image as a JPEG.
    Pdf,
}

/// Returns the canonical encoding name for a file extension.
///
/// The extension may include a leading dot. An empty extension defaults to
/// `JPEG`; unrecognized extensions are returned uppercased for the encoder to validate.
#[must_use]
pub fn format_from_extension(extension: &str) -> String {
    let extension = extension.trim_start_matches('.');
    if extension.is_empty() {
        return "JPEG".to_owned();
    }

    if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") {
        "JPEG".to_owned()
    } else if extension.eq_ignore_ascii_case("tif") || extension.eq_ignore_ascii_case("tiff") {
        "TIFF".to_owned()
    } else if extension.eq_ignore_ascii_case("exr") || extension.eq_ignore_ascii_case("openexr") {
        "OPENEXR".to_owned()
    } else {
        extension.to_ascii_uppercase()
    }
}

impl From<ImageFormat> for OutputFormat {
    fn from(format: ImageFormat) -> Self {
        Self::Image(format)
    }
}

/// Parse a format string into an [`OutputFormat`], accepting `"PDF"` in addition
/// to every raster name understood by [`crate::image::parse_format`].
/// # Errors
/// Returns `ImageError::UnsupportedFormat` on miss.
pub fn parse_output_format(name: &str) -> Result<OutputFormat, ImageError> {
    if name.eq_ignore_ascii_case("HEIC") || name.eq_ignore_ascii_case("HEIF") {
        return Err(ImageError::UnsupportedFormat(
            "HEIC/HEIF encoding is not supported".to_owned(),
        ));
    }
    if name.eq_ignore_ascii_case("PDF") {
        return Ok(OutputFormat::Pdf);
    }
    crate::image::parse_format(name).map(OutputFormat::Image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::ImageFormat;

    fn ftyp(major: &[u8], compatible: &[u8]) -> Vec<u8> {
        let mut bytes = u32::try_from(16 + compatible.len())
            .unwrap()
            .to_be_bytes()
            .to_vec();
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(major);
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(compatible);
        bytes
    }

    #[test]
    fn detects_heif_brands_without_confusing_avif_or_mp4() {
        assert_eq!(guess_format(&ftyp(b"heic", b"mif1")).unwrap(), Format::Heic);
        assert_eq!(
            guess_format(&ftyp(b"mif1", b"miafheix")).unwrap(),
            Format::Heic
        );
        assert_eq!(guess_format(&ftyp(b"mif1", b"")).unwrap(), Format::Heif);
        assert_eq!(
            guess_format(&ftyp(b"mif1", b"avif")).unwrap(),
            Format::Image(ImageFormat::Avif)
        );
        assert!(guess_format(&ftyp(b"mp42", b"isom")).is_err());
        assert_eq!(guess_format(b"%PDF-1.7\n").unwrap(), Format::Pdf);
    }

    #[test]
    fn heif_detection_checks_box_lengths_and_extended_sizes() {
        let mut bytes = ftyp(b"heic", b"mif1");
        assert!(guess_format(&bytes[..bytes.len() - 1]).is_err());
        bytes[..4].copy_from_slice(&4_u32.to_be_bytes());
        assert!(guess_format(&bytes).is_err());

        let mut extended = Vec::from(&b"\0\0\0\x01ftyp"[..]);
        extended.extend_from_slice(&28_u64.to_be_bytes());
        extended.extend_from_slice(b"heic\0\0\0\0mif1");
        assert_eq!(guess_format(&extended).unwrap(), Format::Heic);
    }

    #[test]
    fn rejects_heif_encoding_explicitly() {
        for name in ["heic", "HEIC", "Heif"] {
            let error = parse_output_format(name).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("HEIC/HEIF encoding is not supported")
            );
        }
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
    fn format_from_extension_normalizes_known_extensions() {
        assert_eq!(format_from_extension("jpg"), "JPEG");
        assert_eq!(format_from_extension(".JPEG"), "JPEG");
        assert_eq!(format_from_extension("tif"), "TIFF");
        assert_eq!(format_from_extension("ExR"), "OPENEXR");
        assert_eq!(format_from_extension("pdf"), "PDF");
    }

    #[test]
    fn format_from_extension_defaults_and_preserves_unknown_extensions() {
        assert_eq!(format_from_extension(""), "JPEG");
        assert_eq!(format_from_extension("custom"), "CUSTOM");
    }
}
