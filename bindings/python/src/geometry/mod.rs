use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult};

pub mod point;
pub mod polygon;
use crate::geometry::polygon::PyPolygon;
use point::PyPoint;

pub fn register_submodule(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let child_module = PyModule::new(parent_module.py(), "geometry")?;

    child_module.add_class::<PyPoint>()?;
    child_module.add_class::<PyPolygon>()?;

    parent_module.add_submodule(&child_module)?;

    Ok(())
}
