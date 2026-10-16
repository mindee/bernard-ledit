use crate::input::PdfInput;
use crate::pdf::document::PyPdfDocument;
use crate::pdf::error::{format_page_options_err, format_pdf_err};
use bernard_ledit::pdf::Document;
use bernard_ledit::pdf::page_options::PageOperation;
use pyo3::{PyResult, pyfunction};

/// Applies page options to a PDF document.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
pub fn apply_page_options(
    data: PdfInput<'_>,
    operation: &str,
    on_min_pages: usize,
    page_indexes: Vec<i32>,
) -> PyResult<(Option<PyPdfDocument>, Vec<i32>)> {
    let doc = Document::from_bytes(data.into_vec()?).map_err(|e| format_pdf_err(&e))?;
    let op = operation
        .parse::<PageOperation>()
        .map_err(|e| format_page_options_err(&e))?;
    let out =
        bernard_ledit::pdf::page_options::apply_page_options(&doc, op, on_min_pages, &page_indexes)
            .map_err(|e| format_page_options_err(&e))?;
    Ok((out.document.map(PyPdfDocument::from), out.skipped))
}
