#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"
root_dir="$(cd "$service_dir/../.." && pwd)"
payload="$root_dir/spec/payloads/user_4kb.json"
port="${BENCH_SMOKE_PORT:-18084}"

if [ ! -f "$payload" ]; then
  echo "payload fixture missing: $payload" >&2
  exit 1
fi

"$service_dir/build.sh"

log_file="$service_dir/.smoke.log"
PORT="$port" "$service_dir/sec4-bench-server" >"$log_file" 2>&1 &
pid=$!
cleanup() {
  if kill -0 "$pid" >/dev/null 2>&1; then
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

ping_tmp="/tmp/sec4-smoke-ping.txt"
: >"$ping_tmp"
ready="false"
for _ in $(seq 1 180); do
  if curl -fsS "http://127.0.0.1:${port}/ping" >"$ping_tmp" 2>/dev/null; then
    if [ "$(cat "$ping_tmp" 2>/dev/null || true)" = "ok" ]; then
      ready="true"
      break
    fi
  fi
  if ! kill -0 "$pid" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

if [ "$ready" != "true" ]; then
  echo "sec4 smoke ping failed" >&2
  exit 1
fi

decode_status="$(curl -sS -o /tmp/sec4-smoke-decode.json -w '%{http_code}' \
  -H 'content-type: application/json' \
  --data-binary "@$payload" \
  http://127.0.0.1:${port}/decode)"
if [ "$decode_status" != "200" ]; then
  echo "sec4 smoke /decode expected 200, got $decode_status" >&2
  exit 1
fi
if ! grep -q '"ok":true' /tmp/sec4-smoke-decode.json; then
  echo "sec4 smoke /decode missing ok=true" >&2
  exit 1
fi

user_id="$(jq -r '.id' "$payload")"

users_post_status="$(curl -sS -o /tmp/sec4-smoke-users-post.json -w '%{http_code}' \
  -H 'content-type: application/json' \
  --data-binary "@$payload" \
  http://127.0.0.1:${port}/users)"
if [ "$users_post_status" != "201" ]; then
  echo "sec4 smoke /users POST expected 201, got $users_post_status" >&2
  exit 1
fi

users_get_status="$(curl -sS -o /tmp/sec4-smoke-users-get.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/users/$user_id")"
if [ "$users_get_status" != "200" ]; then
  echo "sec4 smoke /users/:id expected 200, got $users_get_status" >&2
  exit 1
fi
if ! grep -q '"email"' /tmp/sec4-smoke-users-get.json; then
  echo "sec4 smoke /users/:id missing email field" >&2
  exit 1
fi

db_exec_status="$(curl -sS -o /tmp/sec4-smoke-db-exec.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/db/hot-write")"
if [ "$db_exec_status" != "200" ]; then
  echo "sec4 smoke /db/hot-write expected 200, got $db_exec_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and .op == "exec"' /tmp/sec4-smoke-db-exec.json >/dev/null; then
  echo "sec4 smoke /db/hot-write expected {ok:true,op:\"exec\"}" >&2
  exit 1
fi

db_exec_tx_status="$(curl -sS -o /tmp/sec4-smoke-db-exec-tx.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/db/hot-write-tx")"
if [ "$db_exec_tx_status" != "200" ]; then
  echo "sec4 smoke /db/hot-write-tx expected 200, got $db_exec_tx_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and .op == "execTx"' /tmp/sec4-smoke-db-exec-tx.json >/dev/null; then
  echo "sec4 smoke /db/hot-write-tx expected {ok:true,op:\"execTx\"}" >&2
  exit 1
fi

db_query_one_status="$(curl -sS -o /tmp/sec4-smoke-db-query-one.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/db/hot-query-one")"
if [ "$db_query_one_status" != "200" ]; then
  echo "sec4 smoke /db/hot-query-one expected 200, got $db_query_one_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and .record.op == "exec"' /tmp/sec4-smoke-db-query-one.json >/dev/null; then
  echo "sec4 smoke /db/hot-query-one expected {ok:true,record.op:\"exec\"}" >&2
  exit 1
fi

db_records_status="$(curl -sS -o /tmp/sec4-smoke-db-records.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/db/records")"
if [ "$db_records_status" != "200" ]; then
  echo "sec4 smoke /db/records expected 200, got $db_records_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and .count >= 2' /tmp/sec4-smoke-db-records.json >/dev/null; then
  echo "sec4 smoke /db/records expected {ok:true,count>=2}" >&2
  exit 1
fi

echo "sec4 service smoke test passed"
