use super::{
    error::{closed_err, map_pdf_err},
    page::PdfPage,
};
use crate::pdf::error::format_pdf_extract_err;
use crate::pdf::text_char::PhpTextChar;
use bernard_ledit::pdf::{Document, TextChar};
use ext_php_rs::{binary::Binary, binary_slice::BinarySlice, prelude::*, zend::ce};
use std::sync::{Arc, Mutex};

pub(crate) type SharedDoc = Arc<Mutex<Option<Document>>>;

#[php_class]
#[php(name = "BernardLedit\\Pdf\\PdfDocument")]
pub struct PhpPdfDocument {
    pub(crate) inner: SharedDoc,
}

#[php_impl]
impl PhpPdfDocument {
    /// Create a new instance from bytes.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't be loaded, or `$data` isn't a PDF it
    ///   can read.
    #[allow(clippy::needless_pass_by_value)]
    pub fn __construct(data: BinarySlice<u8>) -> PhpResult<Self> {
        super::ensure_pdfium()?;
        let doc = Document::from_bytes(data.to_vec()).map_err(|e| map_pdf_err(&e))?;
        Ok(Self::wrap(doc))
    }

    /// `PdfDocument::create()` empty document (new is reserved).
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't be loaded, or can't create a
    ///   document.
    pub fn create() -> PhpResult<Self> {
        super::ensure_pdfium()?;
        Document::new().map(Self::wrap).map_err(|e| map_pdf_err(&e))
    }

    /// Returns the page count.
    /// # Errors
    /// - `\ValueError` if the document has more than 65535 pages.
    /// - `\Exception` if the document is closed.
    pub fn get_page_count(&self) -> PhpResult<u16> {
        self.with_doc(|d| d.page_count().map_err(|e| map_pdf_err(&e)))?
    }

    /// Return the specified page.
    /// # Errors
    /// - `\ValueError` if there's no page `$index`.
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't load the page.
    /// - `\Exception` if the document is closed.
    pub fn get_page(&self, index: u16) -> PhpResult<PdfPage> {
        self.with_doc(|d| d.page(index).map(|_| ()).map_err(|e| map_pdf_err(&e)))??;
        Ok(PdfPage {
            doc: Arc::clone(&self.inner),
            index,
        })
    }

    /// Append pages `indexes` of `$src` to `$this`.
    /// # Errors
    /// - `\ValueError` if `$src` is this document, or has no page at one of `$indexes`.
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't copy the pages.
    /// - `\Exception` if either document is closed.
    #[allow(clippy::significant_drop_tightening)]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::use_self)]
    pub fn import_pages(&self, src: &PhpPdfDocument, indexes: Vec<u16>) -> PhpResult<()> {
        if Arc::ptr_eq(&self.inner, &src.inner) {
            return Err(PhpException::new(
                "Cannot import pages from a document into itself".into(),
                0,
                ce::value_error(),
            ));
        }
        let mut dst_guard = self.inner.lock().map_err(|_| closed_err())?;
        let src_guard = src.inner.lock().map_err(|_| closed_err())?;
        let dst = dst_guard.as_mut().ok_or_else(closed_err)?;
        let s = src_guard.as_ref().ok_or_else(closed_err)?;
        dst.import_pages(s, &indexes).map_err(|e| map_pdf_err(&e))
    }

    /// Returns the PDF bytes (PHP writes files itself and keeps I/O out of Rust).
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't save the document.
    /// - `\Exception` if the document is closed.
    pub fn as_bytes(&self) -> PhpResult<Binary<u8>> {
        let mut out: Vec<u8> = Vec::new();
        self.with_doc(|d| d.save(&mut out).map_err(|e| map_pdf_err(&e)))??;
        Ok(Binary::new(out))
    }

    /// Rasterizes a page from a given index as a JPEG.
    /// # Errors
    /// - `\ValueError` if there's no page `$index`.
    /// - `\BernardLedit\Image\ImageException` if `$quality` is above 100, or JPEG encoding fails.
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't render the page.
    /// - `\Exception` if the document is closed.
    pub fn rasterize_page_to_jpeg_bytes(&self, index: u16, quality: u8) -> PhpResult<Binary<u8>> {
        self.with_doc(|d| {
            d.rasterize_page(index, quality)
                .map_err(|e| map_pdf_err(&e))
        })?
        .map(Binary::new)
    }

    /// Appends a JPEG as a page of the document.
    /// # Errors
    /// - `\BernardLedit\Image\ImageException` if `$jpeg` is invalid, truncated or in an unsupported
    ///   color space.
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't append the page.
    /// - `\Exception` if the document is closed.
    #[allow(clippy::needless_pass_by_value)]
    pub fn append_jpeg_page(&self, jpeg: BinarySlice<u8>) -> PhpResult<()> {
        self.with_doc_mut(|d| d.append_jpeg_page(&jpeg).map_err(|e| map_pdf_err(&e)))?
    }

    /// Append multiple JPEG pages to the document.
    /// # Errors
    /// - `\BernardLedit\Image\ImageException` if one of `$jpegs` is invalid, truncated or in an
    ///   unsupported color space.
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't append a page.
    /// - `\Exception` if the document is closed.
    #[allow(clippy::needless_pass_by_value)]
    pub fn append_multiple_jpeg_pages(&self, jpegs: Vec<BinarySlice<u8>>) -> PhpResult<()> {
        let vec_slices: Vec<&[u8]> = jpegs.iter().map(|jpeg| jpeg.as_ref()).collect();
        self.with_doc_mut(|d| {
            d.append_multiple_jpeg_pages(&vec_slices)
                .map_err(|e| map_pdf_err(&e))
        })
        .flatten()
    }

    /// Retrieves the entire text from a document as a string.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't read a page's text.
    /// - `\Exception` if the document is closed.
    pub fn get_text(&self) -> PhpResult<String> {
        self.with_doc(|d| d.text().map_err(|e| map_pdf_err(&e)))?
    }

    /// Append text to a page.
    /// # Errors
    /// - `\ValueError` if there's no page `$page_idx`.
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't add the text.
    /// - `\Exception` if the document is closed.
    #[allow(clippy::needless_pass_by_value)]
    pub fn add_text(&self, page_idx: u16, chars: Vec<&PhpTextChar>) -> PhpResult<()> {
        let contiguous_chars: Vec<TextChar> = chars.into_iter().map(Into::into).collect();

        self.with_doc_mut(|d| {
            d.add_text(page_idx.into(), contiguous_chars.as_ref())
                .map_err(|e| map_pdf_err(&e))
        })
        .flatten()
    }

    /// Check if the document has any text.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't read a page.
    /// - `\Exception` if the document is closed.
    pub fn has_text(&self) -> PhpResult<bool> {
        self.with_doc(|d| d.has_text().map_err(|e| map_pdf_err(&e)))?
    }

    /// Returns true if the document has no content.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't read a page.
    /// - `\Exception` if the document is closed.
    pub fn has_no_content(&self) -> PhpResult<bool> {
        self.with_doc(|d| d.has_no_content().map_err(|e| map_pdf_err(&e)))?
    }

    /// Close the document. Closing it again does nothing.
    pub fn close(&self) {
        *self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
    /// Recompresses pages that have no text layer (scans) as JPEG.
    /// Returns `null` when there is nothing to gain (e.g. text-only documents).
    /// # Errors
    /// - `\BernardLedit\Image\ImageException` if `$quality` is outside 5-100, or JPEG encoding
    ///   fails.
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't render a page, or build the new
    ///   document.
    /// - `\Exception` if the document is closed.
    #[php(defaults(quality = 85, optimize = true, dpi = 150.0))]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "Core takes in f32, PHP needs f64"
    )]
    pub fn compress(&self, quality: u8, optimize: bool, dpi: f64) -> PhpResult<Option<Self>> {
        let out = self
            .with_doc(|d| d.compress(quality, dpi as f32, optimize))?
            .map_err(|e| map_pdf_err(&e))?;
        Ok(out.map(Self::wrap))
    }
}

/// PDF document wrapper object, not included in the PHP API.
impl PhpPdfDocument {
    #[must_use]
    pub fn wrap(doc: Document) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(doc))),
        }
    }

    #[allow(clippy::significant_drop_tightening)]
    pub(crate) fn with_doc<R>(&self, f: impl FnOnce(&Document) -> R) -> PhpResult<R> {
        let g = self.inner.lock().map_err(|_| closed_err())?;
        let d = g.as_ref().ok_or_else(closed_err)?;
        Ok(f(d))
    }

    #[allow(clippy::significant_drop_tightening)]
    pub(crate) fn with_doc_mut<R>(&self, f: impl FnOnce(&mut Document) -> R) -> PhpResult<R> {
        let mut g = self.inner.lock().map_err(|_| closed_err())?;
        let d = g.as_mut().ok_or_else(closed_err)?;
        Ok(f(d))
    }
}

/// Compresses PDF bytes. Returns the input unchanged if it can't be made smaller.
/// # Errors
/// - `\BernardLedit\Image\ImageException` if `$quality` is outside 5-100, or JPEG encoding fails.
/// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't be loaded, `$data` isn't a PDF it can
///   read, or it can't render a page or build the new document.
#[php_function]
#[php(name = "BernardLedit\\Pdf\\compress")]
#[php(defaults(quality = 85, optimize = None, dpi = None))]
#[allow(clippy::needless_pass_by_value)]
#[allow(
    clippy::cast_possible_truncation,
    reason = "Core takes in f32, PHP needs f64"
)]
pub fn compress(
    data: BinarySlice<u8>,
    quality: u8,
    optimize: Option<bool>,
    dpi: Option<f64>,
) -> PhpResult<Binary<u8>> {
    super::ensure_pdfium()?;
    let doc = Document::from_bytes(data.to_vec()).map_err(|e| map_pdf_err(&e))?;
    let Some(out) = doc
        .compress(
            quality,
            dpi.unwrap_or(150.0) as f32,
            optimize.unwrap_or(true),
        )
        .map_err(|e| map_pdf_err(&e))?
    else {
        return Ok(Binary::new(data.to_vec()));
    };
    let bytes = out.to_bytes().map_err(|e| map_pdf_err(&e))?;
    Ok(Binary::new(if bytes.len() < data.len() {
        bytes
    } else {
        data.to_vec()
    }))
}

/// Builds one new document per group of 0-indexed page numbers.
/// `data` is a PDF or an image (treated as a one-page document).
/// # Errors
/// - `\ValueError` if `$page_indexes` or one of its groups is empty, or there's no page at one of
///   the indexes.
/// - `\BernardLedit\Image\ImageException` if `$data` is neither a PDF nor an identifiable,
///   decodable image.
/// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't be loaded, `$data` is a PDF it can't
///   read, or it can't build the documents.
#[php_function]
#[php(name = "BernardLedit\\Pdf\\extractFromPageRanges")]
#[allow(clippy::needless_pass_by_value)]
#[allow(
    clippy::cast_possible_truncation,
    reason = "Core takes in f32, PHP needs f64"
)]
pub fn extract_from_page_ranges(
    data: BinarySlice<u8>,
    page_indexes: Vec<Vec<u16>>,
) -> PhpResult<Vec<PhpPdfDocument>> {
    super::ensure_pdfium()?;
    bernard_ledit::pdf::extract::extract_from_page_ranges(&data, &page_indexes)
        .map(|docs| docs.into_iter().map(PhpPdfDocument::wrap).collect())
        .map_err(|e| format_pdf_extract_err(&e))
}

/// Registers the internal functions in this module.
pub fn register_functions(builder: ModuleBuilder) -> ModuleBuilder {
    builder
        .function(wrap_function!(compress))
        .function(wrap_function!(extract_from_page_ranges))
}
