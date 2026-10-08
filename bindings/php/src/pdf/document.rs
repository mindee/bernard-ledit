use super::{
    error::{closed_err, map_pdf_err},
    page::PdfPage,
};
use crate::pdf::text_char::PhpTextChar;
use bernard_ledit::pdf::{Document, TextChar};
use ext_php_rs::{binary::Binary, binary_slice::BinarySlice, prelude::*, zend::ce};
use std::sync::{Arc, Mutex};

pub(crate) type SharedDoc = Arc<Mutex<Option<Document>>>;

#[php_class]
#[php(name = "BernardLedit\\Pdf\\PdfDocument")]
pub struct PdfDocument {
    pub(crate) inner: SharedDoc,
}

#[php_impl]
impl PdfDocument {
    /// Create a new instance from bytes: `new PdfDocument(string $bytes)`.
    /// # Errors
    /// Returns a PHP Exception if the creation fails.
    #[allow(clippy::needless_pass_by_value)]
    pub fn __construct(data: BinarySlice<u8>) -> PhpResult<Self> {
        super::ensure_pdfium()?;
        let doc = Document::from_bytes(data.to_vec()).map_err(|e| map_pdf_err(&e))?;
        Ok(Self::wrap(doc))
    }

    /// Create an instance from file `PdfDocument::fromFile(string $path)`.
    /// # Errors
    /// Returns a PHP Exception if the creation fails.
    pub fn from_file(path: &str) -> PhpResult<Self> {
        super::ensure_pdfium()?;
        let bytes = std::fs::read(path)
            .map_err(|e| PhpException::from_message(format!("Failed to read {path}: {e}")))?;
        let doc = Document::from_bytes(bytes).map_err(|e| map_pdf_err(&e))?;
        Ok(Self::wrap(doc))
    }

    /// `PdfDocument::create()` empty document (new is reserved).
    /// # Errors
    /// Returns a PHP Exception if the creation fails.
    pub fn create() -> PhpResult<Self> {
        super::ensure_pdfium()?;
        Document::new().map(Self::wrap).map_err(|e| map_pdf_err(&e))
    }

    /// Returns the page count.
    /// # Errors
    /// Returns a PHP Exception if the count fails.
    pub fn page_count(&self) -> PhpResult<u16> {
        self.with_doc(|d| d.page_count().map_err(|e| map_pdf_err(&e)))?
    }

    /// Validates the index now (opens & drops the page), returns a lightweight handle.
    /// # Errors
    /// Returns a PHP Exception if the page cannot be retrieved.
    pub fn get_page(&self, index: u16) -> PhpResult<PdfPage> {
        self.with_doc(|d| d.page(index).map(|_| ()).map_err(|e| map_pdf_err(&e)))??;
        Ok(PdfPage {
            doc: Arc::clone(&self.inner),
            index,
        })
    }

    /// Append pages `indexes` of `$src` to `$this`.
    /// # Errors
    /// Returns a PHP Exception if the import fails.
    #[allow(clippy::significant_drop_tightening)]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::use_self)]
    pub fn import_pages(&self, src: &PdfDocument, indexes: Vec<u16>) -> PhpResult<()> {
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
    /// Returns a PHP Exception if the save fails.
    pub fn save(&self) -> PhpResult<Binary<u8>> {
        let mut out: Vec<u8> = Vec::new();
        self.with_doc(|d| d.save(&mut out).map_err(|e| map_pdf_err(&e)))??;
        Ok(Binary::new(out))
    }

    /// Saves to a file.
    /// # Errors
    /// Returns a PHP Exception if the save fails.
    pub fn save_to_file(&self, path: &str) -> PhpResult<()> {
        self.with_doc(|d| d.save_to_file(path).map_err(|e| map_pdf_err(&e)))?
    }

    /// Rasterizes a page from a given index.
    /// # Errors
    /// Returns a PHP Exception if the rasterization fails.
    pub fn rasterize_page(&self, index: u16, quality: u8) -> PhpResult<Binary<u8>> {
        self.with_doc(|d| {
            d.rasterize_page(index, quality)
                .map_err(|e| map_pdf_err(&e))
        })?
        .map(Binary::new)
    }

    /// Appends a sequence of JPEG pages as pages to the document.
    /// # Errors
    /// Returns a PHP Exception if the append fails.
    #[allow(clippy::needless_pass_by_value)]
    pub fn append_jpeg_page(&self, jpeg: BinarySlice<u8>) -> PhpResult<()> {
        self.with_doc_mut(|d| d.append_jpeg_page(&jpeg).map_err(|e| map_pdf_err(&e)))?
    }

    /// Append multiple JPEG pages to the document.
    /// # Errors
    /// Returns a PHP Exception if the append fails.
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
    /// Returns a PHP Exception if the text retrieval fails.
    pub fn text(&self) -> PhpResult<String> {
        self.with_doc(|d| d.text().map_err(|e| map_pdf_err(&e)))?
    }

    /// Append text to a page.
    /// # Errors
    /// Returns a PHP Exception if the text append fails.
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
    /// Returns a PHP Exception if the check fails.
    pub fn has_text(&self) -> PhpResult<bool> {
        self.with_doc(|d| d.has_text().map_err(|e| map_pdf_err(&e)))?
    }

    /// Check if the document has any content.
    /// # Errors
    /// Returns a PHP Exception if the check fails.
    pub fn has_no_content(&self) -> PhpResult<bool> {
        self.with_doc(|d| d.has_no_content().map_err(|e| map_pdf_err(&e)))?
    }

    /// Close the document.
    /// # Errors
    /// Returns a PHP Exception if the close fails.
    pub fn close(&self) -> PhpResult<()> {
        let mut g = self.inner.lock().map_err(|_| closed_err())?;
        *g = None;
        drop(g);
        Ok(())
    }
}

/// PDF document wrapper object.
impl PdfDocument {
    fn wrap(doc: Document) -> Self {
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
