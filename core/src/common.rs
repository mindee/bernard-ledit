//! Types shared across core modules.

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

impl From<ImageFormat> for OutputFormat {
    fn from(format: ImageFormat) -> Self {
        Self::Image(format)
    }
}
