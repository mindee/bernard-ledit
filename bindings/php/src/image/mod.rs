pub mod error;
pub mod image_data;

use ext_php_rs::prelude::*;

pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    let module = module
        .class::<error::ImageException>()
        .class::<error::ImageExtractException>()
        .class::<image_data::PhpImage>();
    image_data::register_functions(module)
}
