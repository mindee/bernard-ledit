use crate::input::PdfInput;
use crate::pdf::document::PyPdfDocument;
use crate::pdf::error::format_pdf_extract_err;
use pyo3::{PyResult, pyfunction};

#[pyfunction]
#[pyo3(signature = (data, page_indexes))]
#[allow(clippy::needless_pass_by_value)]
pub fn extract_from_page_ranges(
    data: PdfInput<'_>,
    page_indexes: Vec<Vec<u16>>,
) -> PyResult<Vec<PyPdfDocument>> {
    let bytes = data.into_vec()?;
    bernard_ledit::pdf::extract::extract_from_page_ranges(&bytes, &page_indexes)
        .map(|docs| docs.into_iter().map(PyPdfDocument::from).collect())
        .map_err(|e| format_pdf_extract_err(&e))
}
