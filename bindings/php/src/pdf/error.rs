use bernard_ledit::pdf::PdfError;
use ext_php_rs::{exception::PhpException, prelude::*, zend::ce};

#[php_class]
#[php(name = "BernardLedit\\Pdf\\PdfiumException")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Default)]
pub struct PdfiumException;

/// Returns a PHP Exception indicating that the `PdfDocument` is closed.
#[must_use]
pub fn closed_err() -> PhpException {
    PhpException::default("PdfDocument is closed".into())
}

/// Returns a PHP Exception for the given `PdfError`.
#[must_use]
pub fn map_pdf_err(e: &PdfError) -> PhpException {
    match e {
        PdfError::PageCountOutOfBounds | PdfError::PageIndexOutOfBounds { .. } => {
            PhpException::new(e.to_string(), 0, ce::value_error()) // PHP has no IndexError; ValueError is idiomatic
        }
        PdfError::Io(_) | PdfError::DocumentCreationFailed => PhpException::default(e.to_string()),
        PdfError::Pdfium(_) | PdfError::Other(_) => {
            PhpException::from_class::<PdfiumException>(e.to_string())
        }
    }
}
