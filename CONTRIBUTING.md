# Contributing to Quilldee

Thank you for your interest in contributing! This document covers everything you need to get started.

## Prerequisites

### Rust toolchain

Install Rust via [rustup](https://rustup.rs). The required toolchain version and components are declared in `rust-toolchain.toml` and will be installed automatically on first use.

### Cargo tools

Install the required cargo tools using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) (pre-built binaries, no compilation needed):

```bash
cargo install cargo-binstall
cargo binstall cargo-deny cargo-nextest
```

### Binding-specific dependencies

You only need these if you are working on the corresponding language binding.

#### PHP (`bindings/php`)

`ext-php-rs` requires PHP development headers and `libclang` for bindgen:

```bash
# Debian / Ubuntu
sudo apt install php-dev libclang-dev

# Fedora / RHEL
sudo dnf install php-devel clang-devel
```

#### Node.js (`bindings/nodejs`)

NAPI-RS requires a Node.js runtime. Install it via your system package manager or [nvm](https://github.com/nvm-sh/nvm):

```bash
# Debian / Ubuntu
sudo apt install nodejs npm
```

#### Java (`bindings/java`)

A JDK is required to run the generated UniFFI bindings. Any JDK 11+ distribution works:

```bash
# Debian / Ubuntu
sudo apt install default-jdk
```

#### .NET (`bindings/dotnet`)

The .NET SDK 8.0+ is required:

```bash
# See https://learn.microsoft.com/en-us/dotnet/core/install/linux
sudo apt install dotnet-sdk-8.0
```

#### Ruby (`bindings/ruby`)

Ruby 3.1+ and Bundler are required:

```bash
# Debian / Ubuntu
sudo apt install ruby ruby-dev bundler
```

#### Python (`bindings/python`)

Python 3.9+ is required:

```bash
# Debian / Ubuntu
sudo apt install python3 python3-venv
```

---

## Building

Build only the core library (no binding dependencies needed):

```bash
cargo build -p quilldee
```

Build a specific binding:

```bash
cargo build -p quilldee-python
cargo build -p quilldee-php
# etc.
```

Build everything (requires all binding dependencies above):

```bash
cargo build --workspace
```

---

## Testing

Run the core test suite:

```bash
cargo nextest run -p quilldee
```

Run all tests:

```bash
cargo nextest run --workspace
```

---

## Linting and formatting

```bash
cargo fmt --all -- --check   # check formatting
cargo clippy --all-features --all-targets -- -D warnings
cargo deny check             # license and advisory checks
```

---

## Git hooks

Git hooks are managed via [`cargo-husky`](https://github.com/rhysd/cargo-husky). They are installed automatically the first time you run:

```bash
cargo test -p quilldee
```

After that, `cargo fmt` is checked on every commit and the full lint + test suite runs on every push.

---

## Submitting changes

1. Fork the repository and create a branch from `main`.
2. Make your changes with tests where applicable.
3. Ensure `cargo fmt`, `cargo clippy`, and `cargo nextest run` all pass.
4. Open a pull request against `main` using the provided template.
