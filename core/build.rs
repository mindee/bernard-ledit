//! Build script for the `pdfium-render` crate.
use std::env;
use std::fs;
use std::path::PathBuf;

const PDFIUM_RELEASE: &str = "chromium/7825";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let os_arch = match target.as_str() {
        "x86_64-apple-darwin" => "mac-x64",
        "aarch64-apple-darwin" => "mac-arm64",
        "x86_64-unknown-linux-gnu" => "linux-x64",
        "aarch64-unknown-linux-gnu" => "linux-arm64",
        "x86_64-pc-windows-msvc" => "win-x64",
        _ => panic!("Unsupported target architecture: {target}"),
    };

    let download_url = format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/{PDFIUM_RELEASE}/pdfium-{os_arch}.tgz"
    );

    let safe_release_name = PDFIUM_RELEASE.replace('/', "_");
    let cache_dir = manifest_dir
        .join(".pdfium_cache")
        .join(safe_release_name)
        .join(os_arch);

    if let Err(e) = fs::create_dir_all(&cache_dir) {
        panic!("Failed to create cache directory: {e}");
    }

    let lib_name = if target.contains("apple") {
        "libpdfium.dylib"
    } else if target.contains("linux") {
        "libpdfium.so"
    } else {
        "pdfium.dll"
    };

    let lib_dir = cache_dir.join("lib");
    let bin_dir = cache_dir.join("bin");

    let source_path = if target.contains("windows") {
        bin_dir.join(lib_name)
    } else {
        lib_dir.join(lib_name)
    };

    if source_path.exists() {
        println!(
            "cargo:warning=Using cached PDFium binary from: {}",
            source_path.display()
        );
    } else {
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir).unwrap();
        }
        fs::create_dir_all(&cache_dir).unwrap();

        println!("cargo:warning=Downloading PDFium from: {download_url}");
        let response = ureq::get(&download_url)
            .call()
            .expect("Failed to download PDFium");
        let reader = response.into_body().into_reader();
        let tar = flate2::read::GzDecoder::new(reader);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&cache_dir).unwrap();
    }

    let target_path = out_dir.join(lib_name);
    println!("cargo:rustc-env=PDFIUM_PATH={}", source_path.display());
    let python_package_dir = manifest_dir.join("../bindings/python/bernard_ledit");

    if source_path.exists() {
        fs::copy(&source_path, &target_path).unwrap_or_else(|e| {
            panic!(
                "Failed to copy PDFium library from {} to {}: {e}",
                source_path.display(),
                target_path.display()
            );
        });
        if python_package_dir.is_dir() {
            let bundled_pdfium_path = python_package_dir.join(lib_name);
            fs::copy(&source_path, &bundled_pdfium_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to copy PDFium library from {} to {}: {e}",
                    source_path.display(),
                    bundled_pdfium_path.display()
                );
            });
        } else {
            println!(
                "cargo:warning=Skipping Python PDFium bundling: {} does not exist",
                python_package_dir.display()
            );
        }
    } else {
        panic!(
            "PDFium extraction succeeded, but expected library was not found at: {}",
            source_path.display()
        );
    }
}
