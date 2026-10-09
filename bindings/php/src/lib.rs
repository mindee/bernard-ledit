//! PHP bindings for the Bernard l'Édit library.
#![cfg_attr(windows, feature(abi_vectorcall))]
#![allow(missing_docs)]

use ext_php_rs::prelude::*;
pub mod image;
pub mod pdf;

/// Loads up the ``bernard-ledit`` library.
/// Changes the library name from ``bernard-ledit-php`` to ``bernard_ledit``.
#[php_module]
#[php(startup = "startup")]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    let module = module.name("bernard_ledit");
    let module = image::register(module);
    pdf::register(module)
}

fn startup(_ty: i32, mod_num: i32) -> i32 {
    pdf::register_ini(mod_num);
    0
}
