#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"
root_dir="$(cd "$service_dir/../.." && pwd)"
payload="$root_dir/spec/payloads/user_4kb.json"

if [ ! -f "$payload" ]; then
  echo "payload fixture missing: $payload" >&2
  exit 1
fi

"$service_dir/build.sh"

log_file="$service_dir/.smoke.log"
PORT=8080 "$service_dir/ailang-bench-server" >"$log_file" 2>&1 &
pid=$!
cleanup() {
  if kill -0 "$pid" >/dev/null 2>&1; then
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

for _ in $(seq 1 180); do
  if curl -fsS "http://127.0.0.1:8080/ping" >/tmp/ailang-smoke-ping.txt 2>/dev/null; then
    break
  fi
  sleep 0.1
done

if [ "$(cat /tmp/ailang-smoke-ping.txt 2>/dev/null || true)" != "ok" ]; then
  echo "ailang smoke ping failed" >&2
  exit 1
fi

decode_status="$(curl -sS -o /tmp/ailang-smoke-decode.json -w '%{http_code}' \
  -H 'content-type: application/json' \
  --data-binary "@$payload" \
  http://127.0.0.1:8080/decode)"
if [ "$decode_status" != "200" ]; then
  echo "ailang smoke /decode expected 200, got $decode_status" >&2
  exit 1
fi
if ! grep -q '"ok":true' /tmp/ailang-smoke-decode.json; then
  echo "ailang smoke /decode missing ok=true" >&2
  exit 1
fi

user_id="$(jq -r '.id' "$payload")"

users_post_status="$(curl -sS -o /tmp/ailang-smoke-users-post.json -w '%{http_code}' \
  -H 'content-type: application/json' \
  --data-binary "@$payload" \
  http://127.0.0.1:8080/users)"
if [ "$users_post_status" != "201" ]; then
  echo "ailang smoke /users POST expected 201, got $users_post_status" >&2
  exit 1
fi

users_get_status="$(curl -sS -o /tmp/ailang-smoke-users-get.json -w '%{http_code}' \
  "http://127.0.0.1:8080/users/$user_id")"
if [ "$users_get_status" != "200" ]; then
  echo "ailang smoke /users/:id expected 200, got $users_get_status" >&2
  exit 1
fi
if ! grep -q '"email"' /tmp/ailang-smoke-users-get.json; then
  echo "ailang smoke /users/:id missing email field" >&2
  exit 1
fi

echo "ailang service smoke test passed"
