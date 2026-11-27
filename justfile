set shell := ["bash", "-c"]

[private]
default:
    @just --list

mod java "bindings/java/justfile"
mod python "bindings/python/justfile"
mod php "bindings/php/justfile"
mod rust "rust.just"
mod core "rust.just"

clean:
    @just core clean
    @just java clean
    @just python clean
    @just php clean

format:
    just core format
    just java format
    just python format
    just php format

format-check:
    just core format-check
    just java format-check
    just python format-check
    just php format-check

# Runs the important checks and test suites, stopping at the first failure
check-all:
    #!/usr/bin/env bash
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

    run_check actionlint

    run_check just core format-check
    skip_check "just core check-deps" "cargo-udeps/Clang 21 build failure"
    run_check just core test

    run_check just python build
    run_check just python format-check
    run_check just python stubs-check
    run_check just python test

    run_check just php build
    run_check just php format-check
    run_check just php stubs-check
    run_check just php test
    run_check just php test-static

    print_summary
    printf '\n%sAll checks passed.%s\n' "$green" "$reset"
