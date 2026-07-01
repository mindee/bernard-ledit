use bernard_ledit::pdf::PdfError;
use pyo3::exceptions::PyException;
use pyo3::{PyErr, create_exception};

create_exception!(
    bernard_ledit.pdf,
    PyPdfiumError,
    PyException,
    "This error represents a PDF processing failure in PDFium."
);

/// Returns a closed `PdfDocument` error
#[must_use = "PdfDocument is closed"]
pub fn closed_err() -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err("PdfDocument is closed")
}

/// Returns a `PdfError` as a `PyException`
#[must_use = "PdfError is formatted into a PyException"]
pub fn format_pdf_err(e: &PdfError) -> PyErr {
    match e {
        PdfError::PageCountOutOfBounds | PdfError::PageIndexOutOfBounds { .. } => {
            pyo3::exceptions::PyIndexError::new_err(e.to_string())
        }
        PdfError::Io(_) => pyo3::exceptions::PyIOError::new_err(e.to_string()),
        PdfError::DocumentCreationFailed => {
            pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
        }
        PdfError::Pdfium(_) | PdfError::Other(_) => PyPdfiumError::new_err(e.to_string()),
    }
}
