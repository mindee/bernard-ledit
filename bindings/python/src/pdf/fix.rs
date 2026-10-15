use crate::input::BinaryInput;
use crate::pdf::error::PyPdfFixError;
use pyo3::types::PyBytes;
use pyo3::{Bound, PyResult, Python, pyfunction};

#[pyfunction]
#[pyo3(signature = (data, maximum_offset = 500))]
pub fn fix_pdf<'py>(
    py: Python<'py>,
    data: BinaryInput<'py>,
    maximum_offset: usize,
) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = data.into_vec()?;
    let fixed = bernard_ledit::pdf::fix::fix_pdf(&bytes, maximum_offset)
        .map_err(|e| PyPdfFixError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, fixed))
}
