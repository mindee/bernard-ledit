#!/usr/bin/env bash
# Run after `just php build`: bash bindings/php/tools/test-server.sh
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
case "$(uname -s)" in
  Linux) extension="$root/target/debug/libbernard_ledit_php.so" ;;
  Darwin) extension="$root/target/debug/libbernard_ledit_php.dylib" ;;
  *) echo "PHP server test supports Linux and macOS only" >&2; exit 1 ;;
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

port="${PHP_SERVER_TEST_PORT:-18765}"
log="$(mktemp)"
php -n -d display_errors=0 -d log_errors=1 -d "extension=$extension" \
  -S "127.0.0.1:$port" "$root/bindings/php/tests/server.php" > "$log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true; wait "$server" 2>/dev/null || true; cat "$log"; rm -f "$log"' EXIT

response="$(curl --fail-with-body --silent --show-error --noproxy '*' \
  --retry 20 --retry-connrefused --retry-delay 1 --max-time 3 \
  --header 'Content-Type: application/pdf' --data-binary "@$fixture" \
  "http://127.0.0.1:$port/")"
kill -0 "$server"
jq -e --argjson size "$(wc -c < "$fixture")" \
  '.size == $size and .pages == 10' <<< "$response"
