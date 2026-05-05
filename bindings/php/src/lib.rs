//! PHP bindings for the Quilldee library.

#![allow(missing_docs)]

use ext_php_rs::prelude::*;

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
