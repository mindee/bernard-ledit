use crate::image::error::{PyImageExtractError, format_image_err};
use bernard_ledit::pdf::PdfError;
use bernard_ledit::pdf::error::PdfExtractError;
use bernard_ledit::pdf::page_options::PageOptionsError;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::{PyErr, create_exception};

create_exception!(
    bernard_ledit.pdf,
    PyPdfiumError,
    PyException,
    "This error represents a PDF processing failure in PDFium."
);

create_exception!(
    bernard_ledit.pdf,
    PyPdfExtractionError,
    PyValueError,
    "This error represents a PDF extraction failure."
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
        PdfError::Pdfium(_) | PdfError::Other(_) | PdfError::Image(_) => {
            PyPdfiumError::new_err(e.to_string())
        }
    }
}

/// Formats a `PdfExtractError` as a `PyException`
#[must_use = "PdfExtractError is formatted into a PyException"]
pub fn format_pdf_extract_err(e: &PdfExtractError) -> PyErr {
    match e {
        PdfExtractError::Pdf(inner) => format_pdf_err(inner),
        PdfExtractError::Image(inner) => format_image_err(inner),
        PdfExtractError::ImageExtract(_) => PyImageExtractError::new_err(e.to_string()),
        PdfExtractError::Builder(_) => PyPdfiumError::new_err(e.to_string()),
        PdfExtractError::EmptyGroup(_) | PdfExtractError::NoGroups => {
            PyPdfExtractionError::new_err(e.to_string())
        }
    }
}

create_exception!(
    bernard_ledit.pdf,
    PyPdfFixError,
    PyValueError,
    "PDF could not be fixed."
);
create_exception!(
    bernard_ledit.pdf,
    PyPageOptionsError,
    PyValueError,
    "Invalid page options."
);

/// Formats a `PageOptionsError` as a `PyException`
#[must_use = "PageOptionsError is formatted into a PyException"]
pub fn format_page_options_err(e: &PageOptionsError) -> PyErr {
    match e {
        PageOptionsError::Pdf(inner) => format_pdf_err(inner),
        _ => PyPageOptionsError::new_err(e.to_string()),
    }
}
