//! PDF module.

/// Bitmap module.
pub mod bitmap;
/// PDF Document module.
pub mod document;
/// PDF Error module.
pub mod error;
/// Font resolution module.
pub mod font_resolver;
/// JPEG embedding module.
pub mod jpeg;
/// PDF Page module.
pub mod page;
/// Text character module.
pub mod text_char;

pub use bitmap::Bitmap;
pub use document::Document;
pub use error::PdfError;
pub use page::Page;
use std::sync::{Mutex, OnceLock};
pub use text_char::TextChar;

use pdfium_render::prelude::*;

static PDFIUM: OnceLock<Pdfium> = OnceLock::new();

/// pdfium and mozjpeg both aren't thread-safe.
pub(crate) static NATIVE_LOCK: Mutex<()> = Mutex::new(());

/// Returns `true` if `PDFium` has been bound.
#[must_use]
pub fn is_initialized() -> bool {
    PDFIUM.get().is_some()
}

/// Binds `PDFium` from a shared library on disk. Does NOT work for `static-pdfium`.
/// # Errors
/// * `PdfiumError` If `PDFium` library binding fails.
#[cfg(not(feature = "static-pdfium"))]
pub fn initialize(library_path: &str) -> Result<(), PdfError> {
    if PDFIUM.get().is_some() {
        return Ok(());
    }
    let bindings = Pdfium::bind_to_library(library_path)?;
    let _ = PDFIUM.set(Pdfium::new(bindings));
    Ok(())
}

#[cfg(not(feature = "static-pdfium"))]
pub(crate) fn pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(|| {
        let path = std::env::var("PDFIUM_PATH").unwrap_or_else(|_| env!("PDFIUM_PATH").to_string());
        let bindings = Pdfium::bind_to_library(path.clone())
            .unwrap_or_else(|err| panic!("failed to bind PDFium at '{path}': {err}"));
        Pdfium::new(bindings)
    })
}

#[cfg(feature = "static-pdfium")]
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
#[cfg(feature = "static-pdfium")]
pub fn initialize_static() -> Result<(), PdfError> {
    if PDFIUM.get().is_some() {
        return Ok(());
    }
    let bindings = Pdfium::bind_to_statically_linked_library()?;
    let _ = PDFIUM.set(Pdfium::new(bindings));
    Ok(())
}
