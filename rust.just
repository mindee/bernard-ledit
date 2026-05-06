[private]
default:
    @just -f rust.just --list

# Builds the core Rust library in release mode
build:
    cargo build --release -p bernard-ledit

# Tests the core Rust library in release mode
test:
    cargo nextest run -p bernard-ledit

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

# Checks for dependencies
check-deps:
    cargo deny check

# Runs Cargo formatting and Clippy on all Rust code
lint: format-check clippy check-deps

# Checks all lint tools
check-all: format lint build check-deps test
    @echo "'\033[0;32m'All Rust checks passed.\033[0m"