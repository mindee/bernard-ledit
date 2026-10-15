use crate::image::error::format_image_extract_err;
use crate::image::image_data::PyImage;
use crate::input::BinaryInput;
use pyo3::{PyResult, pyfunction};

/// Extracts images from a PDF or image file based on specified polygons.
#[pyfunction]
#[pyo3(signature = (data, page_id, polygons, dpi = 72.0))]
#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub fn extract_from_polygons(
    data: BinaryInput<'_>,
    page_id: u16,
    polygons: Vec<Vec<(f32, f32)>>,
    dpi: f32,
) -> PyResult<Vec<PyImage>> {
    let bytes = data.into_vec()?;
    bernard_ledit::image::extract::extract_from_polygons(&bytes, page_id, &polygons, dpi)
        .map(|v| v.into_iter().map(PyImage::from_core).collect())
        .map_err(|e| format_image_extract_err(&e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;
    use pyo3::types::PyBytes;

    #[test]
    fn test_extract_from_polygons_invalid_data_returns_err() {
        Python::initialize();

        Python::attach(|py| {
            let invalid_data = b"definitely not a valid pdf or image";

            let py_bytes = PyBytes::new(py, invalid_data);

            let input = BinaryInput::Bytes(py_bytes);

            let polygons = vec![vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]];

            let result = extract_from_polygons(input, 0, polygons, 72.0);

            assert!(
                result.is_err(),
                "Expected an error due to invalid file bytes"
            );
        });
    }

    #[test]
    fn test_extract_from_polygons_empty_polygons_list() {
        Python::initialize();

        Python::attach(|py| {
            let dummy_pdf_header = b"%PDF-1.4\n%EOF";
            let py_bytes = PyBytes::new(py, dummy_pdf_header);

            let input = BinaryInput::Bytes(py_bytes);
            let empty_polygons: Vec<Vec<(f32, f32)>> = vec![];

            let result = extract_from_polygons(input, 1, empty_polygons, 300.0);

            match result {
                Ok(images) => assert!(images.is_empty(), "Expected 0 images for 0 polygons"),
                Err(e) => {
                    assert!(e.is_instance_of::<pyo3::exceptions::PyException>(py));
                }
            }
        });
    }
}
