//! Python bindings for the polygon class.
#![allow(missing_docs)]

use crate::geometry::point::PyPoint;
use bernard_ledit::geometry::point::Point;
use bernard_ledit::geometry::polygon::Polygon;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::PyAnyMethods;
use pyo3::types::PySlice;
use pyo3::types::PySliceMethods;
use pyo3::{Bound, FromPyObject, IntoPyObjectExt, Py, PyAny, PyRef, PyResult, pyclass, pymethods};

#[pyclass(name = "Polygon", module = "bernard_ledit.geometry")]
pub struct PyPolygon(Polygon);

#[derive(FromPyObject)]
pub enum PointInput<'a> {
    Point(PyRef<'a, PyPoint>),
    Tuple((f64, f64)),
}

#[pymethods]
impl PyPolygon {
    #[new]
    fn new(points: Vec<PointInput>) -> Self {
        let core_polygon = points.into_iter().map(|input| match input {
            PointInput::Point(p) => p.0,
            PointInput::Tuple((x, y)) => Point::new(x, y),
        });

        Self(Polygon::new(core_polygon))
    }
    /// Maps to `Display` `through to_string_representation()`
    #[pyo3(name = "__str__")]
    fn str_method(&self) -> String {
        self.0.to_string_representation()
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
    fn getitem_method(&self, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py = key.py();
        if let Ok(index) = key.extract::<isize>() {
            let len = self.0.0.len();
            let actual_index: Option<usize> = if index < 0 {
                len.checked_sub(index.unsigned_abs())
            } else {
                usize::try_from(index).ok()
            };
            let point = actual_index
                .and_then(|i| self.0.0.get(i))
                .copied()
                .map(PyPoint)
                .ok_or_else(|| {
                    pyo3::exceptions::PyIndexError::new_err("polygon index out of range")
                });
            return point?.into_py_any(py);
        }

        if let Ok(slice) = key.cast::<PySlice>() {
            let len = self.0.0.len().try_into().unwrap();
            let indices = slice.indices(len)?;

            let points_iter =
                std::iter::successors(Some(indices.start), |&i| Some(i + indices.step))
                    .take(indices.slicelength)
                    .map(|i| {
                        #[allow(clippy::cast_sign_loss)]
                        let idx = i as usize;
                        self.0.0[idx]
                    });
            return Self(Polygon::new(points_iter)).into_py_any(py);
        }

        Err(PyTypeError::new_err("indices must be integers or slices"))
    }

    // Length
    #[pyo3(name = "__len__")]
    fn len_method(&self) -> usize {
        self.0.0.len()
    }

    // Centroid method
    fn centroid(&self) -> Option<PyPoint> {
        self.0.centroid().map(PyPoint)
    }
}
