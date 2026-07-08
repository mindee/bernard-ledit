use super::error::format_image_err;
use bernard_ledit::image::{
    Filter, Image, compress as core_compress, format_name, guess_format as core_guess,
    parse_output_format,
};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::str::FromStr;

#[pyclass(name = "Image", module = "bernard_ledit.image")]
/// An image.
pub struct PyImage {
    inner: Image,
}

#[pymethods]
impl PyImage {
    /// (width, height)
    #[getter]
    fn size(&self) -> (u32, u32) {
        self.inner.size()
    }

    /// Uppercase format string, or None if unknown.
    #[getter]
    fn format(&self) -> Option<&'static str> {
        self.inner.format().and_then(format_name)
    }

    fn crop(&self, left: u32, top: u32, right: u32, bottom: u32) -> PyResult<Self> {
        let cropped_image = self
            .inner
            .crop(left, top, right, bottom)
            .map(|inner| Self { inner })
            .map_err(|e| format_image_err(&e))?;
        Ok(cropped_image)
    }

    #[pyo3(signature = (width, height, filter = "lanczos"))]
    fn resize(&self, width: u32, height: u32, filter: &str) -> PyResult<Self> {
        let filter_value = Filter::from_str(filter).map_err(|e| format_image_err(&e))?;
        let resized_image = self
            .inner
            .resize(width, height, filter_value.0)
            .map(|inner| Self { inner })
            .map_err(|e| format_image_err(&e))?;
        Ok(resized_image)
    }

    #[pyo3(signature = (format, quality = 85, optimize = false))]
    fn encode<'py>(
        &self,
        py: Python<'py>,
        format: &str,
        quality: u8,
        optimize: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let parsed_format = parse_output_format(format).map_err(|e| format_image_err(&e))?;
        let bytes_vec_u8 = self
            .inner
            .encode(parsed_format, quality, optimize)
            .map_err(|e| format_image_err(&e))?;
        Ok(PyBytes::new(py, bytes_vec_u8.as_slice()))
    }

    fn __repr__(&self) -> String {
        let (w, h) = self.inner.size();
        let fmt = self.inner.format().and_then(format_name).unwrap_or("?");
        format!("Image({w}x{h}, format:{fmt})")
    }
}

/// Decodes raw bytes into an image.
/// # Errors
/// Returns an error if the image data is invalid.
#[pyfunction]
pub fn decode(data: &[u8]) -> PyResult<PyImage> {
    Ok(PyImage {
        inner: Image::decode(data).map_err(|e| format_image_err(&e))?,
    })
}

/// Guesses the format of the given bytes.
/// # Errors
/// Returns an error if the image data is invalid.
#[pyfunction]
pub fn guess_format(data: &[u8]) -> PyResult<&'static str> {
    let fmt = core_guess(data).map_err(|e| format_image_err(&e))?;
    Ok(format_name(fmt).unwrap_or("UNKNOWN"))
}

/// Compresses a given image.
/// # Errors
/// Returns an error if the image data is invalid or if the operation fails.
#[pyfunction]
#[pyo3(signature = (data, quality = 85, max_width = None, max_height = None))]
pub fn compress<'py>(
    py: Python<'py>,
    data: &[u8],
    quality: u8,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> PyResult<(Bound<'py, PyBytes>, u32, u32)> {
    let (data, width, height) =
        core_compress(data, quality, max_width, max_height).map_err(|e| format_image_err(&e))?;
    Ok((PyBytes::new(py, data.as_slice()), width, height))
}
