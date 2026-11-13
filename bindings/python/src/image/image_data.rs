use super::error::format_image_err;
use crate::input::BinaryInput;
use bernard_ledit::image::{
    FilterType, Image, format_name, guess_format as core_guess, parse_output_format,
};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyclass(name = "Image", module = "bernard_ledit.image")]
/// An image.
pub struct PyImage {
    /// Core image.
    inner: Image,
}

impl PyImage {
    /// Create from core Image.
    pub(crate) const fn from_core(inner: Image) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyImage {
    #[new]
    #[pyo3(signature = (data))]
    fn new(data: BinaryInput<'_>) -> PyResult<Self> {
        let inner = Image::decode(&data.into_vec()?).map_err(|e| format_image_err(&e))?;
        Ok(Self { inner })
    }

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

    /// Crops the bounding box of normalized polygon points from this image.
    fn crop_polygon(&self, polygon: Vec<(f32, f32)>) -> PyResult<Self> {
        self.inner
            .crop_polygon(&polygon)
            .map(Self::from_core)
            .map_err(|e| super::error::format_image_extract_err(&e))
    }

    #[pyo3(signature = (width, height))]
    fn resize(&self, width: u32, height: u32) -> PyResult<Self> {
        let resized_image = self
            .inner
            .resize(width, height, FilterType::Lanczos3)
            .map(|inner| Self { inner })
            .map_err(|e| format_image_err(&e))?;
        Ok(resized_image)
    }

    #[pyo3(signature = (format, jpeg_quality = 85, jpeg_optimize = true))]
    fn encode<'py>(
        &self,
        py: Python<'py>,
        format: &str,
        jpeg_quality: u8,
        jpeg_optimize: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let parsed_format = parse_output_format(format).map_err(|e| format_image_err(&e))?;
        let bytes_vec_u8 = self
            .inner
            .encode(parsed_format, jpeg_quality, jpeg_optimize)
            .map_err(|e| format_image_err(&e))?;
        Ok(PyBytes::new(py, bytes_vec_u8.as_slice()))
    }

    fn __repr__(&self) -> String {
        let (w, h) = self.inner.size();
        let fmt = self.inner.format().and_then(format_name).unwrap_or("?");
        format!("Image({w}x{h}, format:{fmt})")
    }

    #[pyo3(signature = (quality = 85, optimize = true, max_width = None, max_height = None))]
    fn compress_to_jpeg<'py>(
        &self,
        py: Python<'py>,
        quality: u8,
        optimize: bool,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let (bytes, _, _) = self
            .inner
            .compress_to_jpeg(quality, optimize, max_width, max_height)
            .map_err(|e| format_image_err(&e))?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Retrieve the bytes from the image.
    fn as_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let owned_bytes = self.inner.to_bytes().map_err(|e| format_image_err(&e))?;
        Ok(PyBytes::new(py, owned_bytes.as_slice()))
    }
}

/// Guesses the format of the given bytes.
/// # Errors
/// Returns an error if the image data is invalid.
#[pyfunction]
pub fn guess_format(data: &Bound<'_, PyAny>) -> PyResult<&'static str> {
    let bytes: Vec<u8> = if data.hasattr("read")? {
        data.call_method0("read")?.extract()?
    } else {
        data.extract()?
    };
    let fmt = core_guess(&bytes).map_err(|e| format_image_err(&e))?;
    Ok(format_name(fmt).unwrap_or("UNKNOWN"))
}
