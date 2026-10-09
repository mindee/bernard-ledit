pub mod error;
pub mod image_data;

use ext_php_rs::prelude::*;

pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    let module = module
        .class::<error::ImageException>()
        .class::<image_data::Image>();
    image_data::register_functions(module)
}
