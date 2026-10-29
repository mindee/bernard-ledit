use crate::args::value_error;
use crate::exceptions::{image_exception, pdfium_exception};
use crate::image::error::{format_image_extract_err, map_image_err};
use bernard_ledit::pdf::PdfError;
use bernard_ledit::pdf::error::{ImageError as JpegError, PdfBuilderError, PdfExtractError};
use ext_php_rs::exception::PhpException;

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
            value_error(e.to_string())
        }
        PdfError::Image(inner) => map_image_err(inner),
        PdfError::Io(_) | PdfError::DocumentCreationFailed => {
            PhpException::from_message(e.to_string())
        }
        PdfError::Pdfium(_) | PdfError::Other(_) => pdfium_exception(e.to_string()),
    }
}

/// Returns a PHP Exception for the given `PdfBuilderError`.
fn map_builder_err(e: &PdfBuilderError) -> PhpException {
    match e {
        PdfBuilderError::Pdf(inner) => map_pdf_err(inner),
        PdfBuilderError::Image(JpegError::InvalidJpeg | JpegError::UnsupportedColorSpace(_)) => {
            image_exception(e.to_string())
        }
        PdfBuilderError::Image(JpegError::AddObject) => pdfium_exception(e.to_string()),
        PdfBuilderError::PageOutOfRange(_) | PdfBuilderError::EmptyPolygon(_) => {
            value_error(e.to_string())
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
        PdfExtractError::EmptyGroup(_) | PdfExtractError::NoGroups => value_error(e.to_string()),
    }
}
