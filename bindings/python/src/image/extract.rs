use crate::image::error::format_image_extract_err;
use crate::image::image_data::PyImage;
use pyo3::{PyRef, PyResult, pyfunction};

/// Extracts images from an `Image` based on specified polygons.
#[pyfunction]
#[pyo3(signature = (image, polygons))]
#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub fn extract_from_polygons(
    image: PyRef<'_, PyImage>,
    polygons: Vec<Vec<(f32, f32)>>,
) -> PyResult<Vec<PyImage>> {
    bernard_ledit::image::extract::extract_from_polygons(image.as_core(), &polygons)
        .map(|images| images.into_iter().map(PyImage::from_core).collect())
        .map_err(|e| format_image_extract_err(&e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bernard_ledit::image::Image;
    use pyo3::Py;
    use pyo3::Python;

    #[test]
    fn test_extract_from_polygons_empty_polygon_errors() {
        Python::initialize();

        Python::attach(|py| {
            let image = Image::decode(include_bytes!(
                "../../../../tests/data/file_types/receipt.jpg"
            ))
            .unwrap();
            let py_image = Py::new(py, PyImage::from_core(image)).unwrap();
            let image_ref = py_image.borrow(py);
            let result = extract_from_polygons(image_ref, vec![vec![]]);

            assert!(result.is_err());
        });
    }
}
