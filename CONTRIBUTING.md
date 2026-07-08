# Contributing to Bernard l'Édit

Thank you for your interest in contributing! This document covers everything you need to get started.

## Prerequisites

### Rust toolchain

Install Rust via [rustup](https://rustup.rs). The exact toolchain version and components are declared in `rust-toolchain.toml` and will be installed automatically on first use.

### just

All build, test, and lint commands are run via [just](https://github.com/casey/just):

```bash
cargo install just
```

Run `just` with no arguments to list all available commands.

### Cargo tools

Install the required cargo tools using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) (pre-built binaries, no compilation needed):

```bash
cargo install cargo-binstall
cargo binstall cargo-deny cargo-nextest cargo-hack
```

### Binding-specific dependencies

You only need these if you are working on the corresponding language binding.

#### Python (`bindings/python`)

Python 3.9+, a virtualenv, and [maturin](https://github.com/PyO3/maturin) are required:

```bash
# Debian / Ubuntu
sudo apt install python3 python3-venv

python3 -m venv .venv
source .venv/bin/activate
pip install maturin
```

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

---

## Building

```bash
# Rust core
just build

# Python binding (installs into the active virtualenv via maturin develop)
just python build

# Java binding
just java build
```

---

## Testing

```bash
# Rust core
just test

# Python binding
just python test

# Full Rust check (all feature combinations, lint, tests)
just check
```

> **Note:** Rust tests run single-threaded (`RUST_TEST_THREADS=1` via `.cargo/config.toml`).
> This is required because pdfium's renderer and mozjpeg are not safe for concurrent use.
> Do not override this when running the test suite.

---

## Linting and formatting

```bash
# Rust: fmt check + clippy + cargo-deny + cargo-udeps + hack-check
just lint

# Python: ruff + mypy
just python lint-check

# Auto-fix formatting
just format         # Rust
just python format  # Python
```

---

## Git hooks

Git hooks are managed via [`cargo-husky`](https://github.com/rhysd/cargo-husky). They are installed automatically the first time you run:

```bash
just test
```

After that, `cargo fmt` is checked on every commit and the full lint + test suite runs on every push.

---

## Submitting changes

1. Create a branch from `dev` (not `main` — see below).
2. Make your changes with tests where applicable.
3. Ensure `just lint` and `just test` pass (and `just python lint-check` + `just python test` for Python changes).
4. Add a `CHANGELOG.md` entry.
5. If you added a dependency, update `THIRD_PARTY_NOTICES` if required by its license.
6. Open a pull request against **`dev`** using the provided template.

> **Branch strategy:** `main` is the release branch — only `dev` is merged into it (via the release PR template).
> All feature and fix PRs target `dev`.
