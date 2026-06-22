use crate::pdf::{PdfError, bitmap::Bitmap};
use pdfium_render::prelude::*;

/// A struct that represents a PDF page.
pub struct Page<'a> {
    /// The underlying `PDFium` page.
    inner: PdfPage<'a>,
}

impl<'a> Page<'a> {
    pub(crate) const fn new(inner: PdfPage<'a>) -> Self {
        Self { inner }
    }

    /// Get the size of the page in PDF points.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn size(&self) -> (f32, f32) {
        (self.inner.width().value, self.inner.height().value)
    }

    /// Check if the page is empty.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn is_empty(&self) -> bool {
        self.inner.objects().len() == 0
    }

    /// Extract all text from the page.
    /// # Errors
    /// Returns a `PdfError` if text extraction fails.
    pub fn text(&self) -> Result<String, PdfError> {
        Ok(self.inner.text()?.all())
    }

    /// Render the page to a bitmap.
    /// # Errors
    /// Returns a `PdfError` if the page could not be rendered.
    pub fn render(&self, scale: f32) -> Result<Bitmap, PdfError> {
        let cfg = PdfRenderConfig::new().scale_page_by_factor(scale);
        let pb = self.inner.render_with_config(&cfg)?;
        Ok(Bitmap::from_pdfium(&pb))
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf::{Document, pdfium};
    use image::{ImageBuffer, ImageFormat, Rgb};
    use std::io::Cursor;

    /// Minimal synthetic JPEG: valid SOF0 header, no pixel data.
    /// Sufficient for `from_jpeg` (PDF embedding) but NOT for `render()`.
    fn make_minimal_jpeg(width: u16, height: u16) -> Vec<u8> {
        let mut data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xC0, // SOF0
            0x00, 0x08, // segment length
            0x08, // precision
        ];
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&width.to_be_bytes());
        data.push(3); // RGB
        data
    }

    /// Fully valid JPEG, required for tests that call `render()`.
    fn make_real_jpeg(width: u32, height: u32) -> Vec<u8> {
        let img: ImageBuffer<Rgb<u8>, _> =
            ImageBuffer::from_fn(width, height, |x, _| Rgb([x.try_into().unwrap(), 128, 200]));
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
        buf.into_inner()
    }

    #[test]
    fn size_matches_requested_dimensions() {
        pdfium();
        let jpeg = make_minimal_jpeg(200, 150);
        let doc = Document::from_jpeg(&jpeg, 200.0, 150.0).unwrap();
        assert_eq!(doc.page(0).unwrap().size(), (200.0, 150.0));
    }

    #[test]
    fn size_non_square_no_width_height_swap() {
        pdfium();
        let jpeg = make_minimal_jpeg(100, 300);
        let doc = Document::from_jpeg(&jpeg, 100.0, 300.0).unwrap();
        assert_eq!(doc.page(0).unwrap().size(), (100.0, 300.0));
    }

    #[test]
    fn is_empty_false_for_jpeg_page() {
        pdfium();
        let jpeg = make_minimal_jpeg(10, 10);
        let doc = Document::from_jpeg(&jpeg, 10.0, 10.0).unwrap();
        assert!(!doc.page(0).unwrap().is_empty());
    }

    #[test]
    fn render_bitmap_size_matches_page_at_scale_one() {
        pdfium();
        let jpeg = make_real_jpeg(100, 50);
        let doc = Document::from_jpeg(&jpeg, 100.0, 50.0).unwrap();
        let bmp = doc.page(0).unwrap().render(1.0).unwrap();
        assert_eq!(bmp.width, 100);
        assert_eq!(bmp.height, 50);
    }

    #[test]
    fn render_scale_two_doubles_dimensions() {
        pdfium();
        let jpeg = make_real_jpeg(100, 50);
        let doc = Document::from_jpeg(&jpeg, 100.0, 50.0).unwrap();
        let page = doc.page(0).unwrap();
        let bmp1 = page.render(1.0).unwrap();
        let bmp2 = page.render(2.0).unwrap();
        assert!(bmp2.width - 2 * bmp1.width <= 1);
        assert!(bmp2.height - 2 * bmp1.height <= 1);
    }

    #[test]
    fn render_rgba_buffer_size_is_consistent() {
        pdfium();
        let jpeg = make_real_jpeg(20, 30);
        let doc = Document::from_jpeg(&jpeg, 20.0, 30.0).unwrap();
        let bmp = doc.page(0).unwrap().render(1.0).unwrap();
        assert_eq!(bmp.rgba.len(), (bmp.width * bmp.height * 4) as usize);
    }
}
