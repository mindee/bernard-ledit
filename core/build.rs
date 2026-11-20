//! Links with pdfium from PDFIUM_DYNAMIC_LIB_PATH
//! TODO: remove this and use
use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-env-changed=PDFIUM_DYNAMIC_LIB_PATH");
    let path = env::var_os("PDFIUM_DYNAMIC_LIB_PATH").expect(
        "set PDFIUM_DYNAMIC_LIB_PATH to the directory containing the PDFium shared library",
    );
    let name = match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("windows") => "pdfium.lib",
        Ok("macos") => "libpdfium.dylib",
        _ => "libpdfium.so",
    };
    assert!(
        Path::new(&path).join(name).is_file(),
        "PDFIUM_DYNAMIC_LIB_PATH must contain {name}"
    );
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&path).display()
    );
    println!("cargo:rustc-link-lib=dylib=pdfium");
}
