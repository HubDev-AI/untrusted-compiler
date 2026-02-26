#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"
port="${BENCH_SMOKE_PORT:-18091}"
dsn="${BENCH_WORKBENCH_PG_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-postgresql://127.0.0.1:5432/postgres?sslmode=disable}}"

(cd "$service_dir" && BENCH_WORKBENCH_PG_DSN="$dsn" PORT="$port" cargo run --quiet) >"$service_dir/.smoke.log" 2>&1 &
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
  if curl -fsS "http://127.0.0.1:${port}/health" >/tmp/rust-workbench-health.txt 2>/dev/null; then
    if [ "$(cat /tmp/rust-workbench-health.txt 2>/dev/null || true)" = "ok" ]; then
      ready="true"
      break
    fi
  fi
  sleep 0.1
done

if [ "$ready" != "true" ]; then
  echo "rust-workbench smoke health check failed" >&2
  exit 1
fi

auth_header='Authorization: Bearer token123'
run_id="$(date +%s%N)"
task_id="rust-wb-task-${run_id}"

setup_status="$(curl -sS -o /tmp/rust-workbench-setup.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/setup")"
if [ "$setup_status" != "200" ]; then
  echo "rust-workbench /wb/setup expected 200, got $setup_status" >&2
  exit 1
fi

create_status="$(curl -sS -o /tmp/rust-workbench-create.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/tasks?id=${task_id}&title=RustTask&description=smoke&status=open&priority=3&created_at_ms=1700000020000")"
if [ "$create_status" != "201" ]; then
  echo "rust-workbench /wb/tasks expected 201, got $create_status" >&2
  exit 1
fi
if ! jq -e --arg id "$task_id" '.ok == true and .data.id == $id' /tmp/rust-workbench-create.json >/dev/null; then
  echo "rust-workbench /wb/tasks response mismatch" >&2
  exit 1
fi

create_tx_status="$(curl -sS -o /tmp/rust-workbench-create-tx.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/tasks/with-comment?id=rust-wb-task-tx-${run_id}&title=RustTaskTx&description=smoke2&status=in_progress&priority=4&created_at_ms=1700000020001&comment_id=rust-wb-comment-${run_id}&comment_body=hello&comment_created_at_ms=1700000020002")"
if [ "$create_tx_status" != "201" ]; then
  echo "rust-workbench /wb/tasks/with-comment expected 201, got $create_tx_status" >&2
  exit 1
fi
if ! jq -e --arg taskId "rust-wb-task-tx-${run_id}" --arg commentId "rust-wb-comment-${run_id}" '.ok == true and .data.taskId == $taskId and .data.commentId == $commentId' /tmp/rust-workbench-create-tx.json >/dev/null; then
  echo "rust-workbench /wb/tasks/with-comment response mismatch" >&2
  exit 1
fi

get_status="$(curl -sS -o /tmp/rust-workbench-get.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks/${task_id}")"
if [ "$get_status" != "200" ]; then
  echo "rust-workbench /wb/tasks/:id expected 200, got $get_status" >&2
  exit 1
fi
if ! jq -e --arg id "$task_id" '.ok == true and .data.id == $id' /tmp/rust-workbench-get.json >/dev/null; then
  echo "rust-workbench /wb/tasks/:id response mismatch" >&2
  exit 1
fi

list_status="$(curl -sS -o /tmp/rust-workbench-list.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks?status=open&limit=20&offset=0")"
if [ "$list_status" != "200" ]; then
  echo "rust-workbench /wb/tasks list expected 200, got $list_status" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data.items | type == "array") and (.data.count | type == "number")' /tmp/rust-workbench-list.json >/dev/null; then
  echo "rust-workbench /wb/tasks list response mismatch" >&2
  exit 1
fi

echo "rust-workbench smoke test passed"
