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

/// pdfium uses global state during rendering (`FPDF_RenderPageBitmap`) and is
/// not thread-safe by default. Serialise all render calls behind this lock so
/// concurrent threads (Python `threading`, Rust async runtimes, etc.) cannot
/// race inside pdfium internals.
pub(crate) static PDFIUM_RENDER_LOCK: Mutex<()> = Mutex::new(());

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
        let path = std::env::var("PDFIUM_PATH").unwrap_or_else(|_| env!("PDFIUM_PATH").to_string());
        let bindings = Pdfium::bind_to_library(path.clone())
            .unwrap_or_else(|err| panic!("failed to bind PDFium at '{path}': {err}"));
        Pdfium::new(bindings)
    })
}
