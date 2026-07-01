//! PDF module.

/// Bitmap module.
pub mod bitmap;
/// PDF Document module.
pub mod document;
/// PDF Error module.
pub mod error;
/// Font resolution module.
pub mod font_resolver;
/// JPEG embedding module.
pub mod jpeg;
/// PDF Page module.
pub mod page;
/// Text character module.
pub mod text_char;

pub use bitmap::Bitmap;
pub use document::Document;
pub use error::PdfError;
pub use page::Page;
use std::sync::OnceLock;
pub use text_char::TextChar;

use pdfium_render::prelude::*;

static PDFIUM: OnceLock<Pdfium> = OnceLock::new();

#[cfg(test)]
fn default_pdfium_path_for_tests() -> Option<String> {
    let os_arch = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "mac-x64",
        ("macos", "aarch64") => "mac-arm64",
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("windows", "x86_64") => "win-x64",
        _ => return None,
    };

    let lib_name = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };

    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".pdfium_cache")
        .join("chromium_7825")
        .join(os_arch);
    let path = if cfg!(target_os = "windows") {
        base.join("bin").join(lib_name)
    } else {
        base.join("lib").join(lib_name)
    };

    path.exists().then(|| path.display().to_string())
}

/// Initializes the `PDFium` library.
/// # Errors
/// * `PdfiumError` - If `PDFium` library binding fails.
pub fn initialize(library_path: &str) -> Result<(), PdfError> {
    if PDFIUM.get().is_some() {
        return Ok(());
    }
    let bindings = Pdfium::bind_to_library(library_path)?;
    let _ = PDFIUM.set(Pdfium::new(bindings));
    Ok(())
}

pub(crate) fn pdfium() -> &'static Pdfium {
    PDFIUM.get_or_init(|| {
        let path = std::env::var("PDFIUM_PATH")
            .or_else(|_| {
                #[cfg(test)]
                let res = default_pdfium_path_for_tests().ok_or(std::env::VarError::NotPresent);

                #[cfg(not(test))]
                let res = Err(std::env::VarError::NotPresent);

                res
            })
            .expect("PDFIUM_PATH must be set before importing bernard_ledit");
        let bindings = Pdfium::bind_to_library(path).expect("bundled pdfium not found");
        Pdfium::new(bindings)
    })
}
