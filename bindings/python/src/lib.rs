//! Python bindings for the Bernard l'Édit library.

/// Geometry-related functionality.
mod geometry;
/// PDF-related functionality.
pub mod pdf;

use pyo3::prelude::PyModule;
use pyo3::{Bound, PyResult, pymodule};

/// Register the Python submodules for the Bernard l'Édit library.
#[pymodule]
fn _bernard_ledit(m: &Bound<'_, PyModule>) -> PyResult<()> {
    geometry::register_submodule(m)?;
    pdf::register_submodule(m)?;
    Ok(())
}
