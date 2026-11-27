use super::document::PyPdfDocument;
use crate::image::image_data::PyImage;
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
    fn render(&self, py: Python<'_>, scale: f32) -> PyResult<PyImage> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .and_then(|p| p.render(scale))
                .map(PyImage::from_core)
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    fn __repr__(&self) -> String {
        format!("PdfPage(index={})", self.index)
    }
}
