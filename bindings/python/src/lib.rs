//! Python bindings for the Quilldee library.

mod geometry;

use pyo3::prelude::PyModule;
use pyo3::{Bound, PyResult, pymodule};

#[pymodule]
fn _quilldee(m: &Bound<'_, PyModule>) -> PyResult<()> {
    geometry::register_submodule(m)?;
    Ok(())
}
