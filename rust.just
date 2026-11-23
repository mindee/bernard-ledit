[private]
default:
    @just -f rust.just --list

build:
    cargo build --release -p bernard-ledit

clippy:
    cargo clippy --all-features --all-targets -- -D warnings

format:
    cargo fmt --all

format-check:
    cargo fmt --all --check

clean:
    cargo clean

# Checks for dependencies (requires nightly for cargo-udeps; installs it automatically if missing)
check-deps:
    cargo deny check && (rustup toolchain install nightly --no-self-update 2>/dev/null || true) && cargo +nightly udeps

check:
    cargo check -p bernard-ledit

test:
    cargo nextest run -p bernard-ledit

test-release:
    cargo nextest run -p bernard-ledit --release

lint: format-check clippy check-deps check

check-all: format lint build check-deps test test-release
    @echo "'\033[0;32m'All Rust checks passed.\033[0m"
