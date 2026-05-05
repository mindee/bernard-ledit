//! Python bindings for the Bernard l'Édit library.

#![allow(missing_docs)]

use bernard_ledit::geometry::point::Point;
use pyo3::prelude::*;

#[pyclass(name = "Point", module = "bernard_ledit.geometry")]
pub struct PyPoint(pub Point);

#[pymethods]
impl PyPoint {
    #[new]
    const fn new(x: f64, y: f64) -> Self {
        Self(Point::new(x, y))
    }

    #[getter]
    const fn x(&self) -> f64 {
        self.0.x
    }

    #[getter]
    const fn y(&self) -> f64 {
        self.0.y
    }

    #[setter]
    const fn set_x(&mut self, value: f64) {
        self.0.x = value;
    }

    #[setter]
    const fn set_y(&mut self, value: f64) {
        self.0.y = value;
    }

    /// Maps to `Display`
    #[pyo3(name = "__str__")]
    fn str_method(&self) -> String {
        format!("{}", self.0)
    }

    /// Maps to `Debug`
    #[pyo3(name = "__repr__")]
    fn repr_method(&self) -> String {
        format!("{:?}", self.0)
    }

    /// Maps to `PartialEq`
    #[pyo3(name = "__eq__")]
    fn eq_method(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    /// Maps to `Index`
    #[pyo3(name = "__getitem__")]
    fn getitem_method(&self, index: usize) -> f64 {
        self.0[index]
    }
}
