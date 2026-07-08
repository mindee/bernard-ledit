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

    #[pyo3(signature = (format, quality = 85, optimize = true))]
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

    /// Saves the image to a path-like or a writable buffer.
    ///
    /// `format` is inferred from the file extension when saving to a path, or
    /// from the image's own format when saving to a buffer. Pass an explicit
    /// `format` string (e.g. `"JPEG"`, `"PNG"`, `"PDF"`) to override.
    #[pyo3(signature = (dest, format = None, quality = 85, optimize = true))]
    fn save(
        &self,
        py: Python<'_>,
        dest: &Bound<'_, PyAny>,
        format: Option<&str>,
        quality: u8,
        optimize: bool,
    ) -> PyResult<()> {
        let is_buffer = dest.hasattr("write")?;

        let fmt_str: String = if let Some(f) = format {
            f.to_string()
        } else if is_buffer {
            self.inner
                .format()
                .and_then(format_name)
                .unwrap_or("JPEG")
                .to_string()
        } else {
            let os = py.import("os")?;
            let path_str: String = os.call_method1("fspath", (dest,))?.extract()?;
            let ext = std::path::Path::new(&path_str)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg")
                .to_uppercase();
            match ext.as_str() {
                "JPG" | "JPEG" => "JPEG".to_string(),
                "TIF" => "TIFF".to_string(),
                "EXR" => "OPENEXR".to_string(),
                _ => ext,
            }
        };

        let parsed_format = parse_output_format(&fmt_str).map_err(|e| format_image_err(&e))?;
        let encoded = self
            .inner
            .encode(parsed_format, quality, optimize)
            .map_err(|e| format_image_err(&e))?;

        if is_buffer {
            dest.call_method1("write", (PyBytes::new(py, &encoded),))?;
        } else {
            let os = py.import("os")?;
            let path_str: String = os.call_method1("fspath", (dest,))?.extract()?;
            std::fs::write(&path_str, &encoded)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        }
        Ok(())
    }

    fn __repr__(&self) -> String {
        let (w, h) = self.inner.size();
        let fmt = self.inner.format().and_then(format_name).unwrap_or("?");
        format!("Image({w}x{h}, format:{fmt})")
    }
}

/// Decodes raw bytes into an image.
/// Accepts `bytes`, `bytearray`, or any file-like object with a `read()` method (e.g. `BytesIO`).
/// # Errors
/// Returns an error if the image data is invalid.
#[pyfunction]
pub fn decode(data: &Bound<'_, PyAny>) -> PyResult<PyImage> {
    let bytes: Vec<u8> = if data.hasattr("read")? {
        data.call_method0("read")?.extract()?
    } else {
        data.extract()?
    };
    Ok(PyImage {
        inner: Image::decode(&bytes).map_err(|e| format_image_err(&e))?,
    })
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
