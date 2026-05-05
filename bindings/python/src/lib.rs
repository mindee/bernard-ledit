//! Python bindings for the Bernard l'Édit library.

mod geometry;

use pyo3::prelude::PyModule;
use pyo3::{Bound, PyResult, pymodule};

#[pymodule]
fn _bernard_ledit(m: &Bound<'_, PyModule>) -> PyResult<()> {
    geometry::register_submodule(m)?;
    Ok(())
}
