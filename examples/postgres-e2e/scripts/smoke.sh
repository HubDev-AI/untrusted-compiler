#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$EXAMPLE_DIR/../.." && pwd)"
INFRA_DIR="$REPO_ROOT/infra/local-postgres"
ENV_FILE="$INFRA_DIR/.env"
RUNTIME_ENV_FILE="$INFRA_DIR/.runtime.env"
PORT="${PORT:-18080}"

fail() {
  echo "error: $1" >&2
  exit 1
}

resolve_postgres_dsn_file() {
  local dsn_file="$1"
  local dsn
  if [ -z "$dsn_file" ]; then
    return 1
  fi
  if [ ! -f "$dsn_file" ]; then
    return 1
  fi
  dsn="$(tr -d '\r\n' < "$dsn_file")"
  dsn="$(printf '%s' "$dsn" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
  if [ -z "$dsn" ]; then
    return 1
  fi
  printf '%s\n' "$dsn"
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  if ! grep -Fq "$needle" <<<"$haystack"; then
    echo "assertion failed: expected to find [$needle]" >&2
    echo "--- response ---" >&2
    echo "$haystack" >&2
    echo "---------------" >&2
    exit 1
  fi
}

if [ ! -f "$ENV_FILE" ]; then
  fail "missing $ENV_FILE; run infra/local-postgres/scripts/up.sh first"
fi

set -a
# shellcheck source=/dev/null
source "$ENV_FILE"
if [ -f "$RUNTIME_ENV_FILE" ]; then
  # shellcheck source=/dev/null
  source "$RUNTIME_ENV_FILE"
fi
set +a

export SEC4_RT_LASM_DB_ADAPTER=postgres
resolved_dsn="${SEC4_DB_ALPHA_DB_POSTGRES_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-}}"
resolved_dsn="$(printf '%s' "$resolved_dsn" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
if [ -z "$resolved_dsn" ]; then
  for dsn_file in \
    "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE:-}" \
    "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}" \
    "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH:-}" \
    "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH:-}"
  do
    resolved_dsn="$(resolve_postgres_dsn_file "$dsn_file" || true)"
    if [ -n "$resolved_dsn" ]; then
      break
    fi
  done
fi
if [ -z "$resolved_dsn" ]; then
  resolved_dsn="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:${PG_PORT:-5432}/${POSTGRES_DB}?sslmode=disable"
fi
SEC4_DB_ALPHA_DB_POSTGRES_DSN="$resolved_dsn"
SEC4_RT_LASM_DB_POSTGRES_DSN="$resolved_dsn"
export SEC4_DB_ALPHA_DB_POSTGRES_DSN
export SEC4_RT_LASM_DB_POSTGRES_DSN

TMP_DIR="$(mktemp -d)"
LOG_FILE="$TMP_DIR/sec4-postgres-e2e.log"
SEC4_PID=""

cleanup() {
  if [ -n "$SEC4_PID" ] && kill -0 "$SEC4_PID" >/dev/null 2>&1; then
    kill "$SEC4_PID" >/dev/null 2>&1 || true
    wait "$SEC4_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cargo run -p sec4 -- run \
  --path "$EXAMPLE_DIR" \
  --backend lasm \
  --port "$PORT" \
  --serve-timeout-ms 120000 >"$LOG_FILE" 2>&1 &
SEC4_PID="$!"

for _ in $(seq 1 120); do
  if ! kill -0 "$SEC4_PID" >/dev/null 2>&1; then
    cat "$LOG_FILE" >&2
    fail "sec4 run exited before smoke requests"
  fi
  if curl -sS "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

CREATE_TABLE_TEMPLATE='CREATE%20TABLE%20IF%20NOT%20EXISTS%20demo_users%20%28id%20SERIAL%20PRIMARY%20KEY%2C%20email%20TEXT%20NOT%20NULL%20UNIQUE%29'
INSERT_USER_TEMPLATE='INSERT%20INTO%20demo_users%20%28email%29%20VALUES%20%28%241%29%20ON%20CONFLICT%20%28email%29%20DO%20NOTHING'
SELECT_USER_TEMPLATE='SELECT%20id%2C%20email%20FROM%20demo_users%20WHERE%20email%20%3D%20%241'
USER_PARAMS='%5B%22alice%40example.com%22%5D'

exec_response="$(curl -sS -i -X POST "http://127.0.0.1:${PORT}/db/exec?template=${CREATE_TABLE_TEMPLATE}&params=0")"
assert_contains "$exec_response" "HTTP/1.1 200"
assert_contains "$exec_response" '"ok":true'

exec_tx_response="$(curl -sS -i -X POST "http://127.0.0.1:${PORT}/db/exec-tx?template=${INSERT_USER_TEMPLATE}&params=${USER_PARAMS}")"
assert_contains "$exec_tx_response" "HTTP/1.1 200"
assert_contains "$exec_tx_response" '"ok":true'

query_one_response="$(curl -sS -i "http://127.0.0.1:${PORT}/db/query-one?template=${SELECT_USER_TEMPLATE}&params=${USER_PARAMS}&row_schema=7")"
assert_contains "$query_one_response" "HTTP/1.1 200"
assert_contains "$query_one_response" '"rowObject":{'
assert_contains "$query_one_response" "alice@example.com"

list_response="$(curl -sS -i "http://127.0.0.1:${PORT}/db/records")"
assert_contains "$list_response" "HTTP/1.1 200"
if grep -Fq '"adapter":"postgres"' <<<"$list_response"; then
  assert_contains "$list_response" '"dbCache":{'
  assert_contains "$list_response" '"postgresStatementCount":'
  assert_contains "$list_response" '"postgresPlaceholderCount":'
else
  assert_contains "$list_response" '"schema":"DbListRecordsResponse"'
fi

echo "postgres e2e smoke passed"
