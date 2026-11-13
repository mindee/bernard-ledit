//! Types shared across core modules.

use crate::image::ImageError;
use ::image::ImageFormat;

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
    if name.eq_ignore_ascii_case("PDF") {
        return Ok(OutputFormat::Pdf);
    }
    crate::image::parse_format(name).map(OutputFormat::Image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::ImageFormat;

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
