use crate::pdf::error::{PdfBuilderError, PdfError};
use crate::pdf::font_resolver::resolve_builtin_font;
use crate::pdf::jpeg::encode_jpeg_mozjpeg;
use crate::pdf::jpeg::{build_jpeg_pdf, build_jpeg_pdf_auto_size};
use crate::pdf::text_char::TextChar;
use crate::pdf::{page::Page, pdfium};

use pdfium_render::prelude::*;

/// A struct that represents a PDF document.
///
/// The `Document` struct acts as a wrapper around the `PdfDocument`
/// to provide functionality for handling and interacting with PDF files.
pub struct Document {
    inner: PdfDocument<'static>,
}

impl Document {
    /// Load a PDF from in-memory bytes.
    ///
    /// # Errors
    ///
    /// Returns a [`PdfError`] if loading the PDF fails.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, PdfError> {
        let inner = pdfium().load_pdf_from_byte_vec(bytes, None)?;
        Ok(Self { inner })
    }

    /// Create an empty PDF document.
    ///
    /// # Errors
    /// Returns a [`PdfError`] if creating the PDF fails.
    pub fn new() -> Result<Self, PdfError> {
        let inner = pdfium().create_new_pdf()?;
        Ok(Self { inner })
    }

    /// Get the number of pages in the document.
    ///
    /// # Errors
    /// Returns a [`PdfError`] if getting the page count fails.
    pub fn page_count(&self) -> Result<u16, PdfError> {
        self.inner
            .pages()
            .len()
            .try_into()
            .map_err(|_| PdfError::PageCountOutOfBounds)
    }

    /// Get a specific page from the document.
    ///
    /// # Errors
    /// Returns a [`PdfError`] if getting the page fails.
    pub fn page(&self, index: u16) -> Result<Page<'_>, PdfError> {
        let p = self.inner.pages().get(index.into())?;
        Ok(Page::new(p))
    }

    /// Import a subset of pages from `src` into `self`, appending them.
    /// 1-indexed page numbers, auto-converts 0 to 1.
    ///
    /// # Errors
    /// Returns a [`PdfError`] if importing the pages fails.
    pub fn import_pages(&mut self, src: &Self, indexes: &[u16]) -> Result<(), PdfError> {
        let src_page_count = src.page_count()?;
        for &index in indexes {
            if index >= src_page_count {
                return Err(PdfError::PageIndexOutOfBounds {
                    index,
                    page_count: src_page_count,
                });
            }
        }
        let spec = indexes
            .iter()
            .map(|i| (i + 1).to_string())
            .collect::<Vec<_>>()
            .join(",");
        let len = self.inner.pages().len();

        self.inner
            .pages_mut()
            .copy_pages_from_document(&src.inner, &spec, len)?;
        Ok(())
    }

    /// Save the document into a writable buffer.
    ///
    /// # Errors
    /// Returns a [`PdfError`] if saving the document fails.
    pub fn save<W: std::io::Write + 'static>(&self, writer: &mut W) -> Result<(), PdfError> {
        self.inner.save_to_writer(writer)?;
        Ok(())
    }

    /// Extract all text from the document, concatenating pages with newlines.
    ///
    /// # Errors
    /// Returns a [`PdfError`] if text extraction fails for any page.
    pub fn text(&self) -> Result<String, PdfError> {
        let count = self.page_count()?;
        let mut parts = Vec::with_capacity(count as usize);
        for i in 0..count {
            parts.push(self.page(i)?.text()?);
        }
        Ok(parts.join("\n"))
    }

    /// Returns true if the document contains neither text nor objects
    /// # Errors
    /// Returns a [`PdfError`] if text extraction fails for any page.
    pub fn has_no_content(&self) -> Result<bool, PdfError> {
        let count = self.page_count()?;
        for i in 0..count {
            if !self.page(i)?.is_empty()? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Extracts all text from the document as a vec of `TextChar`.
    /// # Errors
    /// Returns a [`PdfError`] if text extraction fails for any page.
    pub fn text_as_chars(&self) -> Result<Vec<Vec<TextChar>>, PdfError> {
        let count = self.page_count()?;
        let mut characters = Vec::with_capacity(count as usize);
        for i in 0..count {
            characters.push(self.page(i)?.chars()?);
        }
        Ok(characters)
    }

    /// Build a single-page PDF from a JPEG-encoded image.
    ///
    /// `width` and `height` are the PDF page dimensions in points.
    ///
    /// The JPEG bytes are embedded **as-is** using `/Filter /DCTDecode` — no pixel
    /// decode or re-encode occurs. The resulting PDF is approximately
    /// `jpeg_bytes.len() + 500 bytes` of PDF framing overhead.
    ///
    /// # Errors
    /// Returns a [`PdfBuilderError`] if the JPEG header cannot be parsed or
    /// if PDF creation fails.
    pub fn from_jpeg(jpeg_bytes: &[u8], width: f64, height: f64) -> Result<Self, PdfBuilderError> {
        let pdf_bytes = build_jpeg_pdf(jpeg_bytes, width, height)?;
        Self::from_bytes(pdf_bytes).map_err(PdfBuilderError::Pdf)
    }

    /// Build a single-page PDF from a JPEG-encoded image.
    ///
    /// `width` and `height` are determined from the JPEG header.
    /// # Errors
    /// Returns a [`PdfBuilderError`] if the JPEG header cannot be parsed or
    /// if PDF creation fails.
    pub fn from_jpeg_autosize(jpeg_bytes: &[u8]) -> Result<Self, PdfBuilderError> {
        let pdf_bytes = build_jpeg_pdf_auto_size(jpeg_bytes)?;
        Self::from_bytes(pdf_bytes).map_err(PdfBuilderError::Pdf)
    }

    /// Add text to a page.
    ///
    /// Each [`TextChar`]'s `font_name`, `font_weight`, and `font_flags` are
    /// used to pick one of the 14 PDF built-in font variants
    /// (Helvetica / Times / Courier families plus Symbol and `ZapfDingbats`),
    /// so requested fonts like "Arial Bold" are rendered using
    /// `Helvetica-Bold`, "Times New Roman Italic" using `Times-Italic`, etc.
    /// # Panics
    /// If the font name is not a valid built-in font name.
    /// # Errors
    /// Returns a `PdfError` if the font name is not a valid built-in font name.
    pub fn add_text(&mut self, page_idx: i32, chars: &[TextChar]) -> Result<(), PdfError> {
        let resolved: Vec<PdfFontBuiltin> = chars
            .iter()
            .map(|c| resolve_builtin_font(&c.font_name, c.font_weight, c.font_flags))
            .collect();

        let mut font_tokens: Vec<(PdfFontBuiltin, PdfFontToken)> = Vec::new();
        for &builtin in &resolved {
            if !font_tokens.iter().any(|(b, _)| *b == builtin) {
                let token = self.inner.fonts_mut().new_built_in(builtin);
                font_tokens.push((builtin, token));
            }
        }

        let mut page = self.inner.pages_mut().get(PdfPageIndex::from(page_idx))?;
        let page_height = page.height();
        page.set_content_regeneration_strategy(PdfPageContentRegenerationStrategy::Manual);
        {
            let objects = page.objects_mut();

            for (char_data, builtin) in chars.iter().zip(resolved.iter()) {
                let font_token = font_tokens
                    .iter()
                    .find(|(b, _)| b == builtin)
                    .map(|(_, t)| *t)
                    .expect("token was pre-resolved above");

                objects.create_text_object(
                    PdfPoints::new(char_data.bounds[1]),
                    PdfPoints::new(page_height.value - char_data.bounds[2]),
                    char_data.char,
                    font_token,
                    PdfPoints::new(char_data.font_size),
                )?;
            }
        }

        page.regenerate_content()?;

        Ok(())
    }

    /// Rasterizes a PDF page and returns the JPEG bytes.
    /// # Errors
    /// Returns a [`PdfError`] if the page cannot be rendered.
    pub fn rasterize_page(&self, page_index: u16, quality: u8) -> Result<Vec<u8>, PdfError> {
        let _lock = crate::pdf::PDFIUM_RENDER_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let page = self.inner.pages().get(page_index.into())?;
        let config = PdfRenderConfig::new();
        let bitmap = page.render_with_config(&config)?;
        let rgb_image = bitmap.as_image()?.into_rgb8();
        encode_jpeg_mozjpeg(
            rgb_image.as_raw(),
            rgb_image.width(),
            rgb_image.height(),
            quality,
            true,
        )
        .map_err(|e| PdfError::Other(format!("Failed to encode JPEG: {e}")))
    }

    /// Appends a JPEG as a new page.
    /// # Panics
    /// If the JPEG cannot be decoded.
    /// # Errors
    /// Returns a [`PdfError`] if the JPEG is invalid or the page cannot be appended.
    pub fn append_jpeg_page(&mut self, jpeg_bytes: &[u8]) -> Result<(), PdfError> {
        let doc = Self::from_jpeg_autosize(jpeg_bytes)
            .map_err(|e| PdfError::Other(format!("Invalid JPEG: {e}")))?;
        self.inner.pages_mut().append(&doc.inner)?;
        Ok(())
    }

    /// Appends a bunch of JPEGs as a new page.
    /// # Panics
    /// If one of the JPEGs cannot be decoded.
    /// # Errors
    /// Returns `PdfError` if a JPEG cannot be decoded.
    pub fn append_multiple_jpeg_pages(&mut self, jpegs: &[&[u8]]) -> Result<(), PdfError> {
        for jpeg in jpegs {
            self.append_jpeg_page(jpeg)?;
        }
        Ok(())
    }

    /// Save the document to a file.
    /// # Errors
    /// Returns `PdfError` if the file cannot be written.
    pub fn save_to_file(&self, path: &str) -> Result<(), PdfError> {
        self.inner.save_to_file(path).map_err(PdfError::from)
    }

    /// Checks whether the document has any text.
    /// # Errors
    /// Returns a [`PdfError`] if text extraction fails for any page.
    pub fn has_text(&self) -> Result<bool, PdfError> {
        let count = self.page_count()?;
        for i in 0..count {
            if !self.page(i)?.text()?.is_empty() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf::text_char::TextChar;
    use crate::pdf::{Document, pdfium};

    #[test]
    fn test_loads_from_jpeg() {
        pdfium();
        let jpeg = test_data_bytes!("file_types/receipt.jpg");
        let doc = Document::from_jpeg(jpeg, 100.0, 100.0).unwrap();
        assert_eq!(doc.page_count().unwrap(), 1);
    }

    #[test]
    fn test_loads_invalid_jpeg() {
        pdfium();
        let jpeg = test_data_bytes!("file_types/pdf/broken_unfixable.pdf");
        assert!(Document::from_jpeg(jpeg, 100.0, 100.0).is_err());
    }

    #[test]
    fn test_loads_from_bytes() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/blank.pdf");
        let doc = Document::from_bytes(bytes.to_vec()).unwrap();
        let page_count = doc.page_count().unwrap();
        let saved = doc.inner.save_to_bytes().unwrap();
        let reloaded = Document::from_bytes(saved).unwrap();
        assert_eq!(reloaded.page_count().unwrap(), page_count);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
    fn test_loads_invalid_bytes() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/broken_unfixable.pdf");
        let _ = Document::from_bytes(bytes.to_vec()).unwrap();
    }

    #[test]
    fn test_creates_0_pages_document() {
        pdfium();
        let doc = Document::new().unwrap();
        assert_eq!(doc.page_count().unwrap(), 0);
    }

    #[test]
    fn test_returns_page_at_index() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/multipage.pdf");
        let doc = Document::from_bytes(bytes.to_vec()).unwrap();
        let page3 = doc.page(2);
        assert_eq!(page3.unwrap().text().unwrap(), "***\r\n***\r\n***");
    }

    #[test]
    fn test_import_page() {
        pdfium();
        let bytes_doc_1 = test_data_bytes!("file_types/pdf/blank.pdf");
        let bytes_doc_2 = test_data_bytes!("file_types/receipt.jpg");
        let doc = Document::from_bytes(bytes_doc_1.to_vec()).unwrap();
        let mut doc2 = Document::from_jpeg(bytes_doc_2, 10.0, 10.0).unwrap();
        doc2.import_pages(&doc, &[0]).unwrap();
        assert_eq!(doc2.page_count().unwrap(), 2);
    }

    #[test]
    fn test_import_pages() {
        pdfium();
        let bytes_multipage = test_data_bytes!("file_types/pdf/multipage.pdf");
        let bytes_blank = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc = Document::from_bytes(bytes_multipage.to_vec()).unwrap();
        let mut doc_blank = Document::from_bytes(bytes_blank.to_vec()).unwrap();
        doc_blank
            .import_pages(&doc, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
            .unwrap();
        assert_eq!(doc_blank.page_count().unwrap(), 13);
    }

    #[test]
    fn test_save() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/blank.pdf");
        let doc = Document::from_bytes(bytes.to_vec()).unwrap();
        let page_count = doc.page_count().unwrap();
        let mut buf = Vec::new();
        doc.save(&mut buf).unwrap();
        assert!(!buf.is_empty());
        let reloaded = Document::from_bytes(buf).unwrap();
        assert_eq!(reloaded.page_count().unwrap(), page_count);
    }

    #[test]
    #[should_panic(expected = "PageIndexOutOfBounds")]
    fn test_import_pages_out_of_bounds() {
        pdfium();
        let bytes_doc_1 = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let bytes_doc_2 = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc = Document::from_bytes(bytes_doc_1.to_vec()).unwrap();
        let mut doc2 = Document::from_bytes(bytes_doc_2.to_vec()).unwrap();
        doc2.import_pages(&doc, &[100, 101, 102]).unwrap();
    }

    #[test]
    fn test_add_text() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc = &mut Document::from_bytes(bytes.to_vec()).unwrap();
        let chars = vec![TextChar {
            char: 'A',
            font_name: String::from("Arial"),
            font_size: 12.0,
            font_weight: 300,
            stroke_color: Option::from([0, 0, 255, 255]),
            fill_color: None,
            font_flags: 0,
            bounds: [0.0, 0.0, 10.0, 10.0],
        }];
        doc.add_text(0, &chars).unwrap();
        assert_eq!(
            doc.inner
                .pages()
                .get(0)
                .unwrap()
                .text()
                .unwrap()
                .to_string(),
            "A"
        );
        assert_eq!(doc.page(0).unwrap().chars().unwrap().len(), 1);
        assert_eq!(doc.page(0).unwrap().chars().unwrap()[0].char, 'A');
        let font_size = doc.page(0).unwrap().chars().unwrap()[0].font_size;
        assert!((font_size - 12.0).abs() < 0.1);
        assert_eq!(
            doc.page(0).unwrap().chars().unwrap()[0].font_name,
            String::from("Helvetica")
        );
        let _ = doc;
    }

    #[test]
    fn test_add_text_resolves_font_families() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc = &mut Document::from_bytes(bytes.to_vec()).unwrap();
        let chars = vec![
            TextChar {
                char: 'T',
                font_name: String::from("Times New Roman Bold"),
                font_size: 12.0,
                font_weight: 700,
                stroke_color: None,
                fill_color: None,
                font_flags: 0,
                bounds: [0.0, 0.0, 10.0, 10.0],
            },
            TextChar {
                char: 'C',
                font_name: String::from("Courier"),
                font_size: 12.0,
                font_weight: 400,
                stroke_color: None,
                fill_color: None,
                font_flags: 0,
                bounds: [20.0, 0.0, 30.0, 10.0],
            },
            TextChar {
                char: 'H',
                font_name: String::from("Helvetica Oblique"),
                font_size: 12.0,
                font_weight: 400,
                stroke_color: None,
                fill_color: None,
                font_flags: 1 << 6,
                bounds: [40.0, 0.0, 50.0, 10.0],
            },
        ];
        doc.add_text(0, &chars).unwrap();
        let out = doc.page(0).unwrap().chars().unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].font_name, "Times-Bold");
        assert_eq!(out[1].font_name, "Courier");
        assert_eq!(out[2].font_name, "Helvetica-Oblique");
    }

    #[test]
    fn test_append_jpeg_page() {
        pdfium();
        let bytes = test_data_bytes!("file_types/receipt.jpg");
        let mut doc = Document::new().unwrap();
        doc.append_jpeg_page(bytes).unwrap();
        assert_eq!(doc.page_count().unwrap(), 1);
    }

    #[test]
    fn test_append_multiple_jpeg_pages() {
        pdfium();
        let bytes1: &[u8] = test_data_bytes!("file_types/receipt.jpg");
        let bytes_array = [bytes1, bytes1, bytes1];
        let mut doc = Document::new().unwrap();
        doc.append_multiple_jpeg_pages(&bytes_array).unwrap();
        assert_eq!(doc.page_count().unwrap(), 3);
    }

    #[test]
    fn test_has_text() {
        pdfium();
        let bytes_multipage: &[u8] = test_data_bytes!("file_types/pdf/multipage.pdf");
        let doc = Document::from_bytes(bytes_multipage.to_vec()).unwrap();
        assert!(doc.has_text().unwrap());
        let bytes_blank: &[u8] = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc_blank = Document::from_bytes(bytes_blank.to_vec()).unwrap();
        assert!(!doc_blank.has_text().unwrap());
    }

    #[test]
    fn test_extract_chars() {
        pdfium();
        let bytes_multipage: &[u8] = test_data_bytes!("file_types/pdf/multipage.pdf");
        let doc = Document::from_bytes(bytes_multipage.to_vec()).unwrap();
        for i in 0..doc.page_count().unwrap() {
            let chars = doc.page(i).unwrap().chars().unwrap();
            let text = doc.page(i).unwrap().text().unwrap();
            assert_eq!(chars.iter().map(|c| c.char).collect::<String>(), text);
            assert_eq!(
                text.replace("\r\n", ""),
                "*".repeat(((i + 1) * (i + 1)).into())
            );
        }
    }

    #[test]
    fn test_is_empty() {
        pdfium();
        let bytes_multipage: &[u8] = test_data_bytes!("file_types/pdf/multipage.pdf");
        let doc = Document::from_bytes(bytes_multipage.to_vec()).unwrap();
        assert!(!doc.has_no_content().unwrap());
        let bytes_blank: &[u8] = test_data_bytes!("file_types/pdf/blank.pdf");
        let doc_blank = Document::from_bytes(bytes_blank.to_vec()).unwrap();
        assert!(doc_blank.has_no_content().unwrap());
    }
}
