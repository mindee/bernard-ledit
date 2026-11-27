# Contributing to Bernard l'Édit

Thank you for your interest in contributing! This document covers everything you need to get started.

## Prerequisites

### Rust toolchain

Install Rust via [rustup](https://rustup.rs). The exact toolchain version and components are declared in `rust-toolchain.toml` and will be installed automatically on first use.

### just

All build, test, formatting, and static-analysis commands are run via [just](https://github.com/casey/just):

```bash
cargo install just
```

Run `just` with no arguments to list all available commands.

### Cargo tools

Install the required cargo tools using [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) (pre-built binaries, no compilation needed):

```bash
cargo install cargo-binstall
cargo binstall cargo-deny cargo-nextest
```

### Binding-specific dependencies

For binding-specific setup, build, and test instructions, see the
[binding documentation](README.md#bindings).

---

## Building

```bash
# Rust core
just build
```

---

## Testing

```bash
# Rust core tests
just core test

# Run the consolidated Rust, Python, and PHP checks
just check-all
```

> **Note:** Rust tests run single-threaded (`RUST_TEST_THREADS=1` via `.cargo/config.toml`).
> PDFium rendering is guarded by a process-wide mutex.
> Do not override this when running the test suite.

---

## Formatting and static checks

```bash
# Check formatting and static analysis
just format-check

# Apply formatting/fixes across all bindings
just format
```

---

## Git hooks

The repository includes a pre-commit hook in `.cargo-husky/hooks`, but Git does not enable it automatically. To enable it for this checkout:

```bash
git config core.hooksPath .cargo-husky/hooks
```

The hook runs checks for the relevant language when files are staged for commit.

---

## Submitting changes

1. Create a branch from `dev` (not `main` — see below).
2. Make your changes with tests where applicable.
3. Ensure `just check-all` passes. Use `just format-check` for formatting and static-analysis checks across all bindings.
4. Add a `CHANGELOG.md` entry.
5. If a dependency requires attribution, include it in the release or package metadata.
6. Open a pull request against **`dev`** using the provided template.

> **Branch strategy:** `main` is the release branch — only `dev` is merged into it (via the release PR template).
> All feature and fix PRs target `dev`.
