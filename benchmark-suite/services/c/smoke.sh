#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/../.." && pwd)"
service_dir="$root_dir/services/c"
payload="$root_dir/spec/payloads/user_4kb.json"

if [ ! -f "$payload" ]; then
  echo "payload fixture missing: $payload" >&2
  exit 1
fi

cc -O2 -std=c11 "$service_dir/server.c" -o "$service_dir/c-bench-server"

log_file="$service_dir/.smoke.log"
PORT=8080 "$service_dir/c-bench-server" >"$log_file" 2>&1 &
pid=$!
cleanup() {
  if kill -0 "$pid" >/dev/null 2>&1; then
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

for _ in $(seq 1 120); do
  if curl -fsS "http://127.0.0.1:8080/ping" >/tmp/c-smoke-ping.txt 2>/dev/null; then
    break
  fi
  sleep 0.1
done

if [ "$(cat /tmp/c-smoke-ping.txt 2>/dev/null || true)" != "ok" ]; then
  echo "c smoke ping failed" >&2
  exit 1
fi

decode_status="$(curl -sS -o /tmp/c-smoke-decode.json -w '%{http_code}' \
  -H 'content-type: application/json' \
  --data-binary "@$payload" \
  http://127.0.0.1:8080/decode)"
if [ "$decode_status" != "200" ]; then
  echo "c smoke /decode expected 200, got $decode_status" >&2
  exit 1
fi
if ! grep -q '"ok":true' /tmp/c-smoke-decode.json; then
  echo "c smoke /decode missing ok=true" >&2
  exit 1
fi

user_id="$(jq -r '.id' "$payload")"

users_post_status="$(curl -sS -o /tmp/c-smoke-users-post.json -w '%{http_code}' \
  -H 'content-type: application/json' \
  --data-binary "@$payload" \
  http://127.0.0.1:8080/users)"
if [ "$users_post_status" != "201" ]; then
  echo "c smoke /users POST expected 201, got $users_post_status" >&2
  exit 1
fi
if ! grep -q '"userId"' /tmp/c-smoke-users-post.json; then
  echo "c smoke /users POST missing userId" >&2
  exit 1
fi

users_get_status="$(curl -sS -o /tmp/c-smoke-users-get.json -w '%{http_code}' \
  "http://127.0.0.1:8080/users/$user_id")"
if [ "$users_get_status" != "200" ]; then
  echo "c smoke /users/:id expected 200, got $users_get_status" >&2
  exit 1
fi
if ! grep -q '"email"' /tmp/c-smoke-users-get.json; then
  echo "c smoke /users/:id missing email field" >&2
  exit 1
fi

bad_id_status="$(curl -sS -o /tmp/c-smoke-users-bad.json -w '%{http_code}' \
  "http://127.0.0.1:8080/users/not-a-uuid")"
if [ "$bad_id_status" != "400" ]; then
  echo "c smoke /users/not-a-uuid expected 400, got $bad_id_status" >&2
  exit 1
fi

echo "c service smoke test passed"
