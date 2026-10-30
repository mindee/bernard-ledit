//! Images: decoding, encoding, compression and polygon extraction.

use crate::{BernardLeditError, args, pdf};
use bernard_ledit::image::{Image as CoreImage, format_name, guess_format as core_guess};
use std::sync::Arc;

type Result<T> = std::result::Result<T, BernardLeditError>;

/// A decoded image.
#[derive(uniffi::Object)]
pub struct Image {
    inner: CoreImage,
}

#[uniffi::export]
impl Image {
    /// Decodes an image from its encoded bytes.
    ///
    /// # Errors
    /// `Image` if `data` isn't an identifiable image, or can't be decoded.
    #[uniffi::constructor]
    pub fn new(data: &[u8]) -> Result<Self> {
        Ok(Self {
            inner: CoreImage::decode(data)?,
        })
    }

    /// Width in pixels.
    #[must_use]
    pub fn width(&self) -> i64 {
        self.inner.size().0.into()
    }

    /// Height in pixels.
    #[must_use]
    pub fn height(&self) -> i64 {
        self.inner.size().1.into()
    }

    /// Upper-case format name (`JPEG`, `PNG`, ...), if known.
    #[must_use]
    pub fn format(&self) -> Option<String> {
        self.inner.format().and_then(format_name).map(String::from)
    }

    /// Encodes the image as `format` (`JPEG`, `PNG`, `PDF`, ...); `quality` (1-100) only applies to
    /// JPEG and PDF.
    ///
    /// # Errors
    /// `InvalidArgument` if `format` isn't supported or `quality` is out of range; `Image` if
    /// encoding fails.
    pub fn encode(&self, format: &str, quality: i32) -> Result<Vec<u8>> {
        let quality = args::quality(quality, 1)?;
        let format = bernard_ledit::image::parse_output_format(format)?;
        Ok(self.inner.encode(format, quality, true)?)
    }
}

/// The result of [`compress_image`].
#[derive(uniffi::Record)]
pub struct CompressedImage {
    /// The JPEG bytes, or the input if compressing didn't make it smaller.
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: i64,
    /// Height in pixels.
    pub height: i64,
}

/// A point, relative to the page's width and height (0 to 1).
#[derive(uniffi::Record)]
pub struct Point {
    /// Horizontal position.
    pub x: f64,
    /// Vertical position.
    pub y: f64,
}

/// Returns the upper-case format name of an encoded image (`JPEG`, `PNG`, ...).
///
/// # Errors
/// `Image` if `data` isn't an identifiable image.
#[uniffi::export]
pub fn guess_format(data: &[u8]) -> Result<String> {
    let format = core_guess(data)?;
    format_name(format)
        .map(String::from)
        .ok_or_else(|| BernardLeditError::Image("unknown image format".into()))
}

/// Compresses an image as a JPEG of `quality` (5-100), scaled down to fit `max_width` and
/// `max_height` if given.
///
/// # Errors
/// `InvalidArgument` if an argument is out of range; `Image` if `data` isn't an identifiable image,
/// or can't be decoded or encoded.
#[uniffi::export]
pub fn compress_image(
    data: &[u8],
    quality: i32,
    max_width: Option<i64>,
    max_height: Option<i64>,
) -> Result<CompressedImage> {
    let quality = args::quality(quality, 5)?;
    let max_width = max_width
        .map(|w| args::int("max_width", w, 1, u32::MAX))
        .transpose()?;
    let max_height = max_height
        .map(|h| args::int("max_height", h, 1, u32::MAX))
        .transpose()?;
    let image = CoreImage::decode(data)?;
    let (bytes, width, height) = image
        .compress(quality, max_width, max_height, true)?
        .unwrap_or_else(|| {
            let (w, h) = image.size();
            (data.to_vec(), w, h)
        });
    Ok(CompressedImage {
        data: bytes,
        width: width.into(),
        height: height.into(),
    })
}

/// Crops one image per polygon from page `page_index` of a PDF rendered at `dpi` (1-1200), or of an
/// image (page 0).
///
/// # Errors
/// `InvalidArgument` if an argument is out of range, a polygon is empty or falls outside the page,
/// or there's no such page; `Image` if `data` is neither a PDF nor a decodable image; `Pdfium` if
/// `PDFium` can't read or render the PDF.
#[uniffi::export]
pub fn extract_from_polygons(
    data: &[u8],
    page_index: i32,
    polygons: Vec<Vec<Point>>,
    dpi: f64,
) -> Result<Vec<Arc<Image>>> {
    let page_index = args::page_index("page_index", page_index)?;
    let dpi = args::dpi(dpi)?;
    #[allow(
        clippy::cast_possible_truncation,
        reason = "core takes f32 coordinates"
    )]
    let polygons: Vec<Vec<(f32, f32)>> = polygons
        .into_iter()
        .map(|polygon| polygon.iter().map(|p| (p.x as f32, p.y as f32)).collect())
        .collect();
    if data.starts_with(b"%PDF-") {
        pdf::ensure_pdfium()?;
    }
    Ok(
        bernard_ledit::image::extract::extract_from_polygons(data, page_index, &polygons, dpi)?
            .into_iter()
            .map(|inner| Arc::new(Image { inner }))
            .collect(),
    )
}
