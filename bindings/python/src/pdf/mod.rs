use pyo3::prelude::*;

/// PDF bitmap.
pub mod bitmap;
/// PDF document.
pub mod document;
/// PDF errors.
pub mod error;
/// PDF page.
pub mod page;

/// Register the `pdf` submodule.
/// # Errors
/// Returns a Python error if the submodule cannot be registered.
pub fn register_submodule(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new(parent.py(), "pdf")?;
    m.add_class::<document::PyPdfDocument>()?;
    m.add_class::<page::PyPdfPage>()?;
    m.add_class::<page::PageSize>()?;
    m.add_class::<bitmap::PyPdfBitmap>()?;
    m.add(
        "PdfiumError",
        parent.py().get_type::<error::PyPdfiumError>(),
    )?;
    parent.add_submodule(&m)?;
    Ok(())
}
