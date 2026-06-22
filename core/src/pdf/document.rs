use crate::pdf::error::{PdfBuilderError, PdfError};
use crate::pdf::jpeg::build_jpeg_pdf;
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
    pub fn from_jpeg(jpeg_bytes: &[u8], width: f32, height: f32) -> Result<Self, PdfBuilderError> {
        let pdf_bytes = build_jpeg_pdf(jpeg_bytes, width, height)?;
        Self::from_bytes(pdf_bytes).map_err(PdfBuilderError::Pdf)
    }
}

#[cfg(test)]
mod tests {
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
}
