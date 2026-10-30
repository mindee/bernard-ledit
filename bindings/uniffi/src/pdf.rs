//! PDF documents: loading, compression, page extraction and repair.

use crate::{BernardLeditError, args};
use bernard_ledit::pdf::{self as core_pdf, Document};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

type Result<T> = std::result::Result<T, BernardLeditError>;

/// Binds `PDFium`, once. A dynamic build loads `library_path`, or the library `PDFIUM_PATH` names;
/// a static build ignores both. Every PDF function calls it with no path, so packages that bundle
/// `PDFium` only need to call it first with the bundled library's path.
///
/// # Errors
/// `Pdfium` if `PDFium` can't be loaded.
#[uniffi::export(default(library_path = None))]
pub fn bind_pdfium(library_path: Option<String>) -> Result<()> {
    bind(library_path)
}

#[cfg(not(feature = "static-link"))]
fn bind(library_path: Option<String>) -> Result<()> {
    if core_pdf::is_initialized() {
        return Ok(());
    }
    let path = library_path
        .or_else(|| std::env::var("PDFIUM_PATH").ok())
        .ok_or_else(|| {
            BernardLeditError::Pdfium(
                "PDFium isn't loaded: call bindPdfium with its path, or set PDFIUM_PATH".into(),
            )
        })?;
    Ok(core_pdf::initialize(&path)?)
}

#[cfg(feature = "static-link")]
fn bind(_library_path: Option<String>) -> Result<()> {
    Ok(core_pdf::initialize_static()?)
}

/// Binds `PDFium` from `PDFIUM_PATH` if nothing has bound it yet.
pub fn ensure_pdfium() -> Result<()> {
    bind(None)
}

/// A PDF document.
#[derive(uniffi::Object)]
pub struct PdfDocument {
    inner: Mutex<Document>,
}

impl PdfDocument {
    fn wrap(doc: Document) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(doc),
        })
    }

    fn doc(&self) -> MutexGuard<'_, Document> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[uniffi::export]
impl PdfDocument {
    /// Loads a PDF from its bytes.
    ///
    /// # Errors
    /// `Pdfium` if `PDFium` can't be loaded, or `data` isn't a PDF it can read.
    #[uniffi::constructor]
    pub fn new(data: &[u8]) -> Result<Self> {
        ensure_pdfium()?;
        Ok(Self {
            inner: Mutex::new(Document::from_bytes(data.to_vec())?),
        })
    }

    /// Number of pages.
    ///
    /// # Errors
    /// `InvalidArgument` if the document has more than 65535 pages.
    pub fn page_count(&self) -> Result<i32> {
        Ok(self.doc().page_count()?.into())
    }

    /// The document's text, pages separated by newlines.
    ///
    /// # Errors
    /// `Pdfium` if `PDFium` can't read a page.
    pub fn text(&self) -> Result<String> {
        Ok(self.doc().text()?)
    }

    /// Whether any page has text.
    ///
    /// # Errors
    /// `Pdfium` if `PDFium` can't read a page.
    pub fn has_text(&self) -> Result<bool> {
        Ok(self.doc().has_text()?)
    }

    /// Renders page `index` at 72 DPI as a JPEG of `quality` (1-100).
    ///
    /// # Errors
    /// `InvalidArgument` if an argument is out of range or there's no such page; `Pdfium` if
    /// `PDFium` can't render the page.
    pub fn rasterize_page(&self, index: i32, quality: i32) -> Result<Vec<u8>> {
        let index = args::page_index("index", index)?;
        let quality = args::quality(quality, 1)?;
        Ok(self.doc().rasterize_page(index, quality)?)
    }

    /// The document's bytes.
    ///
    /// # Errors
    /// `Pdfium` if `PDFium` can't save the document.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.doc().to_bytes()?)
    }
}

/// Recompresses a PDF's pages that have no text (scans) as JPEGs of `quality` (5-100) rendered at
/// `dpi` (1-1200). Returns the input unchanged if that doesn't make it smaller.
///
/// # Errors
/// `InvalidArgument` if an argument is out of range; `Image` if JPEG encoding fails; `Pdfium` if
/// `PDFium` can't be loaded, read the PDF, render a page or build the new document.
#[uniffi::export(default(quality = 85, dpi = 150.0))]
pub fn compress_pdf(data: &[u8], quality: i32, dpi: f64) -> Result<Vec<u8>> {
    let quality = args::quality(quality, 5)?;
    let dpi = args::dpi(dpi)?;
    ensure_pdfium()?;
    let doc = Document::from_bytes(data.to_vec())?;
    let Some(out) = doc.compress(quality, dpi, true)? else {
        return Ok(data.to_vec());
    };
    let bytes = out.to_bytes()?;
    Ok(if bytes.len() < data.len() {
        bytes
    } else {
        data.to_vec()
    })
}

/// Builds one document per group of 0-based page indexes. `data` is a PDF or an image, which counts
/// as a one-page document.
///
/// # Errors
/// `InvalidArgument` if an index is out of range, there are no groups, a group is empty, or there's
/// no such page; `Image` if `data` is neither a PDF nor a decodable image; `Pdfium` if `PDFium`
/// can't be loaded, read the PDF or build the documents.
#[uniffi::export]
pub fn extract_from_page_ranges(
    data: &[u8],
    page_indexes: Vec<Vec<i32>>,
) -> Result<Vec<Arc<PdfDocument>>> {
    let groups = page_indexes
        .into_iter()
        .enumerate()
        .map(|(i, group)| args::ints(&format!("page_indexes[{i}]"), &group, 0, u16::MAX))
        .collect::<Result<Vec<_>>>()?;
    ensure_pdfium()?;
    Ok(core_pdf::extract::extract_from_page_ranges(data, &groups)?
        .into_iter()
        .map(PdfDocument::wrap)
        .collect())
}

/// Drops every byte before the first `%PDF-` marker, if it's within `maximum_offset` bytes.
///
/// # Errors
/// `InvalidArgument` if `maximum_offset` is negative, or there's no marker before it.
#[uniffi::export(default(maximum_offset = 500))]
pub fn fix_pdf(data: &[u8], maximum_offset: i64) -> Result<Vec<u8>> {
    let offset = usize::try_from(maximum_offset).map_err(|_| {
        BernardLeditError::InvalidArgument(format!(
            "maximum_offset must be at least 0, got {maximum_offset}"
        ))
    })?;
    Ok(core_pdf::fix::fix_pdf(data, offset)?.to_vec())
}
