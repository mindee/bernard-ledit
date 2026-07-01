use super::{bitmap::PyPdfBitmap, document::PyPdfDocument, text_char::PyTextChar};
use crate::pdf::error::format_pdf_err;
use pyo3::prelude::*;

#[pyclass(name = "PdfPage", module = "bernard_ledit.pdf")]
/// A PDF page.
pub struct PyPdfPage {
    pub(crate) doc: Py<PyPdfDocument>,
    pub(crate) index: u16,
}

/// PDF page size.
#[pyclass(name = "PageSize", module = "bernard_ledit.pdf")]
pub struct PageSize {
    #[pyo3(get)]
    width: f32,
    #[pyo3(get)]
    height: f32,
}

#[pymethods]
impl PyPdfPage {
    fn get_size(&self, py: Python<'_>) -> PyResult<PageSize> {
        let (width, height) = self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .map(|p| p.size())
                .map_err(|e| format_pdf_err(&e))
        })??;

        Ok(PageSize { width, height })
    }

    fn is_empty(&self, py: Python<'_>) -> PyResult<bool> {
        let emptiness = self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .map(|p| p.is_empty())
                .map_err(|e| format_pdf_err(&e))
        })??;
        emptiness.map_err(|e| format_pdf_err(&e))
    }

    /// Extract the page text.
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .and_then(|p| p.text())
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    /// Extract the page text characters.
    fn chars(&self, py: Python<'_>) -> PyResult<Vec<PyTextChar>> {
        self.doc.borrow(py).with_doc(|d| {
            let chars = d
                .page(self.index)
                .and_then(|p| p.chars())
                .map_err(|e| format_pdf_err(&e))?;
            Ok(chars.into_iter().map(PyTextChar::from).collect())
        })?
    }

    /// Page rotation in degrees.
    fn rotation(&self, py: Python<'_>) -> PyResult<u32> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .and_then(|p| p.rotation())
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    #[pyo3(signature = (scale = 1.0))]
    fn render(&self, py: Python<'_>, scale: f32) -> PyResult<PyPdfBitmap> {
        self.doc.borrow(py).with_doc(|d| {
            let bitmap = d
                .page(self.index)
                .and_then(|p| p.render(scale))
                .map_err(|e| format_pdf_err(&e))?;
            Ok(PyPdfBitmap::from(bitmap))
        })?
    }

    fn __repr__(&self) -> String {
        format!("PdfPage(index={})", self.index)
    }
}
