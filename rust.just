[private]
default:
    @just -f rust.just --list

# Builds the core Rust library in release mode
build:
    cargo build --release -p bernard-ledit

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

# Checks for dependencies (requires nightly for cargo-udeps; installs it automatically if missing)
check-deps:
    cargo deny check && (rustup toolchain install nightly --no-self-update 2>/dev/null || true) && cargo +nightly udeps

# Checks the core library across all feature combinations without dev-dependencies
hack-check:
    cargo hack check -p bernard-ledit --feature-powerset --no-dev-deps

# Tests all feature combinations using nextest
hack-test:
    cargo hack nextest run -p bernard-ledit --feature-powerset

# Tests the core Rust library in release mode
hack-test-release:
    cargo hack nextest run -p bernard-ledit --release --feature-powerset

# Test the core Rust library
test:
    cargo nextest run -p bernard-ledit

# Runs Cargo formatting and Clippy on all Rust code
lint: format-check clippy check-deps hack-check

# Checks all lint tools
check-all: format lint build check-deps test hack-test
    @echo "'\033[0;32m'All Rust checks passed.\033[0m"

