use crate::args::{self, value_error};
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
use ext_php_rs::{php_function, wrap_function};

/// Keeps or removes pages, if the PDF has at least `$on_min_pages` pages. Negative indexes count
/// from the end, and indexes past the end are skipped.
///
/// # Parameters
/// - `operation`: `string` `KEEP_ONLY` or `REMOVE`, case-insensitive.
/// - `on_min_pages`: `int<0, 4294967295>`
/// - `page_indexes`: `list<int<-2147483648, 2147483647>>`
/// # Returns
/// `array{0: \BernardLedit\Pdf\PdfDocument|null, 1: list<int>}` The new document, or `null` if
/// the PDF has fewer than `$on_min_pages` pages; and the skipped indexes.
/// # Errors
/// - `\ValueError` if `$on_min_pages` or an index is out of range, `$operation` isn't `KEEP_ONLY`
///   or `REMOVE`, the PDF has no content, or no pages would be left.
/// - `\BernardLedit\Pdf\PdfiumException` if `PDFium` can't be loaded, `$data` isn't a PDF it can
///   read, or it can't build the new document.
#[php_function]
#[php(name = "BernardLedit\\Pdf\\applyPageOptions")]
#[allow(clippy::needless_pass_by_value)]
pub fn apply_page_options(
    data: BinarySlice<u8>,
    operation: &str,
    on_min_pages: i64,
    page_indexes: Vec<i64>,
) -> PhpResult<Vec<Zval>> {
    let on_min_pages = args::int("on_min_pages", on_min_pages, 0, u32::MAX)?;
    let page_indexes = args::ints("page_indexes", &page_indexes, i32::MIN, i32::MAX)?;
    super::ensure_pdfium()?;
    let doc = Document::from_bytes(data.to_vec()).map_err(|e| map_pdf_err(&e))?;
    let op = operation
        .parse::<PageOperation>()
        .map_err(|e| value_error(e.to_string()))?;
    let out = bernard_ledit::pdf::page_options::apply_page_options(
        &doc,
        op,
        on_min_pages as usize,
        &page_indexes,
    )
    .map_err(|e| match e {
        PageOptionsError::Pdf(inner) => map_pdf_err(&inner),
        other => value_error(other.to_string()),
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
