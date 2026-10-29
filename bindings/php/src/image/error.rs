use crate::args::value_error;
use crate::exceptions::image_exception;
use crate::pdf::error::map_pdf_err;
use bernard_ledit::image::ImageError;
use bernard_ledit::image::error::ImageExtractError;
use ext_php_rs::exception::PhpException;

/// Maps errors for image-related features.
#[must_use]
pub fn map_image_err(e: &ImageError) -> PhpException {
    match e {
        // Arguments the caller got wrong.
        ImageError::UnsupportedFormat(_)
        | ImageError::UnknownFilter(_)
        | ImageError::InvalidCrop { .. }
        | ImageError::InvalidDimensions { .. } => value_error(e.to_string()),
        ImageError::Io(_) => PhpException::from_message(e.to_string()),
        ImageError::UnknownFormat
        | ImageError::NotEnoughData
        | ImageError::Decode(_)
        | ImageError::Encode(_)
        | ImageError::ProgressiveQualityTooLow { .. }
        | ImageError::SizeIncreased { .. } => image_exception(e.to_string()),
    }
}

/// Maps image extraction errors: wrapped errors keep the class they have everywhere else.
#[must_use]
pub fn format_image_extract_err(e: &ImageExtractError) -> PhpException {
    match e {
        ImageExtractError::Pdf(inner) => map_pdf_err(inner),
        ImageExtractError::Image(inner) => map_image_err(inner),
        ImageExtractError::PageOutOfRange(_) | ImageExtractError::EmptyPolygon(_) => {
            value_error(e.to_string())
        }
    }
}

/// Unknown format error.
#[must_use]
pub fn unknown_format_err() -> PhpException {
    map_image_err(&ImageError::UnknownFormat)
}
