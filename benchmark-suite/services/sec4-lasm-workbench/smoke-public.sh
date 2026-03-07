#!/usr/bin/env bash
set -euo pipefail

service_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$service_dir/../../.." && pwd)"
port="${BENCH_SMOKE_PORT:-18089}"
auth_token="${BENCH_WORKBENCH_AUTH_TOKEN:-token123}"

resolve_postgres_dsn_file() {
  local dsn_file="$1"
  local dsn
  if [ -z "$dsn_file" ] || [ ! -f "$dsn_file" ]; then
    return 1
  fi
  dsn="$(tr -d '\r\n' < "$dsn_file")"
  dsn="$(printf '%s' "$dsn" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
  [ -n "$dsn" ] || return 1
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
    [ -n "$dsn" ] || return 1
    printf '%s\n' "$dsn"
  )
}

postgres_dsn_is_reachable() {
  local dsn="$1"
  [ -n "$dsn" ] || return 1
  command -v psql >/dev/null 2>&1 || return 1
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
    echo "sec4-lasm-workbench public smoke requires a reachable Postgres DSN; sqlite fallback is not supported for this canonical app" >&2
    exit 1
  fi
fi

log_file="${TMPDIR:-/tmp}/sec4-lasm-workbench-smoke-public.$$.log"
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
  if curl -fsS "http://127.0.0.1:${port}/health" >/tmp/sec4-lasm-workbench-public-health.txt 2>/dev/null; then
    if [ "$(cat /tmp/sec4-lasm-workbench-public-health.txt 2>/dev/null || true)" = "ok" ]; then
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
  echo "sec4-lasm-workbench public smoke health check failed" >&2
  exit 1
fi

auth_header="Authorization: Bearer ${auth_token}"
run_id="$(date +%s%N)"

setup_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-setup.json -w '%{http_code}' \
  -X POST -H "$auth_header" \
  "http://127.0.0.1:${port}/wb/setup")"
if [ "$setup_code" != "200" ]; then
  echo "public smoke /wb/setup expected 200, got $setup_code" >&2
  exit 1
fi

create_payload="$(jq -nc \
  --arg title "Public Task" \
  --arg description "public json create" \
  --arg status "open" \
  --argjson priority 3 \
  '{title:$title,description:$description,status:$status,priority:$priority,labels:["api","urgent"]}')"
create_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-create.json -w '%{http_code}' \
  -X POST -H "$auth_header" -H 'Content-Type: application/json' \
  --data "$create_payload" \
  "http://127.0.0.1:${port}/wb/tasks")"
if [ "$create_code" != "201" ]; then
  echo "public smoke /wb/tasks expected 201, got $create_code" >&2
  exit 1
fi
task_id="$(jq -r '.data.id' /tmp/sec4-lasm-workbench-public-create.json)"
if [ -z "$task_id" ] || [ "$task_id" = "null" ]; then
  echo "public smoke /wb/tasks missing task id" >&2
  exit 1
fi

tx_payload="$(jq -nc \
  --arg title "Public Tx Task" \
  --arg description "public tx create" \
  --arg status "in_progress" \
  --argjson priority 4 \
  --arg body "first note" \
  '{task:{title:$title,description:$description,status:$status,priority:$priority},comment:{body:$body}}')"
tx_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-create-tx.json -w '%{http_code}' \
  -X POST -H "$auth_header" -H 'Content-Type: application/json' \
  --data "$tx_payload" \
  "http://127.0.0.1:${port}/wb/tasks/with-comment")"
if [ "$tx_code" != "201" ]; then
  echo "public smoke /wb/tasks/with-comment expected 201, got $tx_code" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data.taskId | type == "string") and (.data.commentId | type == "string")' /tmp/sec4-lasm-workbench-public-create-tx.json >/dev/null; then
  echo "public smoke /wb/tasks/with-comment response mismatch" >&2
  exit 1
fi

tx_chain_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-create-tx-chain.json -w '%{http_code}' \
  -X POST -H "$auth_header" -H 'Content-Type: application/json' \
  --data "$tx_payload" \
  "http://127.0.0.1:${port}/wb/tasks/with-comment-tx")"
if [ "$tx_chain_code" != "201" ]; then
  echo "public smoke /wb/tasks/with-comment-tx expected 201, got $tx_chain_code" >&2
  exit 1
fi
if ! jq -e '.ok == true and (.data.taskId | type == "string") and (.data.commentId | type == "string")' /tmp/sec4-lasm-workbench-public-create-tx-chain.json >/dev/null; then
  echo "public smoke /wb/tasks/with-comment-tx response mismatch" >&2
  exit 1
fi

comment_payload='{"body":"follow-up"}'
comment_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-comment.json -w '%{http_code}' \
  -X POST -H "$auth_header" -H 'Content-Type: application/json' \
  --data "$comment_payload" \
  "http://127.0.0.1:${port}/wb/tasks/${task_id}/comments")"
if [ "$comment_code" != "201" ]; then
  echo "public smoke /wb/tasks/:id/comments expected 201, got $comment_code" >&2
  exit 1
fi

get_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-get.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks/${task_id}")"
if [ "$get_code" != "200" ]; then
  echo "public smoke /wb/tasks/:id expected 200, got $get_code" >&2
  exit 1
fi
if ! jq -e --arg expected "$task_id" '.ok == true and .data.id == $expected and (.data.labels | type == "array") and (.data.labels | index("api") != null)' /tmp/sec4-lasm-workbench-public-get.json >/dev/null; then
  echo "public smoke /wb/tasks/:id response mismatch" >&2
  exit 1
fi

list_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-list.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks?status=open&label=api&limit=10&offset=0")"
if [ "$list_code" != "200" ]; then
  echo "public smoke /wb/tasks expected 200, got $list_code" >&2
  exit 1
fi
if ! jq -e --arg expected "$task_id" '.ok == true and .data.limit == 10 and .data.offset == 0 and (.data.items | type == "array") and (.data.items | any(.id == $expected))' /tmp/sec4-lasm-workbench-public-list.json >/dev/null; then
  echo "public smoke /wb/tasks response mismatch" >&2
  exit 1
fi

invalid_create_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-invalid-create.json -w '%{http_code}' \
  -X POST -H "$auth_header" -H 'Content-Type: application/json' \
  --data '{"title":"bad priority","priority":9}' \
  "http://127.0.0.1:${port}/wb/tasks")"
if [ "$invalid_create_code" != "400" ]; then
  echo "public smoke invalid /wb/tasks expected 400, got $invalid_create_code" >&2
  exit 1
fi
if ! jq -e '.ok == false and .error.code == "VALIDATION.INVALID" and .error.message == "priority must be an integer between 1 and 5"' /tmp/sec4-lasm-workbench-public-invalid-create.json >/dev/null; then
  echo "public smoke invalid /wb/tasks response mismatch" >&2
  exit 1
fi

invalid_list_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-invalid-list.json -w '%{http_code}' \
  "http://127.0.0.1:${port}/wb/tasks?priorityMin=oops")"
if [ "$invalid_list_code" != "400" ]; then
  echo "public smoke invalid /wb/tasks list expected 400, got $invalid_list_code" >&2
  exit 1
fi
if ! jq -e '.ok == false and .error.code == "VALIDATION.INVALID" and .error.message == "priorityMin must be an integer"' /tmp/sec4-lasm-workbench-public-invalid-list.json >/dev/null; then
  echo "public smoke invalid /wb/tasks list response mismatch" >&2
  exit 1
fi

missing_code="$(curl -sS -o /tmp/sec4-lasm-workbench-public-missing-comment.json -w '%{http_code}' \
  -X POST -H "$auth_header" -H 'Content-Type: application/json' \
  --data '{"body":"ghost"}' \
  "http://127.0.0.1:${port}/wb/tasks/missing-task/comments")"
if [ "$missing_code" != "404" ]; then
  echo "public smoke missing task comment expected 404, got $missing_code" >&2
  exit 1
fi
if ! jq -e '.ok == false and .error.code == "TASK.NOT_FOUND"' /tmp/sec4-lasm-workbench-public-missing-comment.json >/dev/null; then
  echo "public smoke missing task comment response mismatch" >&2
  exit 1
fi

echo "sec4-lasm-workbench public smoke test passed"
