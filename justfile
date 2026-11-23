set shell := ["bash", "-c"]

# List the default tasks
[private]
default:
    @just --list

mod java "bindings/java/justfile"
mod python "bindings/python/justfile"
mod php "bindings/php/justfile"
mod rust "rust.just"
mod core "rust.just"

# Runs the important checks and test suites, stopping at the first failure
check-all:
    #!/usr/bin/env bash
    # Runs the checks CI runs and the binding test suites, cheapest first, and stop at 1st failure
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
    run_check just core check
    run_check just core test

    run_check just python build
    run_check just python lint-check
    run_check just python stubs-check
    run_check just python test

    run_check just php build
    run_check just php stubs-check
    run_check just php test
    run_check just php test-static

    print_summary
    printf '\n%sAll checks passed.%s\n' "$green" "$reset"

clean:
    @just core clean
    @just java clean
    @just python clean
    @just php clean
