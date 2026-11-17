use crate::image::Image;
use crate::pdf::PdfError;
use crate::pdf::text_char::TextChar;
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

    /// Get the clockwise page rotation in degrees.
    /// # Errors
    /// Returns [`PdfError`] if the rotation cannot be read.
    pub fn rotation(&self) -> Result<f32, PdfError> {
        Ok(self.inner.rotation()?.as_degrees())
    }

    /// Get the unrotated page bounding box as (left, bottom, right, top).
    /// # Errors
    /// Returns [`PdfError`] if the bounding box cannot be read.
    pub fn bounding_box(&self) -> Result<(f32, f32, f32, f32), PdfError> {
        let bounds = self.inner.boundaries().bounding()?.bounds;
        Ok((
            bounds.left().value,
            bounds.bottom().value,
            bounds.right().value,
            bounds.top().value,
        ))
    }

    /// Extract tight character bounds in PDF coordinates.
    /// # Errors
    /// Returns [`PdfError`] if character extraction fails.
    pub fn tight_chars(&self) -> Result<Vec<TextChar>, PdfError> {
        self.inner
            .text()?
            .chars()
            .iter()
            .map(|character| TextChar::from_pdf_char(&character, true))
            .collect()
    }

    /// Check for visible native text not covered by a page-sized image.
    /// # Errors
    /// Returns [`PdfError`] if text or object metadata cannot be read.
    pub fn is_native(&self, image_area_threshold: f32) -> Result<bool, PdfError> {
        // TODO: look at the previous implementation of this ?
        let text = self.inner.text()?;
        let mut visible = false;
        for character in text.chars().iter() {
            if character.unicode_char().is_some_and(|c| !c.is_control())
                && character.render_mode()? != PdfPageTextRenderMode::Invisible
                && character.stroke_color()?.alpha() != 0
            {
                visible = true;
                break;
            }
        }
        if !visible {
            return Ok(false);
        }
        let (width, height) = self.size();
        let objects = self.inner.objects();
        let mut covering_image = false;
        for index in (0..objects.len()).rev() {
            let object = objects.get(index)?;
            match object.object_type() {
                PdfPageObjectType::Image => {
                    let bounds = object.bounds()?.to_rect();
                    if bounds.width().value * bounds.height().value
                        > width * height * image_area_threshold
                    {
                        covering_image = true;
                    }
                }
                PdfPageObjectType::Text if covering_image => return Ok(false),
                _ => {}
            }
        }
        Ok(true)
    }

    /// Check if the page contains neither test nor objects.
    /// # Errors
    /// Returns a `PdfError` if the text can't be extracted.
    pub(crate) fn is_empty(&self) -> Result<bool, PdfError> {
        Ok(self.inner.objects().len() == 0 && self.inner.text()?.is_empty())
    }

    /// Extract all text from the page.
    /// # Errors
    /// Returns a `PdfError` if text extraction fails.
    pub fn text(&self) -> Result<String, PdfError> {
        Ok(self.inner.text()?.all())
    }

    /// Extract all characters from the page.
    /// # Errors
    /// Returns a `PdfError` if text extraction fails.
    pub(crate) fn chars(&self) -> Result<Vec<TextChar>, PdfError> {
        let chars: Vec<TextChar> = self
            .inner
            .text()?
            .chars()
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;
        Ok(chars)
    }

    /// Renders the page as an image at 72 DPI per unit of `scale`.
    /// # Errors
    /// Returns [`PdfError`] if the image cannot be rendered.
    pub fn render(&self, scale: f32) -> Result<Image, PdfError> {
        let _lock = crate::pdf::NATIVE_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cfg = PdfRenderConfig::new().scale_page_by_factor(scale);
        let bitmap = self.inner.render_with_config(&cfg)?;
        Ok(Image::from_dynamic(bitmap.as_image()?))
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf::{Document, pdfium};
    use image::{ImageBuffer, ImageFormat, Rgb};
    use std::io::Cursor;

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
        let jpeg = make_real_jpeg(200, 150);
        let doc = Document::from_jpeg(&jpeg, 200.0, 150.0).unwrap();
        assert_eq!(doc.page(0).unwrap().size(), (200.0, 150.0));
    }

    #[test]
    fn test_size_non_square_no_width_height_swap() {
        pdfium();
        let jpeg = make_real_jpeg(100, 300);
        let doc = Document::from_jpeg(&jpeg, 100.0, 300.0).unwrap();
        assert_eq!(doc.page(0).unwrap().size(), (100.0, 300.0));
    }

    #[test]
    fn test_is_empty_false_for_jpeg_page() {
        pdfium();
        let jpeg = make_real_jpeg(10, 10);
        let doc = Document::from_jpeg(&jpeg, 10.0, 10.0).unwrap();
        assert!(!doc.page(0).unwrap().is_empty().unwrap());
    }

    #[test]
    fn test_render_image_size_matches_page_at_scale_one() {
        pdfium();
        let jpeg = make_real_jpeg(100, 50);
        let doc = Document::from_jpeg(&jpeg, 100.0, 50.0).unwrap();
        assert_eq!(doc.page(0).unwrap().render(1.0).unwrap().size(), (100, 50));
    }

    #[test]
    fn test_render_scale_two_doubles_dimensions() {
        pdfium();
        let jpeg = make_real_jpeg(100, 50);
        let doc = Document::from_jpeg(&jpeg, 100.0, 50.0).unwrap();
        let page = doc.page(0).unwrap();
        let (w1, h1) = page.render(1.0).unwrap().size();
        let (w2, h2) = page.render(2.0).unwrap().size();
        assert!(w2 - 2 * w1 <= 1);
        assert!(h2 - 2 * h1 <= 1);
    }

    #[test]
    fn test_chars_returns_text_chars() {
        pdfium();
        let doc = Document::decode(test_data_bytes!("file_types/pdf/multipage.pdf")).unwrap();
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
    fn compress_to_jpeg_scanned_page_keeps_page_size() {
        pdfium();
        let jpeg = test_data_bytes!("file_types/receipt.jpg");
        let doc = Document::from_jpeg(jpeg, 200.0, 300.0).unwrap();
        let out = doc
            .compress_to_jpeg(80, true, 150.0)
            .unwrap()
            .expect("scanned page should be recompressed");
        let (w, h) = out.page(0).unwrap().size();
        assert!((w - 200.0).abs() < 0.5 && (h - 300.0).abs() < 0.5);
    }
}
