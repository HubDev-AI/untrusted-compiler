#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PORT="${SEC4_LASM_WORKBENCH_PORT:-8088}"
DB_ADAPTER="${SEC4_LASM_WORKBENCH_DB_ADAPTER:-records}"
DB_BASE="${SEC4_LASM_WORKBENCH_DB_BASE:-$EXAMPLE_DIR/.lasm-workbench-db}"
NETWORK_PORT="${SEC4_LASM_WORKBENCH_NETWORK_PORT:-19090}"
SERVE_TIMEOUT_MS="${SEC4_LASM_WORKBENCH_SERVE_TIMEOUT_MS:-120000}"
REQUEST_TIMEOUT_MS="${SEC4_LASM_WORKBENCH_TIMEOUT_MS:-5000}"
SKIP_NETWORK="${SKIP_NETWORK:-0}"
ASSERT="${ASSERT:-1}"

SEC4_PID=""
HTTP_PID=""
TMP_DIR="$(mktemp -d)"
SEC4_LOG="$TMP_DIR/sec4-workbench-smoke.log"

require_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "missing required command: $name" >&2
    exit 1
  fi
}

usage() {
  cat <<'USAGE'
usage: run-smoke.sh [--skip-network] [--skip-assert] [--port <port>] [--db-adapter <records|sqlite|postgres>]

Runs a local LASM workbench smoke sequence against one build/runtime instance.

Environment:
  SEC4_LASM_WORKBENCH_PORT               default 8088
  SEC4_LASM_WORKBENCH_DB_ADAPTER         default records
  SEC4_LASM_WORKBENCH_DB_BASE            default <project>/.lasm-workbench-db
  SEC4_LASM_WORKBENCH_NETWORK_PORT        default 19090
  SEC4_LASM_WORKBENCH_SERVE_TIMEOUT_MS    default 120000
  SEC4_LASM_WORKBENCH_TIMEOUT_MS          default 5000
  SEC4_DB_ALPHA_DB_POSTGRES_DSN (legacy alias SEC4_RT_LASM_DB_POSTGRES_DSN)
                                       required when adapter=postgres
  SEC4_DB_ALPHA_POSTGRES_DSN_FILE (legacy alias SEC4_RT_LASM_DB_POSTGRES_DSN_FILE)
                                       alternative when adapter=postgres
  SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH (legacy alias SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH)
                                       alternative when adapter=postgres
USAGE
}

cleanup() {
  if [ -n "$SEC4_PID" ] && kill -0 "$SEC4_PID" >/dev/null 2>&1; then
    kill "$SEC4_PID" >/dev/null 2>&1 || true
    wait "$SEC4_PID" >/dev/null 2>&1 || true
  fi

  if [ -n "$HTTP_PID" ] && kill -0 "$HTTP_PID" >/dev/null 2>&1; then
    kill "$HTTP_PID" >/dev/null 2>&1 || true
    wait "$HTTP_PID" >/dev/null 2>&1 || true
  fi

  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-network)
      SKIP_NETWORK=1
      shift
      ;;
    --skip-assert)
      ASSERT=0
      shift
      ;;
    --port)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      PORT="$2"
      shift 2
      ;;
    --port=*)
      PORT="${1#--port=}"
      shift
      ;;
    --db-adapter)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      DB_ADAPTER="$2"
      shift 2
      ;;
    --db-adapter=*)
      DB_ADAPTER="${1#--db-adapter=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      usage
      exit 2
      ;;
  esac
done

require_cmd cargo
require_cmd curl

run_args=(
  --path "$EXAMPLE_DIR"
  --backend lasm
  --port "$PORT"
  --serve-timeout-ms "$SERVE_TIMEOUT_MS"
)

case "$DB_ADAPTER" in
  records|records.log)
    mkdir -p "$DB_BASE"
    run_args+=(--db-base "$DB_BASE")
    ;;
  sqlite)
    mkdir -p "$DB_BASE"
    run_args+=(--db-base "$DB_BASE" --db-adapter sqlite)
    ;;
  postgres)
    run_args+=(--db-adapter postgres)
    if [ -n "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE:-${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}}" ]; then
      run_args+=(--db-postgres-dsn-file "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE:-${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}}")
    elif [ -n "${SEC4_DB_ALPHA_DB_POSTGRES_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-}}" ]; then
      run_args+=(--db-postgres-dsn "${SEC4_DB_ALPHA_DB_POSTGRES_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-}}")
    elif [ -n "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH:-${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH:-}}" ]; then
      run_args+=(--db-postgres-dsn-file "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH:-${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH:-}}")
    else
      echo "postgres adapter selected but no DSN was provided" >&2
      usage
      exit 1
    fi
    ;;
  *)
    echo "unsupported db adapter: $DB_ADAPTER" >&2
    exit 1
    ;;
esac

dispatch() {
  local name="$1"
  local url="$2"
  local body_file="$TMP_DIR/${name}.body"
  local code

  code="$(curl --silent --show-error --output "$body_file" --write-out '%{http_code}' \
    --request GET \
    --header 'Authorization: Bearer token123' \
    --max-time "$REQUEST_TIMEOUT_MS" \
    "http://127.0.0.1:${PORT}${url}" || true)"

  if [ "$code" != "200" ]; then
    echo "${name}: expected 200, got ${code}" >&2
    if [ -f "$body_file" ]; then
      echo "--- response body" >&2
      cat "$body_file" >&2
    fi
    echo "--- service log" >&2
    cat "$SEC4_LOG" >&2
    exit 1
  fi

  if [ "$ASSERT" = "1" ]; then
    echo "[ok] ${name}"
  else
    echo "[run] ${name}"
  fi
}

echo "LASM workbench smoke: adapter=${DB_ADAPTER} port=${PORT} db-base=${DB_BASE}"

SEC4_RUN_PREFIX=(cargo)
if [ "$SKIP_NETWORK" -eq 0 ]; then
  SEC4_RUN_PREFIX=(env SEC4_RT_ALLOW_INTERNAL_NET=1 cargo)
fi

if [ "$SKIP_NETWORK" -eq 0 ]; then
  require_cmd python3

  cat >"$TMP_DIR/lasm-workbench-http-handler.py" <<'PY'
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):  # noqa: N802
        body = b"ok"
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args, **_kwargs):
        return


if __name__ == "__main__":
    server = ThreadingHTTPServer(("127.0.0.1", int("${NETWORK_PORT}")), Handler)
    server.serve_forever()
PY

  python3 "$TMP_DIR/lasm-workbench-http-handler.py" >"$TMP_DIR/lasm-workbench-http.log" 2>&1 &
  HTTP_PID=$!

  sleep 0.2
fi

{
  cd "$EXAMPLE_DIR/.." && "${SEC4_RUN_PREFIX[@]}" run -p sec4 -- run "${run_args[@]}" >"$SEC4_LOG" 2>&1
} &
SEC4_PID=$!

echo "starting sec4 process (pid=$SEC4_PID)"

for _ in $(seq 1 120); do
  if ! kill -0 "$SEC4_PID" >/dev/null 2>&1; then
    echo "sec4 process exited before ready" >&2
    cat "$SEC4_LOG" >&2
    exit 1
  fi
  if curl --silent --show-error --output /dev/null --max-time 1 "http://127.0.0.1:${PORT}/" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

DB_QUERY_TEMPLATE='SELECT%201%20LIMIT%201'
DB_PARAMS='%5B%5D'
PUBLIC_URL_ENC='http%3A%2F%2F127.0.0.1%3A'"$NETWORK_PORT"'%2Fpub'
INTERNAL_URL_ENC='http%3A%2F%2F127.0.0.1%3A'"$NETWORK_PORT"'%2Finternal'

dispatch "health" "/"
dispatch "db-write" "/db/write?template=${DB_QUERY_TEMPLATE}&params=${DB_PARAMS}"
dispatch "db-write-tx" "/db/write-tx?template=${DB_QUERY_TEMPLATE}&params=${DB_PARAMS}"
dispatch "db-write-tx-batch" "/db/write-tx-batch"
dispatch "db-list" "/db/list"
dispatch "fs-roundtrip" "/fs/roundtrip?file=smoke.txt&value=hello"
dispatch "gates" "/gates?file=smoke.txt&public_url=http://example.com&internal_url=${INTERNAL_URL_ENC}&port_hint=8080&request_id=123e4567-e89b-12d3-a456-426614174000"

dispatch "meta" "/meta"

auto_db_query="/db/query?template=${DB_QUERY_TEMPLATE}&params=${DB_PARAMS}&row_schema=7"
dispatch "db-query" "$auto_db_query"

if [ "$SKIP_NETWORK" -eq 0 ]; then
  dispatch "net-public" "/net/public?url=${PUBLIC_URL_ENC}"
  dispatch "net-internal" "/net/internal?url=${INTERNAL_URL_ENC}"
fi

if [ "$DB_ADAPTER" = "records" ] || [ "$DB_ADAPTER" = "records.log" ]; then
  if [ ! -f "$DB_BASE/records.log" ]; then
    echo "records adapter expected $DB_BASE/records.log" >&2
    exit 1
  fi
  if [ "$ASSERT" = "1" ]; then
    echo "[ok] records log exists: $DB_BASE/records.log"
  fi
elif [ "$DB_ADAPTER" = "sqlite" ]; then
  if [ ! -f "$DB_BASE/records.sqlite3" ]; then
    echo "sqlite adapter expected $DB_BASE/records.sqlite3" >&2
    exit 1
  fi
  if [ "$ASSERT" = "1" ]; then
    echo "[ok] sqlite artifact exists: $DB_BASE/records.sqlite3"
  fi
else
  if [ "$ASSERT" = "1" ]; then
    echo "[ok] postgres adapter smoke skip local artifact check"
  fi
fi

echo "lasm-workbench smoke passed"
