use crate::args::{self, value_error};
use ext_php_rs::binary::Binary;
use ext_php_rs::binary_slice::BinarySlice;
use ext_php_rs::builders::ModuleBuilder;
use ext_php_rs::prelude::PhpResult;
use ext_php_rs::{php_function, wrap_function};

/// Drops every byte before the first `%PDF-` marker.
/// # Parameters
/// - `maximum_offset`: `int<0, max>` How far into `$data` to look for the marker, in bytes.
/// # Errors
/// - `\ValueError` if `$maximum_offset` is negative, or there's no marker before it.
#[php_function]
#[php(name = "BernardLedit\\Pdf\\fixPdf")]
#[php(defaults(maximum_offset = 500))]
#[allow(clippy::needless_pass_by_value)]
pub fn fix_pdf(data: BinarySlice<u8>, maximum_offset: i64) -> PhpResult<Binary<u8>> {
    let offset = args::non_negative("maximum_offset", maximum_offset)?;
    bernard_ledit::pdf::fix::fix_pdf(&data, offset)
        .map(|fixed| Binary::new(fixed.to_vec()))
        .map_err(|e| value_error(e.to_string()))
}

/// Registers the internal functions in this module.
pub fn register_functions(builder: ModuleBuilder) -> ModuleBuilder {
    builder.function(wrap_function!(fix_pdf))
}
