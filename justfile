set shell := ["bash", "-c"]

# List the default tasks
[private]
default:
    @just --list

# Java binding commands
mod java "bindings/java/justfile"
# Python binding commands
mod python "bindings/python/justfile"
# Rust core commands
mod rust "rust.just"
# Rust core commands
mod core "rust.just"

# Shortcut to lint Rust code.
lint:
    @just core lint

# Shortcut to run Rust tests
test:
    @just core test

# Shortcut to run Rust build
build:
    @just core build

# Shortcut to run Rust format
format:
    @just core format

# Shortcut to run Rust check-all
check-all:
    @just core check-all
