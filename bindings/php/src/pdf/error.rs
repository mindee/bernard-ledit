use crate::image::error::{ImageException, ImageExtractException};
use bernard_ledit::pdf::PdfError;
use bernard_ledit::pdf::error::PdfExtractError;
use ext_php_rs::{exception::PhpException, prelude::*, zend::ce};

#[php_class]
#[php(name = "BernardLedit\\Pdf\\PdfiumException")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Default)]
pub struct PdfiumException;

/// Returns a PHP Exception indicating that the `PdfDocument` is closed.
#[must_use]
pub fn closed_err() -> PhpException {
    PhpException::from_message("PdfDocument is closed".into())
}

/// Returns a PHP Exception for the given `PdfError`.
#[must_use]
pub fn map_pdf_err(e: &PdfError) -> PhpException {
    match e {
        PdfError::PageCountOutOfBounds | PdfError::PageIndexOutOfBounds { .. } => {
            PhpException::new(e.to_string(), 0, ce::value_error())
        }
        PdfError::Io(_) | PdfError::DocumentCreationFailed => {
            PhpException::from_message(e.to_string())
        }
        PdfError::Pdfium(_) | PdfError::Other(_) => {
            PhpException::from_class::<PdfiumException>(e.to_string())
        }
    }
}

#[php_class]
#[php(name = "BernardLedit\\Pdf\\PdfExtractException")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Default)]
pub struct PdfExtractException;

/// Returns a PHP Exception for the given `PdfExtractError`.
#[must_use]
pub fn format_pdf_extract_err(e: &PdfExtractError) -> PhpException {
    match e {
        PdfExtractError::Pdf { .. } | PdfExtractError::Builder { .. } => {
            PhpException::from_class::<PdfExtractException>(e.to_string())
        }
        PdfExtractError::ImageExtract { .. } => {
            PhpException::from_class::<ImageExtractException>(e.to_string())
        }
        PdfExtractError::EmptyGroup { .. } | PdfExtractError::NoGroups => {
            PhpException::new(e.to_string(), 0, ce::value_error())
        }
        PdfExtractError::Image(_) => PhpException::from_class::<ImageException>(e.to_string()),
    }
}
