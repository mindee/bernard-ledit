set shell := ["bash", "-c"]

# List the default tasks
[private]
default:
    @just --list

# Shared UniFFI library behind the Java, .NET, Node.js and Ruby bindings
mod uniffi "bindings/uniffi/justfile"
# Java binding commands (UniFFI, through its Kotlin bindings)
mod java "bindings/java/justfile"
# .NET binding commands (UniFFI)
mod dotnet "bindings/dotnet/justfile"
# Node.js binding commands (UniFFI)
mod nodejs "bindings/nodejs/justfile"
# Ruby binding commands (UniFFI)
mod ruby "bindings/ruby/justfile"
# Python binding commands
mod python "bindings/python/justfile"
# PHP binding commands
mod php "bindings/php/justfile"
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
check:
    @just core check-all

# Runs the important checks and test suites, stopping at the first failure
check-all:
    #!/usr/bin/env bash
    # Runs the checks CI runs and the binding test suites, cheapest first, and stops at the first
    # failure. Nothing gets formatted or fixed, but a failing PHP stubs check leaves the regenerated
    # stubs in place. The Python checks need an active virtualenv with maturin; the PHP stubs are
    # checked on Linux only, and the static PHP build only when PDFIUM_STATIC_LIB_PATH is set.
    set -uo pipefail
    trap 'printf "\nInterrupted.\n" >&2; exit 130' INT
    if [ -t 1 ]; then
        bold=$'\033[1m' green=$'\033[0;32m' red=$'\033[0;31m' yellow=$'\033[0;33m' reset=$'\033[0m'
    else
        bold='' green='' red='' yellow='' reset=''
    fi
    results=()

    print_summary() {
        printf '\n%sSummary%s\n' "$bold" "$reset"
        printf '  %s\n' "${results[@]}"
    }

    # Runs one check and records how long it took; on failure, prints the summary and exits.
    run_check() {
        local start=$SECONDS status
        printf '\n%s==> %s%s\n' "$bold" "$*" "$reset"
        "$@"
        status=$?
        if [ "$status" -ne 0 ]; then
            results+=("${red}FAIL${reset}  $* ($((SECONDS - start))s)")
            print_summary
            printf '\n%sStopped: %s failed.%s\n' "$red" "$*" "$reset"
            exit "$status"
        fi
        results+=("${green}PASS${reset}  $* ($((SECONDS - start))s)")
    }

    # Records a check that didn't run, and why.
    skip_check() {
        results+=("${yellow}SKIP${reset}  $1: $2")
    }

    if command -v actionlint > /dev/null; then
        run_check actionlint
    else
        skip_check actionlint "not installed"
    fi

    run_check just core format-check
    run_check just core clippy
    run_check just core check-deps
    run_check just core hack-check
    # Every feature combination but the static ones, the default features included.
    run_check just core hack-test

    run_check just python build
    run_check just python lint-check
    run_check just python stubs-check
    run_check just python test

    run_check just php build
    # The stub generator only runs on Linux (see bindings/php/tools/stubs/generate.sh); CI checks the stubs there.
    if [ "$(uname -s)" = Linux ]; then
        run_check just php stubs-check
    else
        skip_check "just php stubs-check" "the stub generator runs on Linux only; CI checks it"
    fi
    run_check just php test
    if [ -n "${PDFIUM_STATIC_LIB_PATH:-}" ]; then
        run_check just php test-static
    else
        skip_check "just php test-static" "set PDFIUM_STATIC_LIB_PATH (see tools/pdfium-static/download.sh)"
    fi

    run_check just uniffi lint
    run_check just uniffi test
    # The UniFFI packages, each when its toolchain is installed.
    for package in "java mvn" "dotnet dotnet" "nodejs npm" "ruby bundle"; do
        read -r name tool <<< "$package"
        if command -v "$tool" > /dev/null; then
            run_check just "$name" ci
        else
            skip_check "just $name ci" "$tool isn't installed"
        fi
    done

    print_summary
    printf '\n%sAll checks passed.%s\n' "$green" "$reset"

# Shortcut to run Rust clean
clean:
    @just core clean

# Cleans up all un-necessary files
clean-all: clean
    @just java clean
    @just python clean
    @just php clean
