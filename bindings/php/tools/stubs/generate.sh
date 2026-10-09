#!/usr/bin/env bash
set -euo pipefail

tools="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd -- "$tools/../../../.." && pwd)"
cache="$root/target/php-stub-tool"
extension="${1:?Usage: generate.sh EXTENSION OUTPUT}"
output="${2:?Usage: generate.sh EXTENSION OUTPUT}"

if [[ "$(uname -s)" != Linux ]]; then
  echo "The pinned stub-generator bootstrap currently supports Linux only." >&2
  exit 1
fi

mkdir -p "$cache"
exec 9>"$cache/lock"
flock 9
fingerprint="$(
  { sha256sum "$tools/"*.patch "$tools/Cargo.lock" "$tools/generate.sh"
    rustc -vV
    php-config --version
    php-config --includes
  } | sha256sum | cut -d ' ' -f 1
)"
generator="$cache/install/bin/cargo-php"
work="$(mktemp -d "$cache/run.XXXXXX")"
trap 'rm -rf -- "$work"' EXIT

if [[ ! -x "$generator" || ! -f "$cache/fingerprint" || "$(cat "$cache/fingerprint")" != "$fingerprint" ]]; then
  fetch() {
    local crate="$1" checksum="$2"
    curl --fail --location --silent --show-error \
      "https://static.crates.io/crates/${crate%-*}/$crate.crate" -o "$work/$crate.crate"
    printf '%s  %s\n' "$checksum" "$work/$crate.crate" | sha256sum --check --quiet
    tar -xzf "$work/$crate.crate" -C "$work"
  }
  fetch cargo-php-0.2.1 756e642e0e50a78333d744a9d47fa8956aa621d3ef1b7916c4593d35a170ddd5
  fetch ext-php-rs-introspection-0.2.1 efa29f09b08e644d5dc1e76a318f758afc9dccbe92ccf84b4778e0433a78eecc
  patch --batch --fuzz=0 -d "$work/cargo-php-0.2.1" -p1 < "$tools/cargo-php.patch"
  patch --batch --fuzz=0 -d "$work/ext-php-rs-introspection-0.2.1" -p1 < "$tools/ext-php-rs-introspection.patch"
  cp "$tools/Cargo.lock" "$work/cargo-php-0.2.1/Cargo.lock"
  CARGO_TARGET_DIR="$cache/build" cargo install --locked --force \
    --path "$work/cargo-php-0.2.1" --root "$cache/install"
  printf '%s\n' "$fingerprint" > "$cache/fingerprint"
fi

"$generator" php stubs "$extension" -o "$work/stubs.php"
php -n -d error_reporting=-1 -d display_errors=stderr -l "$work/stubs.php"
php -n -d extension="$extension" "$tools/verify.php" snapshot > "$work/api.json"
php -n -d error_reporting=-1 "$tools/verify.php" check "$work/stubs.php" "$work/api.json"
mkdir -p "$(dirname -- "$output")"
cp "$work/stubs.php" "$output"
echo "Generated and checked $output"
