use crate::image::error::{ImageException, format_image_extract_err, map_image_err};
use bernard_ledit::pdf::PdfError;
use bernard_ledit::pdf::error::{ImageError as JpegError, PdfBuilderError, PdfExtractError};
use ext_php_rs::{exception::PhpException, prelude::*, zend::ce};

/// `BernardLedit\Pdf\PdfiumException extends \Exception`: `PDFium` can't be loaded, or fails to
/// read, render or write a document.
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
        PdfError::Image(inner) => map_image_err(inner),
        PdfError::Io(_) | PdfError::DocumentCreationFailed => {
            PhpException::from_message(e.to_string())
        }
        PdfError::Pdfium(_) | PdfError::Other(_) => {
            PhpException::from_class::<PdfiumException>(e.to_string())
        }
    }
}

/// Returns a PHP Exception for the given `PdfBuilderError`.
fn map_builder_err(e: &PdfBuilderError) -> PhpException {
    match e {
        PdfBuilderError::Pdf(inner) => map_pdf_err(inner),
        PdfBuilderError::Image(JpegError::InvalidJpeg | JpegError::UnsupportedColorSpace(_)) => {
            PhpException::from_class::<ImageException>(e.to_string())
        }
        PdfBuilderError::Image(JpegError::AddObject) => {
            PhpException::from_class::<PdfiumException>(e.to_string())
        }
        PdfBuilderError::PageOutOfRange(_) | PdfBuilderError::EmptyPolygon(_) => {
            PhpException::new(e.to_string(), 0, ce::value_error())
        }
    }
}

/// Returns a PHP Exception for the given `PdfExtractError`: wrapped errors keep the class they
/// have everywhere else.
#[must_use]
pub fn format_pdf_extract_err(e: &PdfExtractError) -> PhpException {
    match e {
        PdfExtractError::Pdf(inner) => map_pdf_err(inner),
        PdfExtractError::Builder(inner) => map_builder_err(inner),
        PdfExtractError::Image(inner) => map_image_err(inner),
        PdfExtractError::ImageExtract(inner) => format_image_extract_err(inner),
        PdfExtractError::EmptyGroup(_) | PdfExtractError::NoGroups => {
            PhpException::new(e.to_string(), 0, ce::value_error())
        }
    }
}
