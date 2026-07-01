use crate::pdf::text_char::TextChar;
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

    /// Check if the page contains neither test nor objects.
    /// # Errors
    /// Returns a `PdfError` if the text can't be extracted.
    pub fn is_empty(&self) -> Result<bool, PdfError> {
        Ok(self.inner.objects().len() == 0 && self.inner.text()?.is_empty())
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

    /// Extract all characters from the page.
    /// # Errors
    /// Returns a `PdfError` if text extraction fails.
    pub fn chars(&self) -> Result<Vec<TextChar>, PdfError> {
        let chars: Vec<TextChar> = self
            .inner
            .text()?
            .chars()
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;
        Ok(chars)
    }

    /// Get the page rotation in degrees.
    /// # Errors
    /// Returns a `PdfError` if the page rotation could not be determined.
    pub fn rotation(&self) -> Result<u32, PdfError> {
        let rotation = self.inner.rotation()?;
        Ok(get_rotation(rotation))
    }

    /// Rotate the page by the specified number of degrees (0, 90, 180, or 270).
    /// # Errors
    /// Returns a `PdfError` if the rotation angle is invalid or could not be set.
    pub fn rotate(&mut self, rotation: u32) -> Result<(), PdfError> {
        let pdfium_rotation = match rotation {
            0 => PdfPageRenderRotation::None,
            90 => PdfPageRenderRotation::Degrees90,
            180 => PdfPageRenderRotation::Degrees180,
            270 => PdfPageRenderRotation::Degrees270,
            _ => return Err(PdfError::Other(format!("Invalid rotation: {rotation}"))),
        };

        self.inner.set_rotation(pdfium_rotation);

        Ok(())
    }
}

/// Convert a `PdfPageRenderRotation` to a u32.
#[must_use]
pub const fn get_rotation(rotation: PdfPageRenderRotation) -> u32 {
    match rotation {
        PdfPageRenderRotation::None => 0,
        PdfPageRenderRotation::Degrees90 => 90,
        PdfPageRenderRotation::Degrees180 => 180,
        PdfPageRenderRotation::Degrees270 => 270,
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
        data.push(3);
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
    fn test_size_matches_requested_dimensions() {
        pdfium();
        let jpeg = make_minimal_jpeg(200, 150);
        let doc = Document::from_jpeg(&jpeg, 200.0, 150.0).unwrap();
        assert_eq!(doc.page(0).unwrap().size(), (200.0, 150.0));
    }

    #[test]
    fn test_size_non_square_no_width_height_swap() {
        pdfium();
        let jpeg = make_minimal_jpeg(100, 300);
        let doc = Document::from_jpeg(&jpeg, 100.0, 300.0).unwrap();
        assert_eq!(doc.page(0).unwrap().size(), (100.0, 300.0));
    }

    #[test]
    fn test_is_empty_false_for_jpeg_page() {
        pdfium();
        let jpeg = make_minimal_jpeg(10, 10);
        let doc = Document::from_jpeg(&jpeg, 10.0, 10.0).unwrap();
        assert!(!doc.page(0).unwrap().is_empty().unwrap());
    }

    #[test]
    fn test_render_bitmap_size_matches_page_at_scale_one() {
        pdfium();
        let jpeg = make_real_jpeg(100, 50);
        let doc = Document::from_jpeg(&jpeg, 100.0, 50.0).unwrap();
        let bmp = doc.page(0).unwrap().render(1.0).unwrap();
        assert_eq!(bmp.width, 100);
        assert_eq!(bmp.height, 50);
    }

    #[test]
    fn test_render_scale_two_doubles_dimensions() {
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
    fn test_render_rgba_buffer_size_is_consistent() {
        pdfium();
        let jpeg = make_real_jpeg(20, 30);
        let doc = Document::from_jpeg(&jpeg, 20.0, 30.0).unwrap();
        let bmp = doc.page(0).unwrap().render(1.0).unwrap();
        assert_eq!(bmp.rgba.len(), (bmp.width * bmp.height * 4) as usize);
    }

    #[test]
    fn test_chars_returns_text_chars() {
        pdfium();
        let doc = Document::from_bytes(test_data_bytes!("file_types/pdf/multipage.pdf").to_vec())
            .unwrap();
        let chars = doc.page(0).unwrap().chars().unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].char, '*');
        assert_eq!(chars[0].stroke_color, [0, 0, 0, 255].into());
        let font_size = doc.page(0).unwrap().chars().unwrap()[0].font_size;
        assert!((font_size - 18.0).abs() < 0.1);
        let expected_bounds: [f32; 4] = [789.884, 31.19, 811.28595, 38.192];
        let tolerance = 2.0;

        for (actual, expected) in chars[0].bounds.iter().zip(expected_bounds.iter()) {
            assert!(
                (actual - expected).abs() < tolerance,
                "Bounds mismatch: {} != {} (diff: {})",
                actual,
                expected,
                (actual - expected).abs()
            );
        }
    }

    #[test]
    fn test_rotation() {
        pdfium();
        let vertical_doc =
            Document::from_bytes(test_data_bytes!("file_types/pdf/blank_1.pdf").to_vec()).unwrap();
        let mut page_0 = vertical_doc.page(0).unwrap();
        assert_eq!(page_0.rotation().unwrap(), 0);
        page_0
            .rotate(180)
            .expect("Rotation should be one of 0, 90, 180 or 270.");
        assert_eq!(page_0.rotation().unwrap(), 180);
    }
}
