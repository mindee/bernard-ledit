//! PDF utilities of the php bindings for bernard-ledit
pub mod document;
pub mod error;
mod fix;
pub mod page;
mod page_options;
pub mod text_char;

use ext_php_rs::prelude::*;

/// Register classes.
pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    let module = module
        .class::<document::PhpPdfDocument>()
        .class::<page::PdfPage>()
        .class::<text_char::PhpTextChar>();

    let module = document::register_functions(module);
    let module = fix::register_functions(module);

    page_options::register_functions(module)
}
