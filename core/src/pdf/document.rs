use crate::image::ImageError;
use crate::pdf::error::{ImageError as JpegError, PdfBuilderError, PdfError};
use crate::pdf::jpeg::{build_jpeg_pdf, build_jpeg_pdf_autosize};
use crate::pdf::text_char::TextChar;
use crate::pdf::{page::Page, pdfium};

use ::image::ImageFormat;
use pdfium_render::prelude::*;

/// A struct that represents a PDF document.
///
/// The `Document` struct acts as a wrapper around the `PdfDocument`
/// to provide functionality for handling and interacting with PDF files.
pub struct Document {
    inner: PdfDocument<'static>,
}

impl Document {
    /// Decode a PDF from in-memory bytes.
    ///
    /// # Errors
    ///
    /// Returns a [`PdfError`] if decoding the PDF fails.
    pub fn decode(data: &[u8]) -> Result<Self, PdfError> {
        let inner = pdfium().load_pdf_from_byte_vec(data.to_vec(), None)?;
        Ok(Self { inner })
    }

    /// Save to a buffer and return it as an u8 vec.
    /// # Errors
    /// Returns a [`PdfError`] if saving the document fails.
    pub fn to_bytes(&self) -> Result<Vec<u8>, PdfError> {
        let mut buffer = Vec::new();
        self.inner.save_to_writer(&mut buffer)?;
        Ok(buffer)
    }
}

impl Document {
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
        let p = self.inner.pages().get(self.page_index(index)?)?;
        Ok(Page::new(p))
    }

    /// Returns `index` as a `PDFium` page index.
    ///
    /// # Errors
    /// Returns [`PdfError::PageIndexOutOfBounds`] if the document has no such page.
    fn page_index(&self, index: u16) -> Result<PdfPageIndex, PdfError> {
        let page_count = self.page_count()?;
        if index >= page_count {
            return Err(PdfError::PageIndexOutOfBounds { index, page_count });
        }
        Ok(index.into())
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
    /// The JPEG bytes are embedded **as-is** using `/Filter /DCTDecode` so no pixel
    /// decode or re-encode occurs. The resulting PDF is approximately
    /// `jpeg_bytes.len() + 500 bytes` of PDF framing overhead.
    ///
    /// # Errors
    /// Returns a [`PdfBuilderError`] if the JPEG header cannot be parsed or
    /// if PDF creation fails.
    pub fn from_jpeg(jpeg_bytes: &[u8], width: f64, height: f64) -> Result<Self, PdfBuilderError> {
        let pdf_bytes = build_jpeg_pdf(jpeg_bytes, width, height)?;
        Self::decode(&pdf_bytes).map_err(PdfBuilderError::Pdf)
    }

    /// Build a single-page PDF from a JPEG-encoded image.
    ///
    /// `width` and `height` are determined from the JPEG header.
    /// # Errors
    /// Returns a [`PdfBuilderError`] if the JPEG header cannot be parsed or
    /// if PDF creation fails.
    pub fn from_jpeg_autosize(jpeg_bytes: &[u8]) -> Result<Self, PdfBuilderError> {
        let pdf_bytes = build_jpeg_pdf_autosize(jpeg_bytes)?;
        Self::decode(&pdf_bytes).map_err(PdfBuilderError::Pdf)
    }

    /// Appends a JPEG as a new page.
    /// # Panics
    /// If the JPEG cannot be decoded.
    /// # Errors
    /// Returns a [`PdfError`] if the JPEG is invalid or the page cannot be appended.
    /// Appends a JPEG as a new page, given its size.
    /// # Panics
    /// If the JPEG cannot be decoded.
    /// # Errors
    /// Returns a [`PdfError`] if the JPEG is invalid or the page cannot be appended.
    fn append_jpeg_page_sized(
        &mut self,
        jpeg: &[u8],
        w_pt: f32,
        h_pt: f32,
    ) -> Result<(), PdfError> {
        let doc = Self::from_jpeg(jpeg, f64::from(w_pt), f64::from(h_pt)).map_err(|e| match e {
            PdfBuilderError::Pdf(inner) => inner,
            PdfBuilderError::Image(
                inner @ (JpegError::InvalidJpeg | JpegError::UnsupportedColorSpace(_)),
            ) => ImageError::Decode(format!("invalid JPEG: {inner}")).into(),
            other => PdfError::Other(other.to_string()),
        })?;
        self.inner.pages_mut().append(&doc.inner)?;
        Ok(())
    }

    /// Appends a page, independently of its content.
    /// # Errors
    /// Returns a [`PdfError`] if the page cannot be appended.
    fn append_page(&mut self, document: &PdfDocument, page_index: i32) -> Result<(), PdfError> {
        let index = PdfPageIndex::from(page_index);
        self.inner
            .pages_mut()
            .copy_page_from_document(document, index, index)
            .map_err(PdfError::from)
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

    /// Compresses each page of a PDF by rasterizing it and re-encoding it as JPEG.
    /// `optimize` controls JPEG Huffman table optimization.
    ///
    /// # Errors
    /// Returns an error if the PDF cannot be parsed, rasterized, or re-assembled.
    pub fn compress_to_jpeg(
        &self,
        quality: u8,
        optimize: bool,
        dpi: f32,
    ) -> Result<Option<Self>, PdfError> {
        let len = self.page_count()?;
        let needs_work: Vec<bool> = (0..len)
            .map(|i| Ok(self.inner.pages().get(i.into())?.text()?.is_empty()))
            .collect::<Result<_, PdfError>>()?;
        if !needs_work.contains(&true) {
            return Ok(None);
        }

        let mut out = Self::new()?;
        for (i, &rasterize) in needs_work.iter().enumerate() {
            let i = u16::try_from(i).map_err(|_| PdfError::PageCountOutOfBounds)?;
            if rasterize {
                let page = self.page(i)?;
                let (w_pt, h_pt) = page.size();
                let img = page.render(dpi / 72.0)?;
                if quality < 5 {
                    return Err(ImageError::ProgressiveQualityTooLow {
                        jpeg_quality: quality,
                    }
                    .into());
                }
                let jpeg = img.encode(ImageFormat::Jpeg, quality, optimize)?;
                out.append_jpeg_page_sized(&jpeg, w_pt, h_pt)?;
            } else {
                out.append_page(&self.inner, i.into())?;
            }
        }
        Ok(Some(out))
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf::{Document, PdfError, pdfium};

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
    fn test_decodes_from_bytes() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/blank.pdf");
        let doc = Document::decode(bytes).unwrap();
        let page_count = doc.page_count().unwrap();
        let saved = doc.inner.save_to_bytes().unwrap();
        let reloaded = Document::decode(&saved).unwrap();
        assert_eq!(reloaded.page_count().unwrap(), page_count);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
    fn test_decodes_invalid_bytes() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/broken_unfixable.pdf");
        let _ = Document::decode(bytes).unwrap();
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
        let doc = Document::decode(bytes).unwrap();
        let page3 = doc.page(2);
        assert_eq!(page3.unwrap().text().unwrap(), "***\r\n***\r\n***");
    }

    #[test]
    fn test_import_page() {
        pdfium();
        let bytes_doc_1 = test_data_bytes!("file_types/pdf/blank.pdf");
        let bytes_doc_2 = test_data_bytes!("file_types/receipt.jpg");
        let doc = Document::decode(bytes_doc_1).unwrap();
        let mut doc2 = Document::from_jpeg(bytes_doc_2, 10.0, 10.0).unwrap();
        doc2.import_pages(&doc, &[0]).unwrap();
        assert_eq!(doc2.page_count().unwrap(), 2);
    }

    #[test]
    fn test_import_pages() {
        pdfium();
        let bytes_multipage = test_data_bytes!("file_types/pdf/multipage.pdf");
        let bytes_blank = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc = Document::decode(bytes_multipage).unwrap();
        let mut doc_blank = Document::decode(bytes_blank).unwrap();
        doc_blank
            .import_pages(&doc, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
            .unwrap();
        assert_eq!(doc_blank.page_count().unwrap(), 13);
    }

    #[test]
    fn test_to_bytes() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/blank.pdf");
        let doc = Document::decode(bytes).unwrap();
        let page_count = doc.page_count().unwrap();
        let output = doc.to_bytes().unwrap();
        assert_ne!(output.len(), 0);
        let reloaded = Document::decode(&output).unwrap();
        assert_eq!(reloaded.page_count().unwrap(), page_count);
    }

    #[test]
    #[should_panic(expected = "PageIndexOutOfBounds")]
    fn test_import_pages_out_of_bounds() {
        pdfium();
        let bytes_doc_1 = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let bytes_doc_2 = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc = Document::decode(bytes_doc_1).unwrap();
        let mut doc2 = Document::decode(bytes_doc_2).unwrap();
        doc2.import_pages(&doc, &[100, 101, 102]).unwrap();
    }
    #[test]
    fn test_page_out_of_bounds() {
        pdfium();
        let bytes = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc = Document::decode(bytes).unwrap();
        let out_of_bounds = |r: Result<(), PdfError>| {
            matches!(
                r,
                Err(PdfError::PageIndexOutOfBounds {
                    index: 1,
                    page_count: 1
                })
            )
        };
        assert!(out_of_bounds(doc.page(1).map(|_| ())));
    }

    #[test]
    fn test_has_text() {
        pdfium();
        let bytes_multipage: &[u8] = test_data_bytes!("file_types/pdf/multipage.pdf");
        let doc = Document::decode(bytes_multipage).unwrap();
        assert!(doc.has_text().unwrap());
        let bytes_blank: &[u8] = test_data_bytes!("file_types/pdf/blank_1.pdf");
        let doc_blank = Document::decode(bytes_blank).unwrap();
        assert!(!doc_blank.has_text().unwrap());
    }

    #[test]
    fn test_extract_chars() {
        pdfium();
        let bytes_multipage: &[u8] = test_data_bytes!("file_types/pdf/multipage.pdf");
        let doc = Document::decode(bytes_multipage).unwrap();
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
        let doc = Document::decode(bytes_multipage).unwrap();
        assert!(!doc.has_no_content().unwrap());
        let bytes_blank: &[u8] = test_data_bytes!("file_types/pdf/blank.pdf");
        let doc_blank = Document::decode(bytes_blank).unwrap();
        assert!(doc_blank.has_no_content().unwrap());
    }
}
