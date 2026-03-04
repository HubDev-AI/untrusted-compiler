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

trim_whitespace() {
  printf '%s' "$1" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//'
}

read_env_value() {
  local file="$1"
  local key="$2"
  local line
  local name
  local value

  while IFS= read -r line; do
    line="$(trim_whitespace "${line%%$'\r'}")"
    [ -z "$line" ] && continue
    case "$line" in
      \#*) continue ;;
    esac

    if [[ "$line" == export\ * ]]; then
      line="$(trim_whitespace "${line#export }")"
    fi

    [[ "$line" != *"="* ]] && continue
    name="$(trim_whitespace "${line%%=*}")"
    [ "$name" != "$key" ] && continue

    value="${line#*=}"
    value="$(trim_whitespace "$value")"
    case "$value" in
      \"*\") value="${value#\"}"; value="${value%\"}" ;;
      \'*\') value="${value#\'}"; value="${value%\'}" ;;
    esac

    printf '%s' "$value"
    return 0
  done < "$file"

  return 1
}

resolve_candidate_file() {
  local path="$1"
  if [ -z "$path" ]; then
    return 1
  fi
  if [ -f "$path" ]; then
    printf '%s' "$path"
    return 0
  fi
  if [ -f "$EXAMPLE_DIR/$path" ]; then
    printf '%s' "$EXAMPLE_DIR/$path"
    return 0
  fi
  return 1
}

resolve_postgres_dsn_from_env_file() {
  local file="$1"
  local dsn
  dsn="$(read_env_value "$file" SEC4_DB_ALPHA_DB_POSTGRES_DSN || true)"
  if [ -n "$dsn" ]; then
    printf '%s' "$dsn"
    return 0
  fi

  dsn="$(read_env_value "$file" SEC4_RT_LASM_DB_POSTGRES_DSN || true)"
  if [ -z "$dsn" ]; then
    return 1
  fi
  printf '%s' "$dsn"
}

resolve_postgres_runtime_file() {
  local candidate_file
  local resolved_file
  local candidates=(
    "${SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE:-}"
    "${SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE:-}"
    "${SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE:-}"
    "${SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE:-}"
    "$SCRIPT_DIR/../../infra/local-postgres/.runtime.env"
    "$EXAMPLE_DIR/../../infra/local-postgres/.runtime.env"
  )

  for candidate_file in "${candidates[@]}"; do
    [ -z "$candidate_file" ] && continue
    resolved_file="$(resolve_candidate_file "$candidate_file" || true)"
    if [ -n "$resolved_file" ] && resolve_postgres_dsn_from_env_file "$resolved_file"; then
      return 0
    fi
  done

  return 1
}

resolve_postgres_dsn_file() {
  local candidate_file
  local resolved_file

  candidate_file="${SEC4_DB_ALPHA_POSTGRES_DSN_FILE:-${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}}"
  if [ -n "$candidate_file" ]; then
    resolved_file="$(resolve_candidate_file "$candidate_file" || true)"
    if [ -n "$resolved_file" ]; then
      printf '%s' "$resolved_file"
      return 0
    fi
  fi

  candidate_file="${SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH:-${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH:-}}"
  if [ -n "$candidate_file" ]; then
    resolved_file="$(resolve_candidate_file "$candidate_file" || true)"
    if [ -n "$resolved_file" ]; then
      printf '%s' "$resolved_file"
      return 0
    fi
  fi

  return 1
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

Runs a tiny LASM DB smoke flow (exec, exec-tx, exec-batch, write-and-query, query-one, list, restart).

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
  SEC4_DB_ALPHA_DB_POSTGRES_DSN          optional DSN literal for postgres
  SEC4_DB_ALPHA_POSTGRES_DSN_FILE        optional DSN file for postgres
  SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH   optional legacy DSN file path for postgres
  SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE
  SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE optional runtime env path
  SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE
  SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE optional runtime env fallback paths
  (fallback order: SEC4_DB_ALPHA_DB_POSTGRES_DSN / SEC4_DB_ALPHA_POSTGRES_DSN_FILE
   / SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH / SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE /
   SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE, then legacy SEC4_RT_LASM_* aliases)
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
    if [ -n "${SEC4_DB_ALPHA_DB_POSTGRES_DSN-}" ]; then
      run_args+=(--db-postgres-dsn "$SEC4_DB_ALPHA_DB_POSTGRES_DSN")
    elif [ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN-}" ]; then
      run_args+=(--db-postgres-dsn "$SEC4_RT_LASM_DB_POSTGRES_DSN")
    elif [ -n "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE-}" ] || [ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE-}" ] || [ -n "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH-}" ] || [ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH-}" ]; then
      dsn_file="$(resolve_postgres_dsn_file || true)"
      if [ -z "${dsn_file-}" ] || [ ! -f "$dsn_file" ]; then
        echo "postgres adapter selected but provided DSN file is missing: ${dsn_file:-<none>}" >&2
        usage
        exit 1
      fi
      run_args+=(--db-postgres-dsn-file "$dsn_file")
    elif [ -n "${SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE-}" ] || [ -n "${SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE-}" ] || [ -n "${SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE-}" ] || [ -n "${SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE-}" ]; then
      runtime_dsn="$(resolve_postgres_runtime_file)" || true
      if [ -n "${runtime_dsn-}" ]; then
        run_args+=(--db-postgres-dsn "$runtime_dsn")
      else
        echo "SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE/SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE is set but no DSN was read" >&2
        usage
        exit 1
      fi
    elif runtime_dsn="$(resolve_postgres_runtime_file)"; then
      run_args+=(--db-postgres-dsn "$runtime_dsn")
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

start_service() {
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

    if curl --silent --show-error --output /dev/null --max-time 1 "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.2
  done

  echo "sec4 process did not become ready" >&2
  cat "$LOG_FILE" >&2
  exit 1
}

stop_service() {
  if [ -n "${SEC4_PID-}" ] && kill -0 "${SEC4_PID}" >/dev/null 2>&1; then
    kill "${SEC4_PID}" >/dev/null 2>&1 || true
    wait "${SEC4_PID}" >/dev/null 2>&1 || true
  fi
  SEC4_PID=""
}

extract_records_count() {
  local body="$1"
  local compact
  compact="$(printf '%s' "$body" | tr -d '\n\r')"
  local count

  if [[ "$compact" =~ \"count\"[[:space:]]*:[[:space:]]*([0-9]+) ]]; then
    echo "${BASH_REMATCH[1]}"
    return 0
  fi

  echo ""
  return 1
}

assert_status() {
  local response_body="$1"
  local expected_rows="$2"
  local request_label="$3"

  if [[ "$response_body" != *"\"rowSchema\":${expected_rows}"* ]]; then
    echo "${request_label}: expected response rowSchema=${expected_rows}" >&2
    echo "--- response body" >&2
    cat <<< "$response_body" >&2
    echo "--- service log" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi
}

assert_equal_numbers() {
  local left="$1"
  local right="$2"
  local message="$3"

  if [ "$left" -ne "$right" ]; then
    echo "${message}: expected ${left} to equal ${right}" >&2
    exit 1
  fi
}

assert_at_least_numbers() {
  local observed="$1"
  local minimum="$2"
  local message="$3"

  if [ "$observed" -lt "$minimum" ]; then
    echo "${message}: expected ${observed} to be >= ${minimum}" >&2
    exit 1
  fi
}

echo "LASM DB alpha smoke: adapter=${DB_ADAPTER} port=${PORT} db-base=${DB_BASE}"

start_service

post db-exec "/db/exec?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}"
post db-exec-tx "/db/exec-tx?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}"
records_after_exec_tx="$(dispatch db-list-before-batch "/db/records")"
count_after_exec_tx="$(extract_records_count "$records_after_exec_tx")"
if [ -z "$count_after_exec_tx" ]; then
  echo "missing records count after exec+tx request in /db/records response" >&2
  echo "--- response body" >&2
  echo "$records_after_exec_tx" >&2
  exit 1
fi

dispatch db-exec-batch "/db/exec-batch?template_a=${DB_QUERY_TEMPLATE}&params_a=${DB_QUERY_PARAMS}&template_b=${DB_QUERY_TEMPLATE}&params_b=${DB_QUERY_PARAMS}"
records_after_exec_batch="$(dispatch db-list-after-batch "/db/records")"
count_after_exec_batch="$(extract_records_count "$records_after_exec_batch")"
if [ -z "$count_after_exec_batch" ]; then
  echo "missing records count after exec-batch request in /db/records response" >&2
  echo "--- response body" >&2
  echo "$records_after_exec_batch" >&2
  exit 1
fi

assert_equal_numbers "$count_after_exec_batch" "$((count_after_exec_tx + 2))" "exec-batch should append exactly two records"

write_and_query_response="$(post db-write-and-query "/db/write-and-query?write_template=${DB_QUERY_TEMPLATE}&write_params=${DB_QUERY_PARAMS}&query_template=${DB_QUERY_TEMPLATE}&query_params=${DB_QUERY_PARAMS}&row_schema=${DB_QUERY_ONE_ROW_SCHEMA}")"
records_after_write_and_query="$(dispatch db-list-after-write-and-query "/db/records")"
count_after_write_and_query="$(extract_records_count "$records_after_write_and_query")"
if [ -z "$count_after_write_and_query" ]; then
  echo "missing records count after write-and-query request in /db/records response" >&2
  echo "--- response body" >&2
  echo "$records_after_write_and_query" >&2
  exit 1
fi

assert_status "$write_and_query_response" "$DB_QUERY_ONE_ROW_SCHEMA" "db-write-and-query"
assert_at_least_numbers "$count_after_write_and_query" "$((count_after_exec_batch + 1))" "write-and-query should append at least one record"

query_body="$(dispatch db-query "/db/query-one?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}&row_schema=${DB_QUERY_ONE_ROW_SCHEMA}")"
assert_status "$query_body" "$DB_QUERY_ONE_ROW_SCHEMA" "db-query"
records_after_query="$(dispatch db-list "/db/records")"
count_after_query="$(extract_records_count "$records_after_query")"
if [ -z "$count_after_query" ]; then
  echo "missing records count before restart in /db/records response" >&2
  echo "--- response body" >&2
  echo "$records_after_query" >&2
  exit 1
fi
echo "[ok] records count before restart: ${count_after_query}"

echo "restarting service for persistence check"
stop_service
start_service

query_body_after="$(dispatch db-query-after-restart "/db/query-one?template=${DB_QUERY_TEMPLATE}&params=${DB_QUERY_PARAMS}&row_schema=${DB_QUERY_ONE_ROW_SCHEMA}")"
assert_status "$query_body_after" "$DB_QUERY_ONE_ROW_SCHEMA" "db-query-after-restart"
records_after_restart="$(dispatch db-list-after-restart "/db/records")"
count_after_restart="$(extract_records_count "$records_after_restart")"
if [ -z "$count_after_restart" ]; then
  echo "missing records count after restart in /db/records response" >&2
  echo "--- response body" >&2
  echo "$records_after_restart" >&2
  exit 1
fi
assert_at_least_numbers "$count_after_restart" "$count_after_query" "persisted records count should not decrease after restart"

if [ "$DB_ADAPTER" = "records" ] || [ "$DB_ADAPTER" = "records.log" ]; then
  if [ ! -f "$DB_BASE/records.log" ]; then
    echo "records adapter expected $DB_BASE/records.log immediately after startup" >&2
    exit 1
  fi
  if [ "$ASSERT" = "1" ]; then
    echo "[ok] records log bootstrap: $DB_BASE/records.log"
  fi

  if [ "$ASSERT" = "1" ]; then
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
