//! PDF utilities of the php bindings for bernard-ledit
pub mod document;
pub mod error;
mod fix;
pub mod page;
mod page_options;
pub mod text_char;

use ext_php_rs::prelude::*;
use std::sync::OnceLock;

/// Register classes and functions.
pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    let module = module
        .class::<document::PhpPdfDocument>()
        .class::<page::PdfPage>()
        .class::<text_char::PhpTextChar>();

    let module = document::register_functions(module);
    let module = fix::register_functions(module);

    page_options::register_functions(module)
}
/// Ensures pdfium is bound.
/// # Errors
/// Returns a `PhpException` if pdfium could not be bound.
pub(crate) fn ensure_pdfium() -> PhpResult<()> {
    static RESULT: OnceLock<Result<(), String>> = OnceLock::new();
    RESULT
        .get_or_init(|| bind_pdfium().map_err(|e| e.to_string()))
        .clone()
        .map_err(crate::exceptions::pdfium_exception)
}

fn bind_pdfium() -> Result<(), bernard_ledit::pdf::PdfError> {
    bernard_ledit::pdf::initialize()
}
