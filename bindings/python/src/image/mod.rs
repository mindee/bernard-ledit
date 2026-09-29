use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// Image errors.
pub mod error;
/// Raster image.
pub mod image_data;

/// Register the `image` submodule.
/// # Errors
/// Returns a Python error if the submodule cannot be registered.
pub fn register_submodule(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new(parent.py(), "image")?;
    m.add_class::<image_data::PyImage>()?;
    m.add_function(wrap_pyfunction!(image_data::decode, &m)?)?;
    m.add_function(wrap_pyfunction!(image_data::guess_format, &m)?)?;
    m.add_function(wrap_pyfunction!(image_data::compress, &m)?)?;
    m.add("ImageError", parent.py().get_type::<error::PyImageError>())?;
    parent.add_submodule(&m)?;
    Ok(())
}
