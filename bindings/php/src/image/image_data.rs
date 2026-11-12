use super::error::{format_image_extract_err, map_image_err, unknown_format_err};
use crate::args::{self, value_error};
use bernard_ledit::image::{
    FilterType, Image, format_name, guess_format as core_guess, parse_output_format,
};
use ext_php_rs::types::Zval;
use ext_php_rs::{binary::Binary, binary_slice::BinarySlice, prelude::*};

#[php_class]
#[php(name = "BernardLedit\\Image\\Image")]
pub struct PhpImage {
    inner: Image,
}

/// Core image conversion.
impl From<Image> for PhpImage {
    fn from(inner: Image) -> Self {
        Self { inner }
    }
}

#[php_impl]
impl PhpImage {
    /// Decodes an image from its encoded bytes: `new Image(string $data)`.
    /// # Errors
    /// - `\BernardLedit\Image\ImageException` if `$data` isn't an identifiable image, or can't be
    ///   decoded.
    #[allow(clippy::needless_pass_by_value)]
    pub fn __construct(data: BinarySlice<u8>) -> PhpResult<Self> {
        let inner = Image::decode(&data).map_err(|e| map_image_err(&e))?;
        Ok(Self { inner })
    }

    /// Dimensions in pixels.
    /// # Returns
    /// `array{0: int, 1: int}` `[$width, $height]`.
    #[must_use]
    pub fn get_size(&self) -> Vec<u32> {
        let (w, h) = self.inner.size();
        vec![w, h]
    }

    /// Width.
    #[must_use]
    pub fn get_width(&self) -> u32 {
        self.inner.size().0
    }

    /// Height.
    #[must_use]
    pub fn get_height(&self) -> u32 {
        self.inner.size().1
    }

    /// Upper-case format name (`"JPEG"`, `"PNG"`, etc.) or `null`.
    #[must_use]
    pub fn get_format(&self) -> Option<&'static str> {
        self.inner.format().and_then(format_name)
    }

    /// Crops the image to the box from (`$left`, `$top`) to (`$right`, `$bottom`), in pixels; the
    /// right and bottom edges are excluded.
    /// # Parameters
    /// - `left`: `int<0, 4294967295>`
    /// - `top`: `int<0, 4294967295>`
    /// - `right`: `int<0, 4294967295>`
    /// - `bottom`: `int<0, 4294967295>`
    /// # Errors
    /// - `\ValueError` if a coordinate is out of range, or the box is empty or extends past the
    ///   image.
    pub fn crop(&self, left: i64, top: i64, right: i64, bottom: i64) -> PhpResult<Self> {
        let left = args::int("left", left, 0, u32::MAX)?;
        let top = args::int("top", top, 0, u32::MAX)?;
        let right = args::int("right", right, 0, u32::MAX)?;
        let bottom = args::int("bottom", bottom, 0, u32::MAX)?;
        self.inner
            .crop(left, top, right, bottom)
            .map(|inner| Self { inner })
            .map_err(|e| map_image_err(&e))
    }

    /// Crops the bounding box of a polygon from this image.
    /// # Parameters
    /// - `polygon`: `list<array{0: float, 1: float}>` Normalized points relative to the image size.
    /// # Errors
    /// - `\ValueError` if the polygon is empty, malformed, or its bounding box is empty.
    /// - `\BernardLedit\Image\ImageException` if cropping fails.
    #[allow(clippy::cast_possible_truncation)]
    pub fn crop_polygon(&self, polygon: Vec<Vec<f64>>) -> PhpResult<Self> {
        if polygon.iter().any(|point| point.len() != 2) {
            return Err(value_error(
                "Each polygon point must contain exactly two coordinates".into(),
            ));
        }
        let polygon: Vec<(f32, f32)> = polygon
            .into_iter()
            .map(|point| (point[0] as f32, point[1] as f32))
            .collect();
        self.inner
            .crop_polygon(&polygon)
            .map(PhpImage::from_core)
            .map_err(|e| format_image_extract_err(&e))
    }

    /// Resizes the image to `$width` by `$height` pixels, with a Lanczos filter.
    /// # Parameters
    /// - `width`: `int<1, 4294967295>`
    /// - `height`: `int<1, 4294967295>`
    /// # Errors
    /// - `\ValueError` if `$width` or `$height` is out of range.
    pub fn resize(&self, width: i64, height: i64) -> PhpResult<Self> {
        let width = args::int("width", width, 1, u32::MAX)?;
        let height = args::int("height", height, 1, u32::MAX)?;
        self.inner
            .resize(width, height, FilterType::Lanczos3)
            .map(|inner| Self { inner })
            .map_err(|e| map_image_err(&e))
    }

    /// Returns the encoded bytes as a PHP string.
    /// # Parameters
    /// - `format`: `string` An output format: `"JPEG"`, `"PNG"`, `"PDF"`, etc.
    /// - `quality`: `int<1, 100>` Only used by JPEG and PDF.
    /// - `optimize`: `bool` Optimize JPEG Huffman tables.
    /// # Errors
    /// - `\ValueError` if `$format` isn't a supported output format, or `$quality` is out of
    ///   range.
    /// - `\BernardLedit\Image\ImageException` if encoding fails.
    #[php(defaults(quality = 85, optimize = true))]
    pub fn encode(&self, format: &str, quality: i64, optimize: bool) -> PhpResult<Binary<u8>> {
        let quality = args::quality(quality, 1)?;
        let fmt = parse_output_format(format).map_err(|e| map_image_err(&e))?;
        let bytes = self
            .inner
            .encode(fmt, quality, optimize)
            .map_err(|e| map_image_err(&e))?;
        Ok(Binary::new(bytes))
    }
}

impl PhpImage {
    /// Creates a PHP image from a core lib image.
    pub(crate) const fn from_core(inner: Image) -> Self {
        Self { inner }
    }
}

/// `BernardLedit\Image\decode(string $data): Image`
/// # Errors
/// - `\BernardLedit\Image\ImageException` if `$data` isn't an identifiable image, or can't be
///   decoded.
#[php_function]
#[php(name = "BernardLedit\\Image\\decode")]
#[allow(clippy::needless_pass_by_value)]
pub fn decode(data: BinarySlice<u8>) -> PhpResult<PhpImage> {
    Image::decode(&data)
        .map(PhpImage::from_core)
        .map_err(|e| map_image_err(&e))
}

/// `BernardLedit\Image\guessFormat(string $data): string`
/// # Errors
/// - `\BernardLedit\Image\ImageException` if `$data` isn't an identifiable image.
#[php_function]
#[php(name = "BernardLedit\\Image\\guessFormat")]
#[allow(clippy::needless_pass_by_value)]
pub fn guess_format(data: BinarySlice<u8>) -> PhpResult<&'static str> {
    let fmt = core_guess(&data).map_err(|e| map_image_err(&e))?;
    format_name(fmt).ok_or_else(unknown_format_err)
}

/// Compresses an image as a JPEG, scaled down to fit `$max_width` and `$max_height`. Returns the
/// input unchanged if that doesn't make it smaller.
/// # Parameters
/// - `quality`: `int<5, 100>`
/// - `max_width`: `int<1, 4294967295>|null`
/// - `max_height`: `int<1, 4294967295>|null`
/// # Returns
/// `array{0: string, 1: int, 2: int}` `[$bytes, $width, $height]`.
/// # Errors
/// - `\ValueError` if `$quality`, `$max_width` or `$max_height` is out of range.
/// - `\BernardLedit\Image\ImageException` if `$data` isn't an identifiable image, or can't be
///   decoded or encoded.
#[php_function]
#[php(name = "BernardLedit\\Image\\compress")]
#[php(defaults(quality = 85, max_width = None, max_height = None, optimize = None))]
#[allow(clippy::needless_pass_by_value)]
pub fn compress(
    data: BinarySlice<u8>,
    quality: i64,
    max_width: Option<i64>,
    max_height: Option<i64>,
    optimize: Option<bool>,
) -> PhpResult<Vec<Zval>> {
    let quality = args::quality(quality, 5)?;
    let max_width = max_width
        .map(|w| args::int("max_width", w, 1, u32::MAX))
        .transpose()?;
    let max_height = max_height
        .map(|h| args::int("max_height", h, 1, u32::MAX))
        .transpose()?;
    let image = Image::decode(&data).map_err(|e| map_image_err(&e))?;
    let (bytes, w, h) = image
        .compress(quality, max_width, max_height, optimize.unwrap_or(true))
        .map_err(|e| map_image_err(&e))?
        .unwrap_or_else(|| {
            let (w, h) = image.size();
            (data.to_vec(), w, h)
        });
    let mut out = Vec::with_capacity(3);
    let mut z = Zval::new();
    z.set_binary(bytes);
    out.push(z);
    for dim in [w, h] {
        let mut z = Zval::new();
        z.set_long(dim);
        out.push(z);
    }
    Ok(out)
}

/// Registers the internal functions in this module.
pub fn register_functions(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(decode))
        .function(wrap_function!(guess_format))
        .function(wrap_function!(compress))
}
