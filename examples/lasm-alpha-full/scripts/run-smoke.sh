#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

PORT="${SEC4_ALPHA_FULL_PORT:-${SEC4_RT_LASM_DB_PORT:-8088}}"
DB_BASE="${SEC4_ALPHA_FULL_DB_BASE:-${SEC4_RT_LASM_DB_BASE:-$EXAMPLE_DIR/.lasm-db}}"
DB_ADAPTER="${SEC4_ALPHA_FULL_DB_ADAPTER:-${SEC4_RT_LASM_DB_ADAPTER:-records}}"
DB_QUERY_TEMPLATE="${SEC4_ALPHA_FULL_QUERY_TEMPLATE:-SELECT%201}"
DB_QUERY_PARAMS="${SEC4_ALPHA_FULL_QUERY_PARAMS:-%5B%5D}"
DB_QUERY_ONE_ROW_SCHEMA="${SEC4_ALPHA_FULL_QUERY_ONE_ROW_SCHEMA:-7}"
REQUEST_TIMEOUT_MS="${SEC4_ALPHA_FULL_TIMEOUT_MS:-5000}"
AUTH_HEADER="${SEC4_ALPHA_FULL_AUTH_HEADER:-Authorization: Bearer token123}"
POSTGRES_DSN="${SEC4_ALPHA_FULL_POSTGRES_DSN:-${SEC4_DB_ALPHA_DB_POSTGRES_DSN:-${SEC4_DB_ALPHA_POSTGRES_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-}}}}"
POSTGRES_DSN_FILE="${SEC4_ALPHA_FULL_POSTGRES_DSN_FILE:-${SEC4_DB_ALPHA_POSTGRES_DSN_FILE:-${SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH:-${SEC4_ALPHA_FULL_POSTGRES_DSN_FILE_PATH:-${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}}}}}}"
POSTGRES_RUNTIME_ENV_FILE="${SEC4_ALPHA_FULL_POSTGRES_RUNTIME_ENV_FILE:-${SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE:-${SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE:-${SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE:-}}}}"
ASSERT="${ASSERT:-1}"

HELP_MSG="Usage: ./scripts/run-smoke.sh [--port <port>] [--db-base <path>] [--db-adapter <records|sqlite|postgres>] [--skip-assert]\n\nEnvironment:\n  SEC4_ALPHA_FULL_PORT (or SEC4_RT_LASM_DB_PORT)\n  SEC4_ALPHA_FULL_DB_BASE (or SEC4_RT_LASM_DB_BASE)\n  SEC4_ALPHA_FULL_DB_ADAPTER (or SEC4_RT_LASM_DB_ADAPTER)\n  SEC4_ALPHA_FULL_QUERY_TEMPLATE\n  SEC4_ALPHA_FULL_QUERY_PARAMS\n  SEC4_ALPHA_FULL_QUERY_ONE_ROW_SCHEMA\n  SEC4_ALPHA_FULL_TIMEOUT_MS\n  SEC4_ALPHA_FULL_POSTGRES_DSN (or SEC4_DB_ALPHA_DB_POSTGRES_DSN / SEC4_DB_ALPHA_POSTGRES_DSN / SEC4_RT_LASM_DB_POSTGRES_DSN)\n  SEC4_ALPHA_FULL_POSTGRES_DSN_FILE (or SEC4_DB_ALPHA_POSTGRES_DSN_FILE / SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH)\n  SEC4_ALPHA_FULL_POSTGRES_RUNTIME_ENV_FILE (or SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE)"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --port)
      PORT="$2"
      shift 2
      ;;
    --db-base)
      DB_BASE="$2"
      shift 2
      ;;
    --db-adapter)
      DB_ADAPTER="$2"
      shift 2
      ;;
    --skip-assert)
      ASSERT="0"
      shift
      ;;
    -h|--help)
      echo -e "$HELP_MSG"
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      echo -e "$HELP_MSG" >&2
      exit 2
      ;;
  esac

done

if ! [[ "$REQUEST_TIMEOUT_MS" =~ ^[0-9]+$ ]] || [ "$REQUEST_TIMEOUT_MS" -eq 0 ]; then
  echo "SEC4_ALPHA_FULL_TIMEOUT_MS must be a positive integer" >&2
  exit 1
fi
REQUEST_TIMEOUT_SECONDS=$(( (REQUEST_TIMEOUT_MS + 999) / 1000 ))
[ "$REQUEST_TIMEOUT_SECONDS" -lt 1 ] && REQUEST_TIMEOUT_SECONDS=1

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "missing required command: $1" >&2; exit 1; }
}

read_dsn_file_value() {
  local file="$1"
  [ -f "$file" ] || return 1

  local dsn=""
  dsn="$(sed -nE 's/^[[:space:]]*(export[[:space:]]+)?SEC4_DB_ALPHA_DB_POSTGRES_DSN[[:space:]]*=//p' "$file")"
  dsn="${dsn%%[$'\n']*}"
  if [ -z "$dsn" ]; then
    dsn="$(sed -nE 's/^[[:space:]]*(export[[:space:]]+)?SEC4_DB_ALPHA_POSTGRES_DSN[[:space:]]*=//p' "$file")"
    dsn="${dsn%%[$'\n']*}"
  fi
  if [ -z "$dsn" ]; then
    dsn="$(sed -nE 's/^[[:space:]]*(export[[:space:]]+)?SEC4_RT_LASM_DB_POSTGRES_DSN[[:space:]]*=//p' "$file")"
    dsn="${dsn%%[$'\n']*}"
  fi

  dsn="${dsn#\"}"
  dsn="${dsn%\"}"
  dsn="${dsn#\'}"
  dsn="${dsn%\'}"
  dsn="${dsn//[[:space:]]/}"

  [ -n "$dsn" ] && printf '%s' "$dsn"
}

run_args=(--path "$EXAMPLE_DIR" --backend lasm --port "$PORT")

case "$DB_ADAPTER" in
  records|records.log)
    run_args+=(--db-base "$DB_BASE")
    ;;
  sqlite)
    run_args+=(--db-base "$DB_BASE" --db-adapter sqlite)
    ;;
  postgres)
    if [ -n "$POSTGRES_DSN" ]; then
      run_args+=(--db-adapter postgres --db-postgres-dsn "$POSTGRES_DSN")
    elif [ -n "$POSTGRES_DSN_FILE" ] && [ -f "$POSTGRES_DSN_FILE" ]; then
      run_args+=(--db-adapter postgres --db-postgres-dsn-file "$POSTGRES_DSN_FILE")
    elif [ -n "$POSTGRES_RUNTIME_ENV_FILE" ] && [ -f "$POSTGRES_RUNTIME_ENV_FILE" ]; then
      if runtime_dsn="$(read_dsn_file_value "$POSTGRES_RUNTIME_ENV_FILE")"; then
        run_args+=(--db-postgres-dsn "$runtime_dsn")
      else
        echo "postgres adapter selected but no DSN key found in $POSTGRES_RUNTIME_ENV_FILE" >&2
        exit 1
      fi
    else
      echo "postgres adapter selected but no DSN was provided" >&2
      exit 1
    fi
    ;;
  *)
    echo "unsupported db adapter: $DB_ADAPTER" >&2
    exit 1
    ;;
esac

require_cmd cargo
require_cmd curl

TMP_DIR="$(mktemp -d)"
LOG_FILE="$TMP_DIR/sec4-lasm-alpha-full-smoke.log"
SEC4_PID=""

cleanup() {
  if [ -n "$SEC4_PID" ] && kill -0 "$SEC4_PID" >/dev/null 2>&1; then
    kill "$SEC4_PID" >/dev/null 2>&1 || true
    wait "$SEC4_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

start_service() {
  {
    cargo run -p sec4 -- run "${run_args[@]}" >"$LOG_FILE" 2>&1
  } &
  SEC4_PID=$!

  for _ in $(seq 1 120); do
    if ! kill -0 "$SEC4_PID" >/dev/null 2>&1; then
      echo "sec4 exited before startup" >&2
      cat "$LOG_FILE" >&2
      exit 1
    fi
    if curl --silent --show-error --output /dev/null --max-time 1 \
      --header "$AUTH_HEADER" "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.2
  done

  echo "sec4 did not become ready" >&2
  cat "$LOG_FILE" >&2
  exit 1
}

stop_service() {
  if [ -n "$SEC4_PID" ] && kill -0 "$SEC4_PID" >/dev/null 2>&1; then
    kill "$SEC4_PID" >/dev/null 2>&1 || true
    wait "$SEC4_PID" >/dev/null 2>&1 || true
  fi
}

request() {
  local method="$1"
  local path="$2"
  local body_file="$TMP_DIR/$(printf '%s' "$path" | tr '/?&=' '____').body"
  local code

  if [ "$method" = "POST" ]; then
    code="$(curl --silent --show-error --output "$body_file" --write-out '%{http_code}' \
      --header "$AUTH_HEADER" --request POST --max-time "$REQUEST_TIMEOUT_SECONDS" \
      "http://127.0.0.1:${PORT}${path}" || true)"
  else
    code="$(curl --silent --show-error --output "$body_file" --write-out '%{http_code}' \
      --header "$AUTH_HEADER" --request GET --max-time "$REQUEST_TIMEOUT_SECONDS" \
      "http://127.0.0.1:${PORT}${path}" || true)"
  fi

  if [ "$code" != "200" ] && [ "$code" != "201" ]; then
    echo "request failed ${path}: code=${code}" >&2
    echo "--- response" >&2
    cat "$body_file" >&2
    echo "--- service log" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi

  if [ "$ASSERT" = "1" ]; then
    echo "[ok] ${method} ${path}"
  fi

  cat "$body_file"
}

start_service

request GET "/health"
request POST "/db/exec?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}"
request POST "/db/exec-tx?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}"
request GET "/db/exec-batch?template_a=${DB_QUERY_TEMPLATE}&params_a=${DB_QUERY_PARAMS}&template_b=${DB_QUERY_TEMPLATE}&params_b=${DB_QUERY_PARAMS}"
request POST "/db/write-and-query?write_template=${DB_QUERY_TEMPLATE}&write_params=${DB_QUERY_PARAMS}&query_template=${DB_QUERY_TEMPLATE}&query_params=${DB_QUERY_PARAMS}&row_schema=${DB_QUERY_ONE_ROW_SCHEMA}"
request GET "/db/query-one?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}&row_schema=${DB_QUERY_ONE_ROW_SCHEMA}"
request GET "/db/records"

echo "restarting for persistence check"
stop_service
start_service
request GET "/db/query-one?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}&row_schema=${DB_QUERY_ONE_ROW_SCHEMA}"

if [ "$DB_ADAPTER" = "records" ] || [ "$DB_ADAPTER" = "records.log" ]; then
  if [ ! -f "$DB_BASE/records.log" ]; then
    echo "expected records artifact at $DB_BASE/records.log" >&2
    exit 1
  fi
fi

if [ "$ASSERT" = "1" ]; then
  echo "lasm-alpha-full smoke passed"
fi
