use bernard_ledit::pdf::Bitmap;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// A bitmap image.
#[pyclass(name = "PdfBitmap", module = "bernard_ledit.pdf")]
pub struct PyPdfBitmap {
    pub(crate) inner: Bitmap,
}

#[pymethods]
impl PyPdfBitmap {
    /// Width in pixels.
    #[getter]
    const fn width(&self) -> u32 {
        self.inner.width
    }

    /// Height in pixels.
    #[getter]
    const fn height(&self) -> u32 {
        self.inner.height
    }

    /// Raw RGBA8 pixel buffer (row-major, 4 bytes per pixel).
    /// Suitable for direct numpy consumption: `np.frombuffer(bmp.to_bytes(), dtype=np.uint8).reshape(h, w, 4)`.
    fn to_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.rgba)
    }

    /// Encode the bitmap as a PNG byte stream.
    /// Does not require Pillow; wrap with `io.BytesIO` if PIL interop is needed.
    fn to_png_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .to_png_bytes()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes))
    }

    fn __repr__(&self) -> String {
        format!("PdfBitmap({}x{})", self.inner.width, self.inner.height)
    }
}

impl From<Bitmap> for PyPdfBitmap {
    fn from(inner: Bitmap) -> Self {
        Self { inner }
    }
}
