use pyo3::prelude::*;

/// PDF bitmap.
pub mod bitmap;
/// PDF document.
pub mod document;
/// PDF errors.
pub mod error;
mod extract;
mod fix;
/// PDF page.
pub mod page;
mod page_options;
/// Text character.
pub mod text_char;

/// Register the `pdf` submodule.
/// # Errors
/// Returns a Python error if the submodule cannot be registered.
pub fn register_submodule(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new(parent.py(), "pdf")?;
    m.add_class::<document::PyPdfDocument>()?;
    m.add_class::<page::PyPdfPage>()?;
    m.add_class::<page::PageSize>()?;
    m.add_class::<bitmap::PyPdfBitmap>()?;
    m.add_class::<text_char::PyTextChar>()?;
    m.add(
        "PdfiumError",
        parent.py().get_type::<error::PyPdfiumError>(),
    )?;
    m.add(
        "PdfExtractionError",
        parent.py().get_type::<error::PyPdfExtractionError>(),
    )?;
    m.add(
        "PdfFixError",
        parent.py().get_type::<error::PyPdfFixError>(),
    )?;
    m.add(
        "PageOptionsError",
        parent.py().get_type::<error::PyPageOptionsError>(),
    )?;
    m.add_function(wrap_pyfunction!(document::compress, &m)?)?;
    m.add_function(wrap_pyfunction!(extract::extract_from_page_ranges, &m)?)?;
    m.add_function(wrap_pyfunction!(fix::fix_pdf, &m)?)?;
    m.add_function(wrap_pyfunction!(page_options::apply_page_options, &m)?)?;
    parent.add_submodule(&m)?;
    Ok(())
}
