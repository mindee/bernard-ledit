use pyo3::prelude::{PyAnyMethods, PyBytesMethods};
use pyo3::types::PyBytes;
use pyo3::{Bound, FromPyObject, PyAny, PyResult};
use std::path::PathBuf;

/// Wrapper for either `bytes` or a binary file-like object.
#[derive(FromPyObject)]
pub enum BinaryInput<'py> {
    /// Raw Bytes.
    Bytes(Bound<'py, PyBytes>),
    /// A Python file-like object (must have a `.read()` method).
    FileLike(Bound<'py, PyAny>),
}

impl BinaryInput<'_> {
    pub fn into_vec(self) -> PyResult<Vec<u8>> {
        match self {
            Self::Bytes(b) => Ok(b.as_bytes().to_vec()),
            Self::FileLike(f) => f.call_method0("read")?.extract(),
        }
    }
}

/// PDF input wrapper.
#[derive(FromPyObject)]
pub enum PdfInput<'py> {
    /// Raw bytes.
    Bytes(Bound<'py, PyBytes>),
    /// Path to a file on disk.
    Path(PathBuf),
    /// A Python file-like object.
    FileLike(Bound<'py, PyAny>),
}

impl PdfInput<'_> {
    pub fn into_vec(self) -> PyResult<Vec<u8>> {
        match self {
            Self::Bytes(b) => BinaryInput::Bytes(b).into_vec(),
            Self::Path(p) => std::fs::read(&p).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!("Failed to read {}: {e}", p.display()))
            }),
            Self::FileLike(f) => BinaryInput::FileLike(f).into_vec(),
        }
    }
}
