//! PHP bindings for the Bernard l'Édit library.

#![allow(missing_docs)]

use ext_php_rs::prelude::*;

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
