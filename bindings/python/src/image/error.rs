use crate::pdf::error::PyPdfiumError;
use bernard_ledit::image::ImageError;
use bernard_ledit::image::error::ExtractError;
use pyo3::exceptions::PyException;
use pyo3::{PyErr, create_exception};

create_exception!(
    bernard_ledit.image,
    PyImageError,
    PyException,
    "This error represents an image processing failure."
);

/// Map a core [`ImageError`] onto the most appropriate Python exception.
#[must_use = "ImageError is formatted into a PyException"]
pub fn format_image_err(e: &ImageError) -> PyErr {
    match e {
        ImageError::UnsupportedFormat(_)
        | ImageError::UnknownFormat
        | ImageError::NotEnoughData
        | ImageError::UnknownFilter(_)
        | ImageError::InvalidCrop { .. }
        | ImageError::InvalidDimensions { .. }
        | ImageError::ProgressiveQualityTooLow { .. } => {
            pyo3::exceptions::PyValueError::new_err(e.to_string())
        }
        ImageError::Io(_) => pyo3::exceptions::PyIOError::new_err(e.to_string()),
        ImageError::Decode(_) | ImageError::Encode(_) | ImageError::SizeIncreased { .. } => {
            PyImageError::new_err(e.to_string())
        }
    }
}

create_exception!(
    bernard_ledit.image,
    PyExtractError,
    PyException,
    "This error represents an extraction failure."
);

/// Map a core [`ExtractError`] onto the most appropriate Python exception.
#[must_use]
pub fn format_extract_err(e: &ExtractError) -> PyErr {
    match e {
        ExtractError::Image(..) => PyImageError::new_err(e.to_string()),
        ExtractError::PageOutOfRange(..) | ExtractError::EmptyPolygon(..) => {
            PyExtractError::new_err(e.to_string())
        }
        ExtractError::Pdf(..) => PyPdfiumError::new_err(e.to_string()),
    }
}
