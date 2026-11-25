[private]
default:
    @just -f rust.just --list

build:
    cargo build -p bernard-ledit

build-release:
    cargo build --release -p bernard-ledit

clean:
    cargo clean -p bernard-ledit

format:
    cargo fmt --all

format-check:
    cargo fmt --all --check
    cargo clippy --all-features --all-targets -- -D warnings
    cargo check -p bernard-ledit

# Checks for dependencies (requires nightly for cargo-udeps; installs it automatically if missing)
check-deps:
    cargo deny check && (rustup toolchain install nightly --no-self-update 2>/dev/null || true) && cargo +nightly udeps

test:
    cargo nextest run -p bernard-ledit

test-release:
    cargo nextest run -p bernard-ledit --release
