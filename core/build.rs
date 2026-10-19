//! Build script: fetches the `PDFium` shared library for dynamic builds.
//! Static builds (`static-pdfium*` features) link a prebuilt archive instead and skip all of this.
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if is_static_build() {
        println!("cargo:rerun-if-env-changed=PDFIUM_STATIC_LIB_PATH");
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target = env::var("TARGET").unwrap();
    let release = read_release(&manifest_dir);
    let (os_arch, lib_name) = platform(&target);

    let cache_dir = manifest_dir
        .join(".pdfium_cache")
        .join(release.replace('/', "_"))
        .join(os_arch);
    // Windows ships the DLL under bin/, the others under lib/.
    let lib_subdir = if target.contains("windows") {
        "bin"
    } else {
        "lib"
    };
    let source_path = cache_dir.join(lib_subdir).join(lib_name);

    ensure_downloaded(&release, os_arch, &cache_dir, &source_path);
    println!("cargo:rustc-env=PDFIUM_PATH={}", source_path.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    copy_library(&source_path, &out_dir.join(lib_name));

    let python_package_dir = manifest_dir.join("../bindings/python/bernard_ledit");
    if python_package_dir.is_dir() {
        copy_library(&source_path, &python_package_dir.join(lib_name));
    } else {
        println!(
            "cargo:warning=Skipping Python PDFium bundling: {} does not exist",
            python_package_dir.display()
        );
    }
}

fn is_static_build() -> bool {
    env::var_os("CARGO_FEATURE_STATIC_LINK").is_some()
}

/// Reads the pinned pdfium-binaries tag (e.g. `chromium/8076`) from `pdfium-release`.
fn read_release(manifest_dir: &Path) -> String {
    let release_file = manifest_dir.join("pdfium-release");
    println!("cargo:rerun-if-changed={}", release_file.display());
    fs::read_to_string(&release_file)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", release_file.display()))
        .trim()
        .to_owned()
}

/// pdfium-binaries' platform name and the library file name for a Rust target triple.
fn platform(target: &str) -> (&'static str, &'static str) {
    match target {
        "x86_64-apple-darwin" => ("mac-x64", "libpdfium.dylib"),
        "aarch64-apple-darwin" => ("mac-arm64", "libpdfium.dylib"),
        "x86_64-unknown-linux-gnu" => ("linux-x64", "libpdfium.so"),
        "aarch64-unknown-linux-gnu" => ("linux-arm64", "libpdfium.so"),
        "x86_64-pc-windows-msvc" => ("win-x64", "pdfium.dll"),
        _ => panic!("Unsupported target architecture: {target}"),
    }
}

/// Downloads and unpacks the release into `cache_dir` unless `library` is already there.
fn ensure_downloaded(release: &str, os_arch: &str, cache_dir: &Path, library: &Path) {
    if library.exists() {
        println!(
            "cargo:warning=Using cached PDFium binary from: {}",
            library.display()
        );
        return;
    }
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir).unwrap();
    }
    fs::create_dir_all(cache_dir)
        .unwrap_or_else(|e| panic!("Failed to create cache directory: {e}"));

    let url = format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/{}/pdfium-{os_arch}.tgz",
        release.replace('/', "%2F")
    );
    println!("cargo:warning=Downloading PDFium from: {url}");
    let response = ureq::get(&url).call().expect("Failed to download PDFium");
    let tar = flate2::read::GzDecoder::new(response.into_body().into_reader());
    tar::Archive::new(tar).unpack(cache_dir).unwrap();

    assert!(
        library.exists(),
        "PDFium extraction succeeded, but expected library was not found at: {}",
        library.display()
    );
}

/// Copies the `PDFium` library from `from` to `to`.
fn copy_library(from: &Path, to: &Path) {
    fs::copy(from, to).unwrap_or_else(|e| {
        panic!(
            "Failed to copy PDFium library from {} to {}: {e}",
            from.display(),
            to.display()
        )
    });
}
