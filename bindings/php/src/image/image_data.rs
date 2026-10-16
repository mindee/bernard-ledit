use super::error::{format_image_extract_err, map_image_err, unknown_format_err};
use bernard_ledit::image::{
    Filter, Image, format_name, guess_format as core_guess, parse_output_format,
};
use ext_php_rs::types::Zval;
use ext_php_rs::zend::ce;
use ext_php_rs::{binary::Binary, binary_slice::BinarySlice, prelude::*};
use std::str::FromStr;

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
    /// Throws if the data is not a supported image format.
    /// Throws if the image cannot be decoded.
    #[allow(clippy::needless_pass_by_value)]
    pub fn __construct(data: BinarySlice<u8>) -> PhpResult<Self> {
        let inner = Image::decode(&data).map_err(|e| map_image_err(&e))?;
        Ok(Self { inner })
    }

    /// Dimensions as `[width, height]`.
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
    /// # Errors
    /// Returns an error if the format name cannot be converted to a string.
    pub fn get_format(&self) -> Option<&'static str> {
        self.inner.format().and_then(format_name)
    }

    /// Crops a rendered page's image from a given bounding box.
    /// # Errors
    /// Returns an error if the cropping operation fails.
    pub fn crop(&self, left: u32, top: u32, right: u32, bottom: u32) -> PhpResult<Self> {
        self.inner
            .crop(left, top, right, bottom)
            .map(|inner| Self { inner })
            .map_err(|e| map_image_err(&e))
    }

    /// Resizes an image using dimensions and a filter.
    /// # Errors
    /// Returns an error if the resizing operation fails.
    pub fn resize(&self, width: u32, height: u32) -> PhpResult<Self> {
        let filter = Filter::from_str("lanczos").map_err(|e| map_image_err(&e))?;
        self.inner
            .resize(width, height, filter.0)
            .map(|inner| Self { inner })
            .map_err(|e| map_image_err(&e))
    }

    /// Returns the encoded bytes as a PHP string.
    /// # Errors
    /// Returns an error if the encoding operation fails.
    #[php(defaults(quality = 85, optimize = true))]
    pub fn encode(&self, format: &str, quality: u8, optimize: bool) -> PhpResult<Binary<u8>> {
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
/// Returns an error if the decoding operation fails.
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
/// Returns an error if the decoding operation fails.
#[php_function]
#[php(name = "BernardLedit\\Image\\guessFormat")]
#[allow(clippy::needless_pass_by_value)]
pub fn guess_format(data: BinarySlice<u8>) -> PhpResult<&'static str> {
    let fmt = core_guess(&data).map_err(|e| map_image_err(&e))?;
    format_name(fmt).ok_or_else(unknown_format_err)
}

/// Compresses an image.
/// # Errors
/// Returns an error if the compression operation fails, or if the resulting dimensions
/// don't fit in a 32-bit integer.
#[php_function]
#[php(name = "BernardLedit\\Image\\compress")]
#[php(defaults(quality = 85, max_width = None, max_height = None, optimize = None))]
#[allow(clippy::needless_pass_by_value)]
pub fn compress(
    data: BinarySlice<u8>,
    quality: u8,
    max_width: Option<u32>,
    max_height: Option<u32>,
    optimize: Option<bool>,
) -> PhpResult<Vec<Zval>> {
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
        let v = i32::try_from(dim)
            .map_err(|e| PhpException::new(e.to_string(), 0, ce::value_error()))?;
        out.push(Zval::from(v));
    }
    Ok(out)
}

/// Extracts polygons from an image.
/// # Errors
/// Return an error if the image cannot be decoded or if the polygons cannot be extracted.
#[php_function]
#[php(defaults(dpi = 72.0))]
#[php(name = "BernardLedit\\Image\\extractFromPolygons")]
#[allow(clippy::needless_pass_by_value)]
#[allow(
    clippy::cast_possible_truncation,
    reason = "Core takes in f32, PHP needs f64"
)]
pub fn extract_from_polygons(
    data: BinarySlice<u8>,
    page_id: u16,
    polygons: Vec<Vec<Vec<f64>>>,
    dpi: f64,
) -> PhpResult<Vec<PhpImage>> {
    if polygons.iter().flatten().any(|point| point.len() != 2) {
        return Err(PhpException::new(
            "Each polygon point must contain exactly two coordinates".into(),
            0,
            ce::value_error(),
        ));
    }
    let polygons_f32: Vec<Vec<(f32, f32)>> = polygons
        .into_iter()
        .map(|polygon| {
            polygon
                .into_iter()
                .map(|point| (point[0] as f32, point[1] as f32))
                .collect()
        })
        .collect();

    if data.starts_with(b"%PDF-") {
        crate::pdf::ensure_pdfium()?;
    }
    bernard_ledit::image::extract::extract_from_polygons(
        &data,
        page_id,
        polygons_f32.as_slice(),
        dpi as f32,
    )
    .map(|v| v.into_iter().map(PhpImage::from_core).collect())
    .map_err(|e| format_image_extract_err(&e))
}

/// Registers the internal functions in this module.
pub fn register_functions(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(decode))
        .function(wrap_function!(guess_format))
        .function(wrap_function!(compress))
        .function(wrap_function!(extract_from_polygons))
}
