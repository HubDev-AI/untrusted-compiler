#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$service_dir/../../.." && pwd)"
port="${BENCH_SMOKE_PORT:-18088}"
auth_token="${BENCH_WORKBENCH_AUTH_TOKEN:-token123}"

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

load_dsn_from_env_file() {
  local env_file="$1"
  if [ -z "$env_file" ] || [ ! -f "$env_file" ]; then
    return 1
  fi
  (
    set -a
    # shellcheck source=/dev/null
    source "$env_file"
    set +a

    local dsn=""
    local candidate_file=""
    for candidate_file in \
      "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE:-}" \
      "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}" \
      "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH:-}" \
      "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH:-}"
    do
      dsn="$(resolve_postgres_dsn_file "$candidate_file" || true)"
      if [ -n "$dsn" ]; then
        break
      fi
    done

    if [ -z "$dsn" ]; then
      dsn="${SEC4_DB_ALPHA_DB_POSTGRES_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-}}"
    fi
    if [ -z "$dsn" ]; then
      local user="${POSTGRES_USER:-sec4}"
      local pass="${POSTGRES_PASSWORD:-sec4dev}"
      local pg_port="${PG_PORT:-5432}"
      local db="${POSTGRES_DB:-sec4_local}"
      dsn="postgres://$user:$pass@127.0.0.1:$pg_port/$db?sslmode=disable"
    fi

    dsn="$(printf '%s' "$dsn" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    if [ -z "$dsn" ]; then
      return 1
    fi
    printf '%s\n' "$dsn"
  )
}

postgres_dsn_is_reachable() {
  local dsn="$1"
  if [ -z "$dsn" ]; then
    return 1
  fi
  if ! command -v psql >/dev/null 2>&1; then
    return 1
  fi
  psql "$dsn" -Atqc 'select 1' >/dev/null 2>&1
}

resolve_repo_local_postgres_env_file() {
  local runtime_env="$repo_root/infra/local-postgres/.runtime.env"
  local base_env="$repo_root/infra/local-postgres/.env"
  if [ -f "$runtime_env" ]; then
    printf '%s\n' "$runtime_env"
    return 0
  fi
  if [ -f "$base_env" ]; then
    printf '%s\n' "$base_env"
    return 0
  fi
  return 1
}

has_explicit_postgres_source="false"
for postgres_env in \
  "${BENCH_WORKBENCH_PG_DSN:-}" \
  "${SEC4_DB_ALPHA_DB_POSTGRES_DSN:-}" \
  "${SEC4_RT_LASM_DB_POSTGRES_DSN:-}" \
  "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE:-}" \
  "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE:-}" \
  "${SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH:-}" \
  "${SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH:-}" \
  "${SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE:-}" \
  "${SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE:-}" \
  "${SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE:-}" \
  "${SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE:-}"
do
  if [ -n "$postgres_env" ]; then
    has_explicit_postgres_source="true"
    break
  fi
done

run_env=()
sec4_bin="${repo_root}/target/debug/sec4"
run_args=(
  --path "$service_dir"
  --backend lasm
  --port "$port"
  --serve-timeout-ms 20000
)
run_cmd=()
if [ -x "$sec4_bin" ]; then
  run_cmd=("$sec4_bin" run)
else
  run_cmd=(cargo run -q -p sec4 -- run)
fi

if [ -n "${BENCH_WORKBENCH_PG_DSN:-}" ]; then
  run_env+=(
    "SEC4_DB_ALPHA_DB_POSTGRES_DSN=${BENCH_WORKBENCH_PG_DSN}"
    "SEC4_RT_LASM_DB_POSTGRES_DSN=${BENCH_WORKBENCH_PG_DSN}"
  )
fi

if [ "$has_explicit_postgres_source" = "true" ]; then
  run_args+=(--db-adapter postgres)
else
  repo_local_env_file="$(resolve_repo_local_postgres_env_file || true)"
  repo_local_dsn="$(load_dsn_from_env_file "$repo_local_env_file" || true)"
  if [ -n "$repo_local_dsn" ] && postgres_dsn_is_reachable "$repo_local_dsn"; then
    run_env+=(
      "SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE=${repo_local_env_file}"
      "SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE=${repo_local_env_file}"
    )
    run_args+=(--db-adapter postgres)
  else
    echo "sec4-lasm-workbench smoke requires a reachable Postgres DSN; sqlite fallback is not supported for this canonical app" >&2
    exit 1
  fi
fi

log_file="${TMPDIR:-/tmp}/sec4-lasm-workbench-smoke.$$.log"
env "${run_env[@]}" "${run_cmd[@]}" "${run_args[@]}" >"$log_file" 2>&1 &
pid=$!
cleanup() {
  if kill -0 "$pid" >/dev/null 2>&1; then
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
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

auth_header="Authorization: Bearer ${auth_token}"
run_id="$(date +%s%N)"
task_id="sec4-lasm-wb-task-${run_id}"
task2_id="sec4-lasm-wb-task-tx-${run_id}"
comment_id="sec4-lasm-wb-comment-${run_id}"

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
if [ "$create_status" != "201" ]; then
  echo "sec4-lasm-workbench smoke /wb/tasks expected 201, got $create_status" >&2
  exit 1
fi
if ! jq -e --arg id "$task_id" '.ok == true and .data.id == $id' /tmp/sec4-lasm-workbench-create.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/tasks response mismatch" >&2
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
if [ "$create_tx_status" != "201" ]; then
  echo "sec4-lasm-workbench smoke /wb/tasks/with-comment expected 201, got $create_tx_status" >&2
  exit 1
fi
if ! jq -e --arg taskId "$task2_id" --arg commentId "$comment_id" '.ok == true and .data.taskId == $taskId and .data.commentId == $commentId' /tmp/sec4-lasm-workbench-create-tx.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/tasks/with-comment response mismatch" >&2
  exit 1
fi

get_status="$(curl -sS -o /tmp/sec4-lasm-workbench-get.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks/${task_id}?row_schema=1")"
if [ "$get_status" != "200" ]; then
  echo "sec4-lasm-workbench smoke /wb/tasks/:id expected 200, got $get_status" >&2
  exit 1
fi
if ! jq -e --arg expected "$task_id" '.ok == true and .data.id == $expected' /tmp/sec4-lasm-workbench-get.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/tasks/:id response mismatch" >&2
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
if ! jq -e --arg first "$task2_id" --arg second "$task_id" '.ok == true and (.data.items | type == "array") and (.data.count | type == "number") and (.data.items | length) >= 2 and .data.items[0].id == $first and .data.items[1].id == $second and .data.count >= 2' /tmp/sec4-lasm-workbench-list.json >/dev/null; then
  echo "sec4-lasm-workbench smoke /wb/tasks response mismatch" >&2
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
