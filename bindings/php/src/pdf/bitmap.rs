use bernard_ledit::pdf::Bitmap;
use ext_php_rs::binary::Binary;
use ext_php_rs::exception::{PhpException, PhpResult};
use ext_php_rs::zend::ce;
use ext_php_rs::{php_class, php_impl};
use std::collections::HashMap;

#[php_class]
pub struct PhpPdfBitmap {
    pub(crate) inner: Bitmap,
}

#[php_impl]
impl PhpPdfBitmap {
    /// Width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.inner.width
    }

    /// Height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.inner.height
    }

    /// Raw RGBA8 pixel buffer (row-major, 4 bytes per pixel).
    #[must_use]
    pub fn to_bytes(&self) -> Binary<u8> {
        self.inner.rgba.clone().into()
    }

    /// Encode the bitmap as a PNG byte stream.
    /// # Errors
    /// Returns a PHP Exception if the conversion fails.
    pub fn to_png_bytes(&self) -> PhpResult<Binary<u8>> {
        let bytes = self
            .inner
            .to_png_bytes()
            .map_err(|e| PhpException::new(e.to_string(), 0, ce::value_error()))?;

        Ok(bytes.into())
    }

    /// Debug info.
    #[must_use]
    pub fn __debug_info(&self) -> HashMap<&'static str, String> {
        let mut info = HashMap::new();

        info.insert("width", self.width().to_string());
        info.insert("height", self.height().to_string());

        info.insert("buffer", "(binary data hidden)".to_string());

        info
    }

    /// String rep.
    #[must_use]
    pub fn __to_string(&self) -> String {
        format!(
            "PdfBitmap(width={}, height={})",
            self.width(),
            self.height()
        )
    }
}

impl From<Bitmap> for PhpPdfBitmap {
    fn from(inner: Bitmap) -> Self {
        Self { inner }
    }
}
