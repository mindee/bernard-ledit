//! Requires a static `PDFium` archive for every build.
use std::{env, path::Path};

fn main() {
    let target = env::var("TARGET").expect("Cargo must provide TARGET");
    let target_var = format!("PDFIUM_STATIC_LIB_PATH_{}", target.replace('-', "_"));
    println!("cargo:rerun-if-env-changed=PDFIUM_STATIC_LIB_PATH");
    println!("cargo:rerun-if-env-changed={target_var}");
    let path = env::var_os(&target_var)
        .or_else(|| env::var_os("PDFIUM_STATIC_LIB_PATH"))
        .expect("PDFIUM_STATIC_LIB_PATH must point to the static PDFium archive directory");
    let archive = if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        "pdfium.lib"
    } else {
        "libpdfium.a"
    };
    let archive = Path::new(&path).join(archive);
    assert!(
        archive.is_file(),
        "Static PDFium archive not found: {}. See README.md for download instructions.",
        archive.display()
    );
    println!("cargo:rerun-if-changed={}", archive.display());
}
