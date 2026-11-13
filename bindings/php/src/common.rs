use ext_php_rs::prelude::*;

/// Returns the canonical encoding name for a file extension.
#[php_function]
#[php(name = "BernardLedit\\Common\\formatFromExtension")]
pub fn format_from_extension(extension: &str) -> String {
    bernard_ledit::common::format_from_extension(extension)
}

pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    module.function(wrap_function!(format_from_extension))
}
