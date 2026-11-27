#!/usr/bin/env bash
# Run a PHP server test using the built PHP extension.
# works with all OS environments
# sends blank.pdf and expects the result 10.
# expects the PHP extension to be built and the PDF file fixture blank.pdf
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"

OS="$(uname -s)"
# Windows shell environments report their runtime rather than "Windows".
case "$OS" in
  MINGW*|MSYS*|CYGWIN*) OS="Windows" ;;
esac

case "$OS" in
  Linux) extension="$root/target/debug/libbernard_ledit_php.so" ;;
  Darwin) extension="$root/target/debug/libbernard_ledit_php.dylib" ;;
  Windows) extension="$root/target/debug/bernard_ledit_php.dll" ;;
  *) echo "Unsupported platform for PHP server test" >&2; exit 1 ;;
esac

if [ ! -f "$extension" ]; then
  echo "PHP extension not found: $extension. Run 'just php build' first." >&2
  exit 1
fi
fixture="$root/tests/data/file_types/pdf/blank.pdf"
if [ ! -f "$fixture" ]; then
  echo "PDF fixture not found: $fixture. Initialize the tests/data submodule." >&2
  exit 1
fi

router="$root/bindings/php/tests/server.php"
case "$OS" in
  Windows)
    extension="$(cygpath -m "$extension")"
    router="$(cygpath -m "$router")"
    ;;
esac

port="${PHP_SERVER_TEST_PORT:-18765}"
log="$(mktemp)"
php -n -d display_errors=0 -d log_errors=1 -d "extension=$extension" \
  -S "127.0.0.1:$port" "$router" > "$log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true; wait "$server" 2>/dev/null || true; cat "$log"; rm -f "$log"' EXIT

response="$(curl --fail-with-body --silent --show-error --noproxy '*' \
  --retry 20 --retry-connrefused --retry-delay 1 --max-time 3 \
  --header 'Content-Type: application/pdf' --data-binary @- \
  "http://127.0.0.1:$port/" < "$fixture")"
kill -0 "$server"
jq -e --argjson size "$(wc -c < "$fixture")" \
  '.size == $size and .pages == 10' <<< "$response"
