//! PDF module.

/// Bitmap module.
pub mod bitmap;
/// PDF Document module.
pub mod document;
/// PDF Error module.
pub mod error;
/// JPEG embedding module.
pub mod jpeg;
/// PDF Page module.
pub mod page;

pub use bitmap::Bitmap;
pub use document::Document;
pub use error::PdfError;
pub use page::Page;
use std::sync::OnceLock;

use pdfium_render::prelude::*;

static PDFIUM: OnceLock<Pdfium> = OnceLock::new();

/// Initializes the `PDFium` library.
/// # Errors
/// * `PdfiumError` - If `PDFium` library binding fails.
pub fn initialize(library_path: &str) -> Result<(), PdfError> {
    if PDFIUM.get().is_some() {
        return Ok(());
    }
    let bindings = Pdfium::bind_to_library(library_path)?;
    let _ = PDFIUM.set(Pdfium::new(bindings));
    Ok(())
}

pub(crate) fn pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(|| {
        let path = std::env::var("PDFIUM_PATH").expect("PDFIUM_PATH environment variable not set");
        let bindings = Pdfium::bind_to_library(path).expect("bundled pdfium not found");
        Pdfium::new(bindings)
    })
}
