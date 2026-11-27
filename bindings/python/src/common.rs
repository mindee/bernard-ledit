use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// Register the `common` submodule.
/// # Errors
/// Returns a Python error if the submodule cannot be registered.
pub fn register_submodule(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let module = PyModule::new(parent.py(), "common")?;
    module.add_function(wrap_pyfunction!(format_from_extension, &module)?)?;
    parent.add_submodule(&module)?;
    Ok(())
}

/// Returns the canonical encoding name for a file extension.
#[pyfunction]
fn format_from_extension(extension: &str) -> String {
    bernard_ledit::common::format_from_extension(extension)
}
