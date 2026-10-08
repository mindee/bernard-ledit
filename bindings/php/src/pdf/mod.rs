//! PDF utilities of the php bindings for bernard-ledit
pub mod bitmap;
pub mod document;
pub mod error;
pub mod page;
pub mod text_char;

use ext_php_rs::{
    flags::IniEntryPermission,
    prelude::*,
    zend::{IniEntryDef, IniEntryDefs},
};
use std::ffi::CStr;
use std::sync::OnceLock;

#[cfg(not(feature = "static-pdfium"))]
use ext_php_rs::zend::ExecutorGlobals;

pub const INI_PDFIUM_PATH: &CStr = c"bernard_ledit.pdfium_path";

/// Register classes and ini entries.
pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<error::PdfiumException>()
        .class::<document::PdfDocument>()
        .class::<page::PdfPage>()
        .class::<bitmap::PhpPdfBitmap>()
        .class::<text_char::PhpTextChar>()
}

/// Called from MINIT (`lib.rs::startup`).
pub fn register_ini(mod_num: i32) {
    static INI_ENTRIES: IniEntryDefs<2> = IniEntryDefs::new([
        IniEntryDef::new(INI_PDFIUM_PATH, c"", IniEntryPermission::System),
        IniEntryDef::end(),
    ]);
    IniEntryDef::register(INI_ENTRIES.as_slice(), mod_num);
}

/// Ensures pdfium is bound.
/// # Errors
/// Returns a `PhpException` if pdfium could not be bound.
pub(crate) fn ensure_pdfium() -> PhpResult<()> {
    static RESULT: OnceLock<Result<(), String>> = OnceLock::new();
    RESULT
        .get_or_init(|| bind_pdfium().map_err(|e| e.to_string()))
        .clone()
        .map_err(PhpException::from_class::<error::PdfiumException>)
}

#[cfg(feature = "static-pdfium")]
fn bind_pdfium() -> Result<(), bernard_ledit::pdf::PdfError> {
    bernard_ledit::pdf::initialize_static()
}

#[cfg(not(feature = "static-pdfium"))]
fn bind_pdfium() -> Result<(), bernard_ledit::pdf::PdfError> {
    use bernard_ledit::pdf::{PdfError, initialize};
    let ini = ExecutorGlobals::get().ini_values();
    let from_ini = ini
        .get(INI_PDFIUM_PATH.to_str().unwrap())
        .cloned()
        .flatten()
        .filter(|s| !s.is_empty());
    let ext_dir = ini.get("extension_dir").cloned().flatten();
    let candidates = [
        std::env::var("PDFIUM_PATH").ok(),
        from_ini,
        ext_dir.map(|d| format!("{d}/libpdfium.so")),
    ];
    let mut last = None;
    for path in candidates.into_iter().flatten() {
        match initialize(&path) {
            Ok(()) => return Ok(()),
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| {
        PdfError::Other(
            "libpdfium not found: set PDFIUM_PATH env or bernard_ledit.pdfium_path INI".into(),
        )
    }))
}
