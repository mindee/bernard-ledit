use crate::image::error::map_image_err;
use bernard_ledit::image::ImageError;
use bernard_ledit::pdf::Bitmap;
use ext_php_rs::binary::Binary;
use ext_php_rs::exception::PhpResult;
use ext_php_rs::{php_class, php_impl};
use std::collections::HashMap;

#[php_class]
#[php(name = "PdfBitmap")]
pub struct PhpPdfBitmap {
    pub(crate) inner: Bitmap,
}

#[php_impl]
impl PhpPdfBitmap {
    /// Width in pixels.
    #[must_use]
    pub const fn get_width(&self) -> u32 {
        self.inner.width
    }

    /// Height in pixels.
    #[must_use]
    pub const fn get_height(&self) -> u32 {
        self.inner.height
    }

    /// Raw RGBA8 pixel buffer (row-major, 4 bytes per pixel).
    #[must_use]
    pub fn to_bytes(&self) -> Binary<u8> {
        self.inner.rgba.clone().into()
    }

    /// Encode the bitmap as a PNG byte stream.
    /// # Errors
    /// - `\BernardLedit\Image\ImageException` if PNG encoding fails.
    pub fn to_png_bytes(&self) -> PhpResult<Binary<u8>> {
        let bytes = self
            .inner
            .to_png_bytes()
            .map_err(|e| map_image_err(&ImageError::Encode(e.to_string())))?;

        Ok(bytes.into())
    }

    /// Debug info.
    #[must_use]
    pub fn __debug_info(&self) -> HashMap<&'static str, String> {
        let mut info = HashMap::new();

        info.insert("width", self.get_width().to_string());
        info.insert("height", self.get_height().to_string());

        info.insert("buffer", "(binary data hidden)".to_string());

        info
    }

    /// String rep.
    #[must_use]
    pub fn __to_string(&self) -> String {
        format!(
            "PdfBitmap(width={}, height={})",
            self.get_width(),
            self.get_height()
        )
    }
}

impl From<Bitmap> for PhpPdfBitmap {
    fn from(inner: Bitmap) -> Self {
        Self { inner }
    }
}
