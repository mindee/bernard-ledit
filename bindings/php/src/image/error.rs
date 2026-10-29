use crate::pdf::error::map_pdf_err;
use bernard_ledit::image::ImageError;
use bernard_ledit::image::error::ImageExtractError;
use ext_php_rs::{exception::PhpException, prelude::*, zend::ce};

/// `BernardLedit\Image\ImageException extends \Exception`: input that isn't an identifiable or
/// decodable image, and encoding failures.
#[php_class]
#[php(name = "BernardLedit\\Image\\ImageException")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Default)]
pub struct ImageException;

/// Maps errors for image-related features.
#[must_use]
pub fn map_image_err(e: &ImageError) -> PhpException {
    match e {
        // Arguments the caller got wrong.
        ImageError::UnsupportedFormat(_)
        | ImageError::UnknownFilter(_)
        | ImageError::InvalidCrop { .. }
        | ImageError::InvalidDimensions { .. } => {
            PhpException::new(e.to_string(), 0, ce::value_error())
        }
        ImageError::Io(_) => PhpException::from_message(e.to_string()),
        ImageError::UnknownFormat
        | ImageError::NotEnoughData
        | ImageError::Decode(_)
        | ImageError::Encode(_)
        | ImageError::ProgressiveQualityTooLow { .. }
        | ImageError::SizeIncreased { .. } => {
            PhpException::from_class::<ImageException>(e.to_string())
        }
    }
}

/// Maps image extraction errors: wrapped errors keep the class they have everywhere else.
#[must_use]
pub fn format_image_extract_err(e: &ImageExtractError) -> PhpException {
    match e {
        ImageExtractError::Pdf(inner) => map_pdf_err(inner),
        ImageExtractError::Image(inner) => map_image_err(inner),
        ImageExtractError::PageOutOfRange(_) | ImageExtractError::EmptyPolygon(_) => {
            PhpException::new(e.to_string(), 0, ce::value_error())
        }
    }
}

/// Unknown format error.
#[must_use]
pub fn unknown_format_err() -> PhpException {
    map_image_err(&ImageError::UnknownFormat)
}
