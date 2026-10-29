use crate::pdf::document::PhpPdfDocument;
use crate::pdf::error::map_pdf_err;
use bernard_ledit::pdf::Document;
use bernard_ledit::pdf::page_options::{PageOperation, PageOptionsError};
use ext_php_rs::binary_slice::BinarySlice;
use ext_php_rs::builders::ModuleBuilder;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::exception::PhpException;
use ext_php_rs::prelude::PhpResult;
use ext_php_rs::types::Zval;
use ext_php_rs::zend::ce;
use ext_php_rs::{php_function, wrap_function};

/// Keeps or removes pages. Negative indexes count from the end.
///
/// # Errors
/// - `\ValueError` if `$operation` isn't `KEEP_ONLY` or `REMOVE`, the PDF has no pages, or no
///   pages would be left.
/// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't be loaded, `$data` isn't a PDF it can
///   read, or it can't build the new document.
#[php_function]
#[php(name = "BernardLedit\\Pdf\\applyPageOptions")]
#[allow(clippy::needless_pass_by_value)]
pub fn apply_page_options(
    data: BinarySlice<u8>,
    operation: &str,
    on_min_pages: u32,
    page_indexes: Vec<i32>,
) -> PhpResult<Vec<Zval>> {
    super::ensure_pdfium()?;
    let doc = Document::from_bytes(data.to_vec()).map_err(|e| map_pdf_err(&e))?;
    let op = operation
        .parse::<PageOperation>()
        .map_err(|e| PhpException::new(e.to_string(), 0, ce::value_error()))?;
    let out = bernard_ledit::pdf::page_options::apply_page_options(
        &doc,
        op,
        on_min_pages as usize,
        &page_indexes,
    )
    .map_err(|e| match e {
        PageOptionsError::Pdf(inner) => map_pdf_err(&inner),
        other => PhpException::new(other.to_string(), 0, ce::value_error()),
    })?;

    let mut doc_zval = Zval::new();
    if let Some(d) = out.document {
        PhpPdfDocument::wrap(d)
            .set_zval(&mut doc_zval, false)
            .map_err(|e| PhpException::from_message(e.to_string()))?;
    } else {
        doc_zval.set_null();
    }
    let mut skipped = Zval::new();
    skipped
        .set_array(out.skipped)
        .map_err(|e| PhpException::from_message(e.to_string()))?;
    Ok(vec![doc_zval, skipped])
}

/// Registers the internal functions in this module.
pub fn register_functions(builder: ModuleBuilder) -> ModuleBuilder {
    builder.function(wrap_function!(apply_page_options))
}
