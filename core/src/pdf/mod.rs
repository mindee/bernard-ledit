//! PDF module.
/// PDF module.
pub mod pdf_processor;

use pdfium_render::prelude::*;

/// Initializes the `PDFium` library.
/// # Errors
/// * `PdfiumError` - If `PDFium` library binding fails.
pub fn initialize_pdfium(library_path: &str) -> Result<Pdfium, PdfiumError> {
    let bindings = Pdfium::bind_to_library(library_path)?;
    Ok(Pdfium::new(bindings))
}
