//! PDF module.

/// PDF Document module.
pub mod document;
/// PDF Error module.
pub mod error;
/// PDF Extraction module.
pub mod extract;
/// PDF fixing utility.
pub mod fix;
/// PDF Page module.
pub mod page;
/// Page options to apply to a PDF.
pub mod page_options;
/// Text character module.
pub mod text_char;

pub use document::Document;
pub use error::PdfError;
pub use page::Page;
use std::sync::{Mutex, OnceLock};
pub use text_char::TextChar;

use pdfium_render::prelude::*;

static PDFIUM: OnceLock<Pdfium> = OnceLock::new();

/// Serializes PDFium operations that are not thread-safe.
pub(crate) static NATIVE_LOCK: Mutex<()> = Mutex::new(());

/// Binds `PDFium` from a shared library on disk. Does NOT work for `static-pdfium`.
/// # Errors
/// * `PdfiumError` If `PDFium` library binding fails.
#[cfg(not(feature = "static-link"))]
pub fn initialize(library_path: &str) -> Result<(), PdfError> {
    if PDFIUM.get().is_some() {
        return Ok(());
    }
    let bindings = Pdfium::bind_to_library(library_path)?;
    let _ = PDFIUM.set(Pdfium::new(bindings));
    Ok(())
}

#[cfg(not(feature = "static-link"))]
pub(crate) fn pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(|| {
        let path = std::env::var("PDFIUM_PATH").unwrap_or_else(|_| env!("PDFIUM_PATH").to_string());
        let bindings = Pdfium::bind_to_library(path.clone())
            .unwrap_or_else(|err| panic!("failed to bind PDFium at '{path}': {err}"));
        Pdfium::new(bindings)
    })
}

#[cfg(feature = "static-link")]
pub(crate) fn pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(|| {
        let bindings = Pdfium::bind_to_statically_linked_library()
            .unwrap_or_else(|err| panic!("failed to bind static PDFium: {err}"));
        Pdfium::new(bindings)
    })
}

/// Binds the statically linked `PDFium`. Only exists with `static-pdfium`.
/// # Errors
/// * `PdfiumError` if binding fails.
#[cfg(feature = "static-link")]
pub fn initialize_static() -> Result<(), PdfError> {
    if PDFIUM.get().is_some() {
        return Ok(());
    }
    let bindings = Pdfium::bind_to_statically_linked_library()?;
    let _ = PDFIUM.set(Pdfium::new(bindings));
    Ok(())
}
