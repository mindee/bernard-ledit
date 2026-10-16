use ext_php_rs::binary::Binary;
use ext_php_rs::binary_slice::BinarySlice;
use ext_php_rs::builders::ModuleBuilder;
use ext_php_rs::exception::PhpException;
use ext_php_rs::prelude::PhpResult;
use ext_php_rs::zend::ce;
use ext_php_rs::{php_function, wrap_function};

/// Drops every byte before the first `%PDF-` marker.
/// # Errors
/// Throws a [`ValueError`] if there is no marker before `$maximumOffset`.
#[php_function]
#[php(name = "BernardLedit\\Pdf\\fixPdf")]
#[php(defaults(maximum_offset = 500))]
#[allow(clippy::needless_pass_by_value)]
pub fn fix_pdf(data: BinarySlice<u8>, maximum_offset: i32) -> PhpResult<Binary<u8>> {
    let offset = usize::try_from(maximum_offset).map_err(|_| {
        PhpException::new(
            "maximum_offset cannot be negative".into(),
            0,
            ce::value_error(),
        )
    })?;
    bernard_ledit::pdf::fix::fix_pdf(&data, offset)
        .map(|fixed| Binary::new(fixed.to_vec()))
        .map_err(|e| PhpException::new(e.to_string(), 0, ce::value_error()))
}

/// Registers the internal functions in this module.
pub fn register_functions(builder: ModuleBuilder) -> ModuleBuilder {
    builder.function(wrap_function!(fix_pdf))
}
