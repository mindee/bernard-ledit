use crate::pdf::error::PyPdfiumError;
use bernard_ledit::image::ImageError;
use bernard_ledit::image::error::ImageExtractError;
use pyo3::exceptions::PyValueError;
use pyo3::{PyErr, create_exception};

create_exception!(
    bernard_ledit.image,
    PyImageError,
    PyValueError,
    "This error represents an image processing failure."
);

create_exception!(
    bernard_ledit.image,
    PyImageExtractError,
    PyValueError,
    "This error represents an extraction failure."
);

/// Map a core [`ImageError`] onto the most appropriate Python exception.
#[must_use = "ImageError is formatted into a PyException"]
pub fn format_image_err(e: &ImageError) -> PyErr {
    match e {
        ImageError::Io(_) => pyo3::exceptions::PyIOError::new_err(e.to_string()),
        ImageError::UnsupportedFormat(_)
        | ImageError::UnknownFormat
        | ImageError::NotEnoughData
        | ImageError::UnknownFilter(_)
        | ImageError::InvalidCrop { .. }
        | ImageError::InvalidDimensions { .. }
        | ImageError::ProgressiveQualityTooLow { .. }
        | ImageError::Decode(_)
        | ImageError::Encode(_)
        | ImageError::SizeIncreased { .. } => PyImageError::new_err(e.to_string()),
    }
}

/// Map a core [`ImageExtractError`] onto the most appropriate Python exception.
#[must_use]
pub fn format_image_extract_err(e: &ImageExtractError) -> PyErr {
    match e {
        ImageExtractError::Image(inner) => format_image_err(inner),
        ImageExtractError::PageOutOfRange(..) | ImageExtractError::EmptyPolygon(..) => {
            PyImageExtractError::new_err(e.to_string())
        }
        ImageExtractError::Pdf(..) => PyPdfiumError::new_err(e.to_string()),
    }
}
