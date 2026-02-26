#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"
port="${BENCH_SMOKE_PORT:-18089}"
dsn="${BENCH_WORKBENCH_PG_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-postgresql://127.0.0.1:5432/postgres?sslmode=disable}}"

BENCH_WORKBENCH_PG_DSN="$dsn" PORT="$port" node "$service_dir/server.mjs" >"$service_dir/.smoke.log" 2>&1 &
pid="$!"
cleanup() {
  if kill -0 "$pid" >/dev/null 2>&1; then
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

ready="false"
for _ in $(seq 1 120); do
  if curl -fsS "http://127.0.0.1:${port}/health" >/tmp/node-workbench-health.txt 2>/dev/null; then
    if [ "$(cat /tmp/node-workbench-health.txt 2>/dev/null || true)" = "ok" ]; then
      ready="true"
      break
    fi
  fi
  sleep 0.1
done

if [ "$ready" != "true" ]; then
  echo "node-workbench smoke health check failed" >&2
  exit 1
fi

auth_header='Authorization: Bearer token123'
run_id="$(date +%s%N)"

setup_status="$(curl -sS -o /tmp/node-workbench-setup.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/setup")"
if [ "$setup_status" != "200" ]; then
  echo "node-workbench /wb/setup expected 200, got $setup_status" >&2
  exit 1
fi

task_id="node-wb-task-${run_id}"
create_status="$(curl -sS -o /tmp/node-workbench-create.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/tasks?id=${task_id}&title=NodeTask&description=smoke&status=open&priority=3&created_at_ms=1700000001000")"
if [ "$create_status" != "201" ]; then
  echo "node-workbench /wb/tasks expected 201, got $create_status" >&2
  exit 1
fi
if ! jq -e --arg id "$task_id" '.ok == true and .data.id == $id' /tmp/node-workbench-create.json >/dev/null; then
  echo "node-workbench /wb/tasks response mismatch" >&2
  exit 1
fi

create_tx_status="$(curl -sS -o /tmp/node-workbench-create-tx.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/tasks/with-comment?id=node-wb-task-tx-${run_id}&title=NodeTaskTx&description=smoke2&status=in_progress&priority=4&created_at_ms=1700000001001&comment_id=node-wb-comment-${run_id}&comment_body=hello&comment_created_at_ms=1700000001002")"
if [ "$create_tx_status" != "201" ]; then
  echo "node-workbench /wb/tasks/with-comment expected 201, got $create_tx_status" >&2
  exit 1
fi
if ! jq -e --arg taskId "node-wb-task-tx-${run_id}" --arg commentId "node-wb-comment-${run_id}" '.ok == true and .data.taskId == $taskId and .data.commentId == $commentId' /tmp/node-workbench-create-tx.json >/dev/null; then
  echo "node-workbench /wb/tasks/with-comment response mismatch" >&2
  exit 1
fi

get_status="$(curl -sS -o /tmp/node-workbench-get.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks/${task_id}")"
if [ "$get_status" != "200" ]; then
  echo "node-workbench /wb/tasks/:id expected 200, got $get_status" >&2
  exit 1
fi
if ! jq -e --arg id "$task_id" '.ok == true and .data.id == $id' /tmp/node-workbench-get.json >/dev/null; then
  echo "node-workbench /wb/tasks/:id response mismatch" >&2
  exit 1
fi

list_status="$(curl -sS -o /tmp/node-workbench-list.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks?status=open&limit=20&offset=0")"
if [ "$list_status" != "200" ]; then
  echo "node-workbench /wb/tasks list expected 200, got $list_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data.items | type == "array") and (.data.count | type == "number")' /tmp/node-workbench-list.json >/dev/null; then
  echo "node-workbench /wb/tasks list response mismatch" >&2
  exit 1
fi

echo "node-workbench smoke test passed"
