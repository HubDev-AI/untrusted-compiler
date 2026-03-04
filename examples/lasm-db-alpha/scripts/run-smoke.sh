#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

PORT="${SEC4_DB_ALPHA_PORT:-${SEC4_RT_LASM_DB_PORT:-8088}}"
DB_ADAPTER="${SEC4_DB_ALPHA_DB_ADAPTER:-${SEC4_RT_LASM_DB_ADAPTER:-records}}"
DB_BASE="${SEC4_DB_ALPHA_DB_BASE:-${SEC4_RT_LASM_DB_BASE:-$EXAMPLE_DIR/.lasm-db}}"
DB_QUERY_TEMPLATE="${SEC4_DB_ALPHA_QUERY_TEMPLATE:-SELECT%201}"
DB_QUERY_PARAMS="${SEC4_DB_ALPHA_QUERY_PARAMS:-%5B%5D}"
DB_QUERY_ONE_ROW_SCHEMA="${SEC4_DB_ALPHA_QUERY_ONE_ROW_SCHEMA:-7}"
SERVE_TIMEOUT_MS="${SEC4_DB_ALPHA_SERVE_TIMEOUT_MS:-${SEC4_RT_LASM_DB_SERVE_TIMEOUT_MS:-120000}}"
REQUEST_TIMEOUT_MS="${SEC4_DB_ALPHA_TIMEOUT_MS:-${SEC4_RT_LASM_DB_TIMEOUT_MS:-5000}}"
ASSERT="${ASSERT:-1}"

normalize_request_timeout() {
  if ! [[ "$REQUEST_TIMEOUT_MS" =~ ^[0-9]+$ ]] || [ "$REQUEST_TIMEOUT_MS" -eq 0 ]; then
    echo "SEC4_DB_ALPHA_TIMEOUT_MS must be a positive integer (milliseconds): '$REQUEST_TIMEOUT_MS'" >&2
    exit 1
  fi

  REQUEST_TIMEOUT_SECONDS=$(( (REQUEST_TIMEOUT_MS + 999) / 1000 ))
  if [ "$REQUEST_TIMEOUT_SECONDS" -lt 1 ]; then
    REQUEST_TIMEOUT_SECONDS=1
  fi
}

SEC4_PID=""
TMP_DIR="$(mktemp -d)"
LOG_FILE="$TMP_DIR/sec4-db-alpha-smoke.log"

cleanup() {
  if [ -n "${SEC4_PID}" ] && kill -0 "${SEC4_PID}" >/dev/null 2>&1; then
    kill "${SEC4_PID}" >/dev/null 2>&1 || true
    wait "${SEC4_PID}" >/dev/null 2>&1 || true
  fi

  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

usage() {
  cat <<'USAGE'
usage: run-smoke.sh \
  [--port <port>] \
  [--db-base <path>] \
  [--db-adapter <records|sqlite|postgres>] \
  [--query-template <url-encoded-sql>] \
  [--query-params <url-encoded-json>] \
  [--query-one-row-schema <schema_id>] \
  [--serve-timeout-ms <ms>] \
  [--request-timeout-ms <ms>] \
  [--skip-assert]

Runs a tiny LASM DB smoke flow (exec, exec-tx, query-one, list).

Environment:
  SEC4_DB_ALPHA_PORT                    default 8088
  SEC4_DB_ALPHA_DB_ADAPTER              default records
  SEC4_DB_ALPHA_DB_BASE                 default <project>/.lasm-db
  SEC4_DB_ALPHA_QUERY_TEMPLATE           default SELECT%201 (URL-encoded)
  SEC4_DB_ALPHA_QUERY_PARAMS             default %5B%5D
  SEC4_DB_ALPHA_QUERY_ONE_ROW_SCHEMA     default 7
  SEC4_DB_ALPHA_SERVE_TIMEOUT_MS         default 120000
  SEC4_DB_ALPHA_TIMEOUT_MS               default 5000
  # compatibility fallbacks:
  SEC4_RT_LASM_DB_PORT/SEC4_RT_LASM_DB_ADAPTER/SEC4_RT_LASM_DB_BASE
  SEC4_RT_LASM_DB_SERVE_TIMEOUT_MS/SEC4_RT_LASM_DB_TIMEOUT_MS
  SEC4_RT_LASM_DB_POSTGRES_DSN           required when db-adapter=postgres
  SEC4_RT_LASM_DB_POSTGRES_DSN_FILE      alternative when db-adapter=postgres
  SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH  alternative when db-adapter=postgres
USAGE
}

require_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "missing required command: $name" >&2
    exit 1
  fi
}

while [ "$#" -gt 0 ]; do
  case "$1" in
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
    --db-base)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      DB_BASE="$2"
      shift 2
      ;;
    --db-base=*)
      DB_BASE="${1#--db-base=}"
      shift
      ;;
    --query-template)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      DB_QUERY_TEMPLATE="$2"
      shift 2
      ;;
    --query-template=*)
      DB_QUERY_TEMPLATE="${1#--query-template=}"
      shift
      ;;
    --query-params)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      DB_QUERY_PARAMS="$2"
      shift 2
      ;;
    --query-params=*)
      DB_QUERY_PARAMS="${1#--query-params=}"
      shift
      ;;
    --query-one-row-schema)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      DB_QUERY_ONE_ROW_SCHEMA="$2"
      shift 2
      ;;
    --query-one-row-schema=*)
      DB_QUERY_ONE_ROW_SCHEMA="${1#--query-one-row-schema=}"
      shift
      ;;
    --serve-timeout-ms)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      SERVE_TIMEOUT_MS="$2"
      shift 2
      ;;
    --serve-timeout-ms=*)
      SERVE_TIMEOUT_MS="${1#--serve-timeout-ms=}"
      shift
      ;;
    --request-timeout-ms)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      REQUEST_TIMEOUT_MS="$2"
      normalize_request_timeout
      shift 2
      ;;
    --request-timeout-ms=*)
      REQUEST_TIMEOUT_MS="${1#--request-timeout-ms=}"
      normalize_request_timeout
      shift
      ;;
    --skip-assert)
      ASSERT="0"
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

normalize_request_timeout

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
    if [ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}" ]; then
      run_args+=(--db-postgres-dsn-file "$SEC4_RT_LASM_DB_POSTGRES_DSN_FILE")
    elif [ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN:-}" ]; then
      run_args+=(--db-postgres-dsn "$SEC4_RT_LASM_DB_POSTGRES_DSN")
    elif [ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH:-}" ]; then
      run_args+=(--db-postgres-dsn-file "$SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH")
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
  local path="$2"
  local body_file="$TMP_DIR/${name}.body"
  local code

  code="$(curl --silent --show-error --output "$body_file" --write-out '%{http_code}' \
    --request GET \
    --max-time "$REQUEST_TIMEOUT_SECONDS" \
    "http://127.0.0.1:${PORT}${path}" || true)"

  if [ "$code" != "200" ]; then
    echo "${name}: expected 200, got ${code}" >&2
    echo "--- response body" >&2
    cat "$body_file" >&2
    echo "--- service log" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi

  if [ "$ASSERT" = "1" ]; then
    echo "[ok] ${name}"
  else
    echo "[run] ${name}"
  fi

  cat "$body_file"
}

post() {
  local name="$1"
  local path="$2"
  local body_file="$TMP_DIR/${name}.body"
  local code

  code="$(curl --silent --show-error --output "$body_file" --write-out '%{http_code}' \
    --request POST \
    --max-time "$REQUEST_TIMEOUT_SECONDS" \
    "http://127.0.0.1:${PORT}${path}" || true)"

  if [ "$code" != "200" ] && [ "$code" != "201" ]; then
    echo "${name}: expected 200/201, got ${code}" >&2
    echo "--- response body" >&2
    cat "$body_file" >&2
    echo "--- service log" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi

  if [ "$ASSERT" = "1" ]; then
    echo "[ok] ${name}"
  else
    echo "[run] ${name}"
  fi

  cat "$body_file"
}

echo "LASM DB alpha smoke: adapter=${DB_ADAPTER} port=${PORT} db-base=${DB_BASE}"

{
  cargo run -p sec4 -- run "${run_args[@]}" >"$LOG_FILE" 2>&1
} &
SEC4_PID=$!

for _ in $(seq 1 120); do
  if ! kill -0 "$SEC4_PID" >/dev/null 2>&1; then
    echo "sec4 process exited before ready" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi

  if curl --silent --show-error --output /dev/null --max-time 1 "http://127.0.0.1:${PORT}/" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

post db-exec "/db/exec?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}"
post db-exec-tx "/db/exec-tx?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}"
dispatch db-query "/db/query-one?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}&row_schema=${DB_QUERY_ONE_ROW_SCHEMA}"
dispatch db-list "/db/records"

if [ "$DB_ADAPTER" = "records" ] || [ "$DB_ADAPTER" = "records.log" ]; then
  if [ ! -f "$DB_BASE/records.log" ]; then
    echo "records adapter expected $DB_BASE/records.log" >&2
    exit 1
  fi

  if [ "$ASSERT" = "1" ]; then
    echo "[ok] records log exists: $DB_BASE/records.log"
    echo "records.tail:"
    tail -n 3 "$DB_BASE/records.log"
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

echo "lasm-db-alpha smoke passed"
