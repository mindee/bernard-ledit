use bernard_ledit::image::ImageError;
use ext_php_rs::{exception::PhpException, prelude::*, zend::ce};

/// `BernardLedit\Image\ImageException extends \Exception` Decode and encode failures.
#[php_class]
#[php(name = "BernardLedit\\Image\\ImageException")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Default)]
pub struct ImageException;

/// Maps errors for image-related features.
#[must_use]
pub fn map_image_err(e: &ImageError) -> PhpException {
    match e {
        ImageError::UnsupportedFormat(_)
        | ImageError::UnknownFormat
        | ImageError::NotEnoughData
        | ImageError::UnknownFilter(_)
        | ImageError::InvalidCrop { .. }
        | ImageError::InvalidDimensions { .. } => {
            PhpException::new(e.to_string(), 0, ce::value_error())
        }
        ImageError::Io(_) => PhpException::from_message(e.to_string()),
        ImageError::Decode(_) | ImageError::Encode(_) => {
            PhpException::from_class::<ImageException>(e.to_string())
        }
    }
}

/// Unknown format error.
#[must_use]
pub fn unknown_format_err() -> PhpException {
    map_image_err(&ImageError::UnknownFormat)
}
