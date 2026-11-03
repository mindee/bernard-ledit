//! The one error type every function returns, mapped from core's errors like the PHP binding does.

use bernard_ledit::image::ImageError;
use bernard_ledit::image::error::ImageExtractError;
use bernard_ledit::pdf::PdfError;
use bernard_ledit::pdf::error::{ImageError as JpegError, PdfBuilderError, PdfExtractError};
use bernard_ledit::pdf::fix::PdfFixError;
use bernard_ledit::pdf::page_options::PageOptionsError;

/// Errors raised by bernard-ledit. Each language names it after its conventions:
/// `BernardLeditException` in Kotlin/Java and .NET, `BernardLeditError` in Ruby and TypeScript.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum BernardLeditError {
    /// An argument is out of range or otherwise invalid: PHP's `\ValueError`.
    #[error("{0}")]
    InvalidArgument(String),
    /// Input that isn't an identifiable or decodable image, or an encoding failure.
    #[error("{0}")]
    Image(String),
    /// `PDFium` can't be loaded, or fails to read, render or write a document.
    #[error("{0}")]
    Pdfium(String),
    /// Anything else, such as I/O errors.
    #[error("{0}")]
    Other(String),
}

impl From<ImageError> for BernardLeditError {
    fn from(e: ImageError) -> Self {
        let message = e.to_string();
        match e {
            ImageError::UnsupportedFormat(_)
            | ImageError::UnknownFilter(_)
            | ImageError::InvalidCrop { .. }
            | ImageError::InvalidDimensions { .. } => Self::InvalidArgument(message),
            ImageError::Io(_) => Self::Other(message),
            ImageError::UnknownFormat
            | ImageError::NotEnoughData
            | ImageError::Decode(_)
            | ImageError::Encode(_)
            | ImageError::ProgressiveQualityTooLow { .. }
            | ImageError::SizeIncreased { .. } => Self::Image(message),
        }
    }
}

impl From<PdfError> for BernardLeditError {
    fn from(e: PdfError) -> Self {
        let message = e.to_string();
        match e {
            PdfError::PageCountOutOfBounds | PdfError::PageIndexOutOfBounds { .. } => {
                Self::InvalidArgument(message)
            }
            PdfError::Image(inner) => inner.into(),
            PdfError::Io(_) | PdfError::DocumentCreationFailed => Self::Other(message),
            PdfError::Pdfium(_) | PdfError::Other(_) => Self::Pdfium(message),
        }
    }
}

impl From<PdfBuilderError> for BernardLeditError {
    fn from(e: PdfBuilderError) -> Self {
        let message = e.to_string();
        match e {
            PdfBuilderError::Pdf(inner) => inner.into(),
            PdfBuilderError::Image(
                JpegError::InvalidJpeg | JpegError::UnsupportedColorSpace(_),
            ) => Self::Image(message),
            PdfBuilderError::Image(JpegError::AddObject) => Self::Pdfium(message),
            PdfBuilderError::PageOutOfRange(_) | PdfBuilderError::EmptyPolygon(_) => {
                Self::InvalidArgument(message)
            }
        }
    }
}

impl From<ImageExtractError> for BernardLeditError {
    fn from(e: ImageExtractError) -> Self {
        let message = e.to_string();
        match e {
            ImageExtractError::Pdf(inner) => inner.into(),
            ImageExtractError::Image(inner) => inner.into(),
            ImageExtractError::PageOutOfRange(_) | ImageExtractError::EmptyPolygon(_) => {
                Self::InvalidArgument(message)
            }
        }
    }
}

impl From<PdfExtractError> for BernardLeditError {
    fn from(e: PdfExtractError) -> Self {
        let message = e.to_string();
        match e {
            PdfExtractError::Pdf(inner) => inner.into(),
            PdfExtractError::Builder(inner) => inner.into(),
            PdfExtractError::Image(inner) => inner.into(),
            PdfExtractError::ImageExtract(inner) => inner.into(),
            PdfExtractError::EmptyGroup(_) | PdfExtractError::NoGroups => {
                Self::InvalidArgument(message)
            }
        }
    }
}

impl From<PageOptionsError> for BernardLeditError {
    fn from(e: PageOptionsError) -> Self {
        match e {
            PageOptionsError::Pdf(inner) => inner.into(),
            other => Self::InvalidArgument(other.to_string()),
        }
    }
}

impl From<PdfFixError> for BernardLeditError {
    fn from(e: PdfFixError) -> Self {
        Self::InvalidArgument(e.to_string())
    }
}
