#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"
port="${BENCH_SMOKE_PORT:-18088}"
if [ -n "${BENCH_SMOKE_DB_BASE:-}" ]; then
  db_base="$BENCH_SMOKE_DB_BASE"
  cleanup_db_base="false"
  mkdir -p "$db_base"
else
  db_base="$(mktemp -d "/tmp/sec4-lasm-workbench-db.XXXXXX")"
  cleanup_db_base="true"
fi

log_file="$service_dir/.smoke.log"
cargo run -q -p sec4 -- run \
  --path "$service_dir" \
  --backend lasm \
  --db-adapter sqlite \
  --db-base "$db_base" \
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
  if curl -fsS "http://127.0.0.1:${port}/health" >/tmp/sec4-lasm-workbench-health.txt 2>/dev/null; then
    if [ "$(cat /tmp/sec4-lasm-workbench-health.txt 2>/dev/null || true)" = "ok" ]; then
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
  echo "sec4-lasm-workbench smoke health check failed" >&2
  exit 1
fi

auth_header='Authorization: Bearer token123'
task_id="wb-task-1"
task2_id="wb-task-2"
comment_id="wb-comment-1"

setup_status="$(curl -sS -o /tmp/sec4-lasm-workbench-setup.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/setup")"
if [ "$setup_status" != "200" ]; then
  echo "sec4-lasm-workbench smoke /wb/setup expected 200, got $setup_status" >&2
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
create_status="$(curl -sS -o /tmp/sec4-lasm-workbench-create.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/tasks?params=${task_params_uri}")"
if [ "$create_status" != "200" ]; then
  echo "sec4-lasm-workbench smoke /wb/tasks expected 200, got $create_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and .op == "exec"' /tmp/sec4-lasm-workbench-create.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/tasks expected {ok:true,op:\"exec\"}" >&2
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
create_tx_status="$(curl -sS -o /tmp/sec4-lasm-workbench-create-tx.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/tasks/with-comment?task_params=${task2_params_uri}&comment_params=${comment_params_uri}")"
if [ "$create_tx_status" != "200" ]; then
  echo "sec4-lasm-workbench smoke /wb/tasks/with-comment expected 200, got $create_tx_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and .op == "execTx"' /tmp/sec4-lasm-workbench-create-tx.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/tasks/with-comment expected {ok:true,op:\"execTx\"}" >&2
  exit 1
fi

get_status="$(curl -sS -o /tmp/sec4-lasm-workbench-get.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks/${task_id}?row_schema=1")"
if [ "$get_status" != "200" ]; then
  echo "sec4-lasm-workbench smoke /wb/tasks/:id expected 200, got $get_status" >&2
  exit 1
fi
if ! jq -e --arg expected "$task_id" '.ok == true and .rowObject.id == $expected' /tmp/sec4-lasm-workbench-get.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/tasks/:id expected rowObject.id=$task_id" >&2
  exit 1
fi

list_params='["",20,0]'
list_params_uri="$(printf '%s' "$list_params" | jq -sRr @uri)"
list_status="$(curl -sS -o /tmp/sec4-lasm-workbench-list.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks?params=${list_params_uri}&row_schema=1")"
if [ "$list_status" != "200" ]; then
  echo "sec4-lasm-workbench smoke /wb/tasks expected 200, got $list_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.rowObject.id | type == "string")' /tmp/sec4-lasm-workbench-list.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/tasks expected rowObject.id string" >&2
  exit 1
fi

records_status="$(curl -sS -o /tmp/sec4-lasm-workbench-records.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/records")"
if [ "$records_status" != "200" ]; then
  echo "sec4-lasm-workbench smoke /wb/records expected 200, got $records_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and .count >= 3' /tmp/sec4-lasm-workbench-records.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/records expected {ok:true,count>=3}" >&2
  exit 1
fi

echo "sec4-lasm-workbench smoke test passed"
