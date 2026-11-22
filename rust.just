[private]
default:
    @just -f rust.just --list

# Builds the core Rust library in release mode
build:
    PDFIUM_DYNAMIC_LIB_PATH="$(just core pdfium-path)" cargo build --release -p bernard-ledit

# Runs cargo clippy
clippy:
    cargo clippy --all-features --all-targets -- -D warnings

# Runs cargo fmt on Rust
format:
    cargo fmt --all

# Runs cargo fmt check on Rust
format-check:
    cargo fmt --all --check

# Cleans Rust
clean:
    cargo clean && rm -rf ./core/.pdfium_cache

pdfium_release := replace(`tr -d '[:space:]' < core/pdfium-release`, "/", "_")
pdfium_os_arch := if os() == "macos" {
    if arch() == "aarch64" {
        "mac-arm64"
    } else if arch() == "x86_64" {
        "mac-x64"
    } else {
        "unsupported"
    }
} else if os() == "windows" {
    if arch() == "x86_64" { "win-x64" } else { "unsupported" }
} else if os() == "linux" {
    if arch() == "aarch64" {
        "linux-arm64"
    } else if arch() == "x86_64" {
        "linux-x64"
    } else {
        "unsupported"
    }
} else {
    "unsupported"
}
pdfium_lib := if os() == "windows" { "pdfium.dll" } else if os() == "macos" { "libpdfium.dylib" } else { "libpdfium.so" }
pdfium_lib_subdir := if os() == "windows" { "bin" } else { "lib" }
pdfium_cache_dir := "core/.pdfium_cache/" + pdfium_release + "/" + pdfium_os_arch

# Returns the directory containing the development PDFium shared library, downloading it once.
pdfium-path:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{pdfium_os_arch}}" = "unsupported" ]; then
        echo "Unsupported PDFium platform: {{os()}}/{{arch()}}" >&2
        exit 1
    fi
    cache_dir="{{pdfium_cache_dir}}"
    lib_dir="$cache_dir/{{pdfium_lib_subdir}}"
    lib_path="$lib_dir/{{pdfium_lib}}"
    if [ ! -f "$lib_path" ]; then
        mkdir -p "$cache_dir"
        release="$(tr -d '[:space:]' < core/pdfium-release)"
        release_url="${release//\//%2F}"
        url="https://github.com/bblanchon/pdfium-binaries/releases/download/$release_url/pdfium-{{pdfium_os_arch}}.tgz"
        echo "Downloading PDFium from: $url" >&2
        curl -fL "$url" | tar -xz -C "$cache_dir"
    fi
    [ -f "$lib_path" ] || { echo "PDFium library not found at $lib_path" >&2; exit 1; }
    realpath "$lib_dir"

# Checks for dependencies (requires nightly for cargo-udeps; installs it automatically if missing)
check-deps:
    cargo deny check && (rustup toolchain install nightly --no-self-update 2>/dev/null || true) && cargo +nightly udeps

# Checks the core library across all feature combinations without dev-dependencies
hack-check:
    cargo hack check -p bernard-ledit --feature-powerset --no-dev-deps

# Features that link PDFium statically: linking them needs a prebuilt static library
# (PDFIUM_STATIC_LIB_PATH), which build-static-pdfium.yml's tests provide.
static_features := "static-pdfium,static-pdfium-libcxx,static-pdfium-msvc"

# Tests all feature combinations using nextest, except the static PDFium ones.
hack-test:
    #!/usr/bin/env bash
    set -euo pipefail
    pdfium_lib_dir="$(just core pdfium-path)"
    PDFIUM_DYNAMIC_LIB_PATH="$pdfium_lib_dir" \
    DYLD_LIBRARY_PATH="$pdfium_lib_dir:${DYLD_LIBRARY_PATH:-}" \
    LD_LIBRARY_PATH="$pdfium_lib_dir:${LD_LIBRARY_PATH:-}" \
    PATH="$pdfium_lib_dir:$PATH" \
        cargo hack nextest run -p bernard-ledit --feature-powerset --exclude-features {{static_features}}

# Tests all feature combinations in release mode, except the static PDFium ones.
hack-test-release:
    #!/usr/bin/env bash
    set -euo pipefail
    pdfium_lib_dir="$(just core pdfium-path)"
    PDFIUM_DYNAMIC_LIB_PATH="$pdfium_lib_dir" \
    DYLD_LIBRARY_PATH="$pdfium_lib_dir:${DYLD_LIBRARY_PATH:-}" \
    LD_LIBRARY_PATH="$pdfium_lib_dir:${LD_LIBRARY_PATH:-}" \
    PATH="$pdfium_lib_dir:$PATH" \
        cargo hack nextest run -p bernard-ledit --release --feature-powerset --exclude-features {{static_features}}

# Test the core Rust library
test:
    #!/usr/bin/env bash
    set -euo pipefail
    pdfium_lib_dir="$(just core pdfium-path)"
    PDFIUM_DYNAMIC_LIB_PATH="$pdfium_lib_dir" \
    DYLD_LIBRARY_PATH="$pdfium_lib_dir:${DYLD_LIBRARY_PATH:-}" \
    LD_LIBRARY_PATH="$pdfium_lib_dir:${LD_LIBRARY_PATH:-}" \
    PATH="$pdfium_lib_dir:$PATH" \
        cargo nextest run -p bernard-ledit

# Runs Cargo formatting and Clippy on all Rust code
lint: format-check clippy check-deps hack-check

# Checks all lint tools
check-all: format lint build check-deps test hack-test
    @echo "'\033[0;32m'All Rust checks passed.\033[0m"
