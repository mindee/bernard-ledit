use super::{bitmap::PyPdfBitmap, document::PyPdfDocument};
use crate::pdf::error::format_pdf_err;
use pyo3::prelude::*;

#[pyclass(name = "PdfPage", module = "bernard_ledit.pdf")]
/// A PDF page.
pub struct PyPdfPage {
    pub(crate) doc: Py<PyPdfDocument>,
    pub(crate) index: u16,
}

#[pymethods]
impl PyPdfPage {
    /// Extract the page text.
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .and_then(|p| p.text())
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    #[pyo3(signature = (scale = 1.0))]
    fn render_to_bitmap(&self, py: Python<'_>, scale: f32) -> PyResult<PyPdfBitmap> {
        self.doc.borrow(py).with_doc(|d| {
            let bitmap = d
                .page(self.index)
                .and_then(|p| p.render_to_bitmap(scale))
                .map_err(|e| format_pdf_err(&e))?;
            Ok(PyPdfBitmap::from(bitmap))
        })?
    }

    fn __repr__(&self) -> String {
        format!("PdfPage(index={})", self.index)
    }
}
