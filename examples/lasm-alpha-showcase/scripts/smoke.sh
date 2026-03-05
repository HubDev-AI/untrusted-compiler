#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

PORT="${SEC4_LASM_SHOWCASE_PORT:-${SEC4_RT_LASM_DB_PORT:-8090}}"
DB_ADAPTER="${SEC4_LASM_SHOWCASE_DB_ADAPTER:-${SEC4_DB_ALPHA_DB_ADAPTER:-records}}"
DB_BASE="${SEC4_LASM_SHOWCASE_DB_BASE:-${SEC4_DB_ALPHA_DB_BASE:-$EXAMPLE_DIR/.lasm-db}}"
TIMEOUT_MS="${SEC4_LASM_SHOWCASE_TIMEOUT_MS:-${SEC4_RT_LASM_DB_TIMEOUT_MS:-5000}}"
REQ_TIMEOUT_SECONDS="$(( (TIMEOUT_MS + 999) / 1000 ))"

if [ "$REQ_TIMEOUT_SECONDS" -lt 1 ]; then
  REQ_TIMEOUT_SECONDS=1
fi

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required command: $cmd" >&2
    exit 1
  fi
}

url_encode_default_gets() {
  # keep these fixed and readable for deterministic smoke checks
  printf '%s' "SELECT%201"
}

run_curl() {
  local method="$1"
  local path="$2"
  echo "==> $method $path"
  curl -sS --max-time "$REQ_TIMEOUT_SECONDS" -i -X "$method" \
    "http://127.0.0.1:$PORT$path"
}

run_query() {
  local method="$1"
  local path="$2"
  local query="$3"
  run_curl "$method" "$path?$query"
}

cleanup() {
  if [ -n "${SEC4_SHOWCASE_SERVER_PID:-}" ] && kill -0 "$SEC4_SHOWCASE_SERVER_PID" >/dev/null 2>&1; then
    kill "$SEC4_SHOWCASE_SERVER_PID" >/dev/null 2>&1 || true
    wait "$SEC4_SHOWCASE_SERVER_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

for cmd in curl cargo; do
  require_cmd "$cmd"
done

mkdir -p "$DB_BASE"

set +m
(
  SEC4_DB_ALPHA_DB_ADAPTER="$DB_ADAPTER" \
  SEC4_DB_ALPHA_DB_BASE="$DB_BASE" \
  SEC4_DB_ALPHA_TIMEOUT_MS="$TIMEOUT_MS" \
  cargo run -p sec4 -- run \
    --path "$EXAMPLE_DIR" \
    --backend lasm \
    --db-base "$DB_BASE" \
    --db-adapter "$DB_ADAPTER" \
    --port "$PORT"
) >/tmp/sec4-lasm-showcase-smoke.log 2>&1 &
SEC4_SHOWCASE_SERVER_PID=$!
set -m

sleep 2

echo "Server started (pid $SEC4_SHOWCASE_SERVER_PID) on http://127.0.0.1:$PORT"

run_curl GET "/"
run_curl GET "/status"
run_query POST "/db/exec" "template=$(url_encode_default_gets)&params=%5B%22alpha%22%5D"
run_query POST "/db/exec-tx" "template=$(url_encode_default_gets)&params=%5B%22beta%22%5D"
run_query POST "/db/write-and-query" "write_template=$(url_encode_default_gets)&write_params=%5B1%5D&query_template=$(url_encode_default_gets)&query_params=%5B1%5D&row_schema=7"
run_query GET "/db/query-one" "template=$(url_encode_default_gets)&params=%5B1%5D&row_schema=7"
run_curl GET "/db/list"
run_curl GET "/fs/roundtrip?file=note.txt&value=hello"
run_curl GET "/fs/inspect?file=note.txt"
run_query GET "/net/public" "url=https%3A%2F%2Fexample.com"

cleanup
unset SEC4_SHOWCASE_SERVER_PID
echo "Smoke completed"
