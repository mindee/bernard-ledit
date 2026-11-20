use super::document::PyPdfDocument;
use crate::image::image_data::PyImage;
use crate::pdf::error::format_pdf_err;
use crate::pdf::text_char::PyTextChar;
use pyo3::prelude::*;

#[pyclass(name = "PdfPage", module = "bernard_ledit.pdf")]
/// A PDF page.
pub struct PyPdfPage {
    pub(crate) doc: Py<PyPdfDocument>,
    pub(crate) index: u16,
}

#[pymethods]
impl PyPdfPage {
    /// Page dimensions in PDF points, including page rotation.
    #[getter]
    fn size(&self, py: Python<'_>) -> PyResult<(f32, f32)> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .map(|p| p.size())
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    /// Clockwise page rotation in degrees.
    #[getter]
    fn rotation(&self, py: Python<'_>) -> PyResult<f32> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .and_then(|p| p.rotation())
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    /// Unrotated page bounds: (left, bottom, right, top).
    #[getter]
    fn bounding_box(&self, py: Python<'_>) -> PyResult<(f32, f32, f32, f32)> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .and_then(|p| p.bounding_box())
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    /// Characters with tight bounds in (bottom, left, top, right) PDF coordinates.
    fn chars(&self, py: Python<'_>) -> PyResult<Vec<PyTextChar>> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .and_then(|p| p.tight_chars())
                .map(|chars| chars.into_iter().map(PyTextChar::from).collect())
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    /// Whether visible native text is not covered by a page-sized image.
    #[pyo3(signature = (image_area_threshold = 0.8))]
    fn is_native(&self, py: Python<'_>, image_area_threshold: f32) -> PyResult<bool> {
        self.doc.borrow(py).with_doc(|d| {
            d.page(self.index)
                .and_then(|p| p.is_native(image_area_threshold))
                .map_err(|e| format_pdf_err(&e))
        })?
    }

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
