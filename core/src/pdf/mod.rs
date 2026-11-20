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

pub(crate) fn pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(|| {
        // NOTE: pdfium library is dynamically linked! this just binds to the already linked library to its rust binding (pdfium-render)
        // bind_to_statically_linked_library is a misleading name
        let bindings = Pdfium::bind_to_statically_linked_library()
            .unwrap_or_else(|err| panic!("failed to initialize PDFium bindings: {err}"));
        Pdfium::new(bindings)
    })
}
