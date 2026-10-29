use super::{
    document::SharedDoc,
    error::{closed_err, map_pdf_err},
};
use crate::pdf::{bitmap::PhpPdfBitmap, text_char::PhpTextChar};
use bernard_ledit::pdf::{Page, PdfError};
use ext_php_rs::prelude::*;

#[php_class]
#[php(name = "BernardLedit\\Pdf\\PdfPage")]
pub struct PdfPage {
    pub(crate) doc: SharedDoc,
    pub(crate) index: u16,
}

#[php_impl]
impl PdfPage {
    /// Index of the page.
    #[must_use]
    pub const fn get_index(&self) -> u16 {
        self.index
    }

    /// `[width, height]` in points.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't load the page.
    /// - `\Exception` if the document is closed.
    pub fn get_size(&self) -> PhpResult<Vec<f64>> {
        self.with_page(|p| {
            let (w, h) = p.size();
            Ok(vec![f64::from(w), f64::from(h)])
        })
    }

    /// Checks whether the page is empty.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't read the page.
    /// - `\Exception` if the document is closed.
    pub fn is_empty(&self) -> PhpResult<bool> {
        self.with_page(|p| p.is_empty())
    }

    /// Returns the characters on the page.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't read the page's characters.
    /// - `\Exception` if the document is closed.
    pub fn get_chars(&self) -> PhpResult<Vec<PhpTextChar>> {
        self.with_page(|p| Ok(p.chars()?.into_iter().map(PhpTextChar::from).collect()))
    }

    /// Returns the text of the page.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't read the page's text.
    /// - `\Exception` if the document is closed.
    pub fn get_text(&self) -> PhpResult<String> {
        self.with_page(|p| p.text())
    }

    /// Renders the page.
    /// # Errors
    /// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't render the page.
    /// - `\Exception` if the document is closed.
    #[php(defaults(scale = 1.0_f32))]
    pub fn render_to_bitmap(&self, scale: f32) -> PhpResult<PhpPdfBitmap> {
        self.with_page(|p| p.render_to_bitmap(scale))
            .map(PhpPdfBitmap::from)
    }
}

impl PdfPage {
    /// Re-opens the page under the lock every call (Page<'a> borrows the Document, which cannot be stored).
    /// # Errors
    /// Returns a PHP Exception if the `PdfDocument` is inaccessible.
    #[allow(clippy::significant_drop_tightening)]
    fn with_page<R>(&self, f: impl FnOnce(Page<'_>) -> Result<R, PdfError>) -> PhpResult<R> {
        let g = self.doc.lock().map_err(|_| closed_err())?;
        let d = g.as_ref().ok_or_else(closed_err)?;
        let page = d.page(self.index).map_err(|e| map_pdf_err(&e))?;
        f(page).map_err(|e| map_pdf_err(&e))
    }
}
