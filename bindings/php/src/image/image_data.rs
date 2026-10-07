use super::error::{map_image_err, unknown_format_err};
use bernard_ledit::image::{
    Filter, Image as CoreImage, format_name, guess_format as core_guess, parse_output_format,
};
use ext_php_rs::types::Zval;
use ext_php_rs::zend::ce;
use ext_php_rs::{binary::Binary, binary_slice::BinarySlice, prelude::*};
use std::str::FromStr;

#[php_class]
#[php(name = "BernardLedit\\Image\\Image")]
pub struct Image {
    inner: CoreImage,
}

#[php_impl]
impl Image {
    /// `[width, height]`
    #[must_use]
    pub fn size(&self) -> Vec<u32> {
        let (w, h) = self.inner.size();
        vec![w, h]
    }

    /// Width.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.inner.size().0
    }

    /// Height.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.inner.size().1
    }

    /// Upper-case format name (`"JPEG"`, `"PNG"`, etc.) or `null`.
    /// # Errors
    /// Returns an error if the format name cannot be converted to a string.
    pub fn format(&self) -> Option<&'static str> {
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
    pub fn resize(&self, width: u32, height: u32, filter: Option<&str>) -> PhpResult<Self> {
        let filter =
            Filter::from_str(filter.unwrap_or("lanczos")).map_err(|e| map_image_err(&e))?;
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

impl Image {
    /// Creates a PHP image from a core lib image.
    pub(crate) const fn from_core(inner: CoreImage) -> Self {
        Self { inner }
    }
}

/// `BernardLedit\Image\decode(string $data): Image`
/// # Errors
/// Returns an error if the decoding operation fails.
#[php_function]
#[php(name = "BernardLedit\\Image\\decode")]
#[allow(clippy::needless_pass_by_value)]
pub fn decode(data: BinarySlice<u8>) -> PhpResult<Image> {
    CoreImage::decode(&data)
        .map(Image::from_core)
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

/// Mirrors the python implementation:
///
/// `BernardLedit\Image\compress(
///   string $data,
///   int $quality = 85,
///   ?int $maxWidth = null,
///   ?int $maxHeight = null
/// ): array{0:string,1:int,2:int}`
/// # Errors
/// Returns an error if the compression operation fails, or if the resulting dimensions
/// don't fit in a 32-bit integer.
#[php_function]
#[php(name = "BernardLedit\\Image\\compress")]
#[php(defaults(quality = 85, max_width = None, max_height = None))]
#[allow(clippy::needless_pass_by_value)]
pub fn compress(
    data: BinarySlice<u8>,
    quality: u8,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> PhpResult<Vec<Zval>> {
    let (bytes, w, h) = bernard_ledit::image::compress(&data, quality, max_width, max_height)
        .map_err(|e| map_image_err(&e))?;
    let mut out = Vec::with_capacity(3);
    let mut z = Zval::new();
    z.set_binary(bytes);
    out.push(z);
    let w_i32 =
        i32::try_from(w).map_err(|e| PhpException::new(e.to_string(), 0, ce::value_error()))?;
    out.push(Zval::from(w_i32));
    let h_i32 =
        i32::try_from(h).map_err(|e| PhpException::new(e.to_string(), 0, ce::value_error()))?;
    out.push(Zval::from(h_i32));
    Ok(out)
}

/// Registers the functions in this module.
/// Note: this is present in this file because #[`php_function`] macro generates a non-public
/// internal function which the module can't otherwise use.
pub fn register_functions(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(decode))
        .function(wrap_function!(guess_format))
        .function(wrap_function!(compress))
}
