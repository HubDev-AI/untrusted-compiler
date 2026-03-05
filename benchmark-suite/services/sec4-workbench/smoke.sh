#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"
port="${BENCH_SMOKE_PORT:-18092}"
if [ -n "${BENCH_SMOKE_DB_BASE:-}" ]; then
  db_base="$BENCH_SMOKE_DB_BASE"
  cleanup_db_base="false"
  mkdir -p "$db_base"
else
  db_base="$(mktemp -d "/tmp/sec4-workbench-db.XXXXXX")"
  cleanup_db_base="true"
fi

log_file="$service_dir/.smoke.log"
SEC4_RT_DB_BASE="$db_base" \
  cargo run -q -p sec4 -- run \
    --path "$service_dir" \
    --backend c \
    --port "$port" \
    --serve-timeout-ms 20000 \
    >"$log_file" 2>&1 &
pid=$!
cleanup() {
  if kill -0 "$pid" >/dev/null 2>&1; then
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  fi
  if [ "$cleanup_db_base" = "true" ] && [ -d "$db_base" ]; then
    rm -rf "$db_base"
  fi
}
trap cleanup EXIT

ready="false"
for _ in $(seq 1 180); do
  if curl -fsS "http://127.0.0.1:${port}/health" >/tmp/sec4-workbench-health.txt 2>/dev/null; then
    if [ "$(cat /tmp/sec4-workbench-health.txt 2>/dev/null || true)" = "ok" ]; then
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
  echo "sec4-workbench smoke health check failed" >&2
  exit 1
fi

auth_header='Authorization: Bearer token123'
run_id="$(date +%s%N)"

task_id="sec4-wb-task-${run_id}"
task2_id="sec4-wb-task-tx-${run_id}"
comment_id="sec4-wb-comment-${run_id}"

setup_status="$(curl -sS -o /tmp/sec4-workbench-setup.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/setup")"
if [ "$setup_status" != "200" ]; then
  echo "sec4-workbench /wb/setup expected 200, got $setup_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data | type == "number")' /tmp/sec4-workbench-setup.json >/dev/null; then
  echo "sec4-workbench /wb/setup response mismatch" >&2
  exit 1
fi

task_params="$(jq -nc \
  --arg id "$task_id" \
  --arg title "Task One" \
  --arg description "smoke create task" \
  --arg status "open" \
  --argjson priority 3 \
  --argjson created 1700000000000 \
  '[ $id, $title, $description, $status, $priority, $created ]')"
task_params_uri="$(printf '%s' "$task_params" | jq -sRr @uri)"
create_status="$(curl -sS -o /tmp/sec4-workbench-create.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/tasks?params=${task_params_uri}")"
if [ "$create_status" != "201" ]; then
  echo "sec4-workbench /wb/tasks expected 201, got $create_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data | type == "number")' /tmp/sec4-workbench-create.json >/dev/null; then
  echo "sec4-workbench /wb/tasks response mismatch" >&2
  exit 1
fi

task2_params="$(jq -nc \
  --arg id "$task2_id" \
  --arg title "Task Two" \
  --arg description "smoke tx task" \
  --arg status "in_progress" \
  --argjson priority 4 \
  --argjson created 1700000000001 \
  '[ $id, $title, $description, $status, $priority, $created ]')"
comment_params="$(jq -nc \
  --arg id "$comment_id" \
  --arg task_id "$task2_id" \
  --arg body "initial comment" \
  --argjson created 1700000000002 \
  '[ $id, $task_id, $body, $created ]')"
task2_params_uri="$(printf '%s' "$task2_params" | jq -sRr @uri)"
comment_params_uri="$(printf '%s' "$comment_params" | jq -sRr @uri)"
create_tx_status="$(curl -sS -o /tmp/sec4-workbench-create-tx.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/tasks/with-comment?task_params=${task2_params_uri}&comment_params=${comment_params_uri}")"
if [ "$create_tx_status" != "201" ]; then
  echo "sec4-workbench /wb/tasks/with-comment expected 201, got $create_tx_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data | type == "number")' /tmp/sec4-workbench-create-tx.json >/dev/null; then
  echo "sec4-workbench /wb/tasks/with-comment response mismatch" >&2
  exit 1
fi

get_status="$(curl -sS -o /tmp/sec4-workbench-get.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks/${task_id}?row_schema=1")"
if [ "$get_status" != "200" ]; then
  echo "sec4-workbench /wb/tasks/:id expected 200, got $get_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data | type == "number")' /tmp/sec4-workbench-get.json >/dev/null; then
  echo "sec4-workbench /wb/tasks/:id response mismatch" >&2
  exit 1
fi

list_params='["",20,0]'
list_params_uri="$(printf '%s' "$list_params" | jq -sRr @uri)"
list_status="$(curl -sS -o /tmp/sec4-workbench-list.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks?params=${list_params_uri}&row_schema=1")"
if [ "$list_status" != "200" ]; then
  echo "sec4-workbench /wb/tasks expected 200, got $list_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data | type == "number")' /tmp/sec4-workbench-list.json >/dev/null; then
  echo "sec4-workbench /wb/tasks response mismatch" >&2
  exit 1
fi

records_status="$(curl -sS -o /tmp/sec4-workbench-records.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/records")"
if [ "$records_status" != "200" ]; then
  echo "sec4-workbench /wb/records expected 200, got $records_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data | type == "number")' /tmp/sec4-workbench-records.json >/dev/null; then
  echo "sec4-workbench /wb/records response mismatch" >&2
  exit 1
fi

echo "sec4-workbench smoke test passed"
