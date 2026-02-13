#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")" && pwd)"
port="18082"

(cd "$root_dir" && PORT="$port" cargo run) >/tmp/sec4-rust-bench-smoke.log 2>&1 &
pid="$!"
trap 'kill "$pid" >/dev/null 2>&1 || true' EXIT

for _ in $(seq 1 50); do
  if curl -fsS "http://127.0.0.1:${port}/ping" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

ping_body="$(curl -sS "http://127.0.0.1:${port}/ping")"
if [ "$ping_body" != "ok" ]; then
  echo "ping failed: expected ok, got: $ping_body" >&2
  exit 1
fi

decode_status="$(curl -sS -o /tmp/sec4-rust-bench-decode.json -w '%{http_code}' \
  -X POST "http://127.0.0.1:${port}/decode" \
  -H 'Content-Type: application/json' \
  --data-binary "$(cat "$root_dir/../../spec/payloads/user_4kb.json")")"
if [ "$decode_status" != "200" ]; then
  echo "decode failed: expected 200, got: $decode_status" >&2
  exit 1
fi

if ! grep -q '"ok":true' /tmp/sec4-rust-bench-decode.json; then
  echo "decode payload missing ok=true" >&2
  exit 1
fi

echo "rust benchmark smoke passed"
