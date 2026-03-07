#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--matrix path] [--impls sec4-lasm,node,go,rust]
          [--endpoints wb-tasks-post,wb-tasks-with-comment,wb-tasks-with-comment-tx,wb-task-comment-post,wb-task-get,wb-tasks-list]
          [--lasm-mode single|fixed|proxy|auto] [--lasm-mode-compare-repeats-file path]
          [--lasm-db-adapter sqlite|postgres] [--lasm-db-base path] [--lasm-postgres-dsn-file path]
          [--lasm-db-postgres-shared-client-max-active-per-key <n>]
          [--lasm-db-postgres-shared-client-max-active-total <n>]
          [--lasm-instances <n>] [--lasm-autoscale-max-instances <n>]
          [--lasm-autoscale-target-connections <n>] [--lasm-autoscale-check-ms <n>]
          [--lasm-cluster-relay-workers <n>] [--lasm-cluster-relay-queue <n>]
          [--lasm-cluster-accept-workers <n>] [--lasm-cluster-relay-accept-batch-max <n>]
          [--lasm-cluster-relay-pump-batch-max <n>]
          [--port <n>] [--out-step-matrix path] [--out-runs path]

Runs workbench step-load benchmark profiles across implemented matrix lanes and emits:
1) per-impl endpoint step summaries + step analyses
2) cross-impl workbench step matrix artifact
USAGE
}

dry_run="false"
matrix_path=""
impls_csv="sec4-lasm,node,go,rust"
endpoints_csv="wb-tasks-post,wb-tasks-with-comment,wb-task-comment-post,wb-task-get,wb-tasks-list"
bench_port="${BENCH_WORKBENCH_PORT:-18093}"
lasm_db_adapter="${BENCH_WORKBENCH_LASM_DB_ADAPTER:-sqlite}"
lasm_db_base="${BENCH_WORKBENCH_LASM_DB_BASE:-}"
lasm_postgres_dsn_file="${BENCH_WORKBENCH_LASM_POSTGRES_DSN_FILE:-}"
lasm_mode="${BENCH_WORKBENCH_LASM_MODE:-}"
lasm_mode_compare_repeats_file=""
lasm_db_postgres_shared_client_max_active_per_key="${BENCH_WORKBENCH_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_ACTIVE_PER_KEY:-}"
lasm_db_postgres_shared_client_max_active_total="${BENCH_WORKBENCH_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_ACTIVE_TOTAL:-}"
lasm_instances="${BENCH_WORKBENCH_LASM_INSTANCES:-1}"
lasm_autoscale_max_instances="${BENCH_WORKBENCH_LASM_AUTOSCALE_MAX_INSTANCES:-}"
lasm_autoscale_target_connections="${BENCH_WORKBENCH_LASM_AUTOSCALE_TARGET_CONNECTIONS:-256}"
lasm_autoscale_check_ms="${BENCH_WORKBENCH_LASM_AUTOSCALE_CHECK_MS:-1000}"
lasm_cluster_relay_workers="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_WORKERS:-}"
lasm_cluster_relay_queue="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_QUEUE:-}"
lasm_cluster_accept_workers="${BENCH_WORKBENCH_LASM_CLUSTER_ACCEPT_WORKERS:-}"
lasm_cluster_relay_accept_batch_max="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX:-}"
lasm_cluster_relay_pump_batch_max="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_PUMP_BATCH_MAX:-}"
require_wrk2="${BENCH_WORKBENCH_REQUIRE_WRK2:-1}"
out_step_matrix=""
out_runs=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
      shift
      ;;
    --matrix)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      matrix_path="$2"
      shift 2
      ;;
    --matrix=*)
      matrix_path="${1#--matrix=}"
      shift
      ;;
    --impls)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      impls_csv="$2"
      shift 2
      ;;
    --impls=*)
      impls_csv="${1#--impls=}"
      shift
      ;;
    --endpoints)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      endpoints_csv="$2"
      shift 2
      ;;
    --endpoints=*)
      endpoints_csv="${1#--endpoints=}"
      shift
      ;;
    --port)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      bench_port="$2"
      shift 2
      ;;
    --port=*)
      bench_port="${1#--port=}"
      shift
      ;;
    --lasm-db-adapter)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_db_adapter="$2"
      shift 2
      ;;
    --lasm-db-adapter=*)
      lasm_db_adapter="${1#--lasm-db-adapter=}"
      shift
      ;;
    --lasm-db-base)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_db_base="$2"
      shift 2
      ;;
    --lasm-db-base=*)
      lasm_db_base="${1#--lasm-db-base=}"
      shift
      ;;
    --lasm-postgres-dsn-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_postgres_dsn_file="$2"
      shift 2
      ;;
    --lasm-postgres-dsn-file=*)
      lasm_postgres_dsn_file="${1#--lasm-postgres-dsn-file=}"
      shift
      ;;
    --lasm-mode)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_mode="$2"
      shift 2
      ;;
    --lasm-mode=*)
      lasm_mode="${1#--lasm-mode=}"
      shift
      ;;
    --lasm-mode-compare-repeats-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_mode_compare_repeats_file="$2"
      shift 2
      ;;
    --lasm-mode-compare-repeats-file=*)
      lasm_mode_compare_repeats_file="${1#--lasm-mode-compare-repeats-file=}"
      shift
      ;;
    --lasm-db-postgres-shared-client-max-active-per-key)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_db_postgres_shared_client_max_active_per_key="$2"
      shift 2
      ;;
    --lasm-db-postgres-shared-client-max-active-per-key=*)
      lasm_db_postgres_shared_client_max_active_per_key="${1#--lasm-db-postgres-shared-client-max-active-per-key=}"
      shift
      ;;
    --lasm-db-postgres-shared-client-max-active-total)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_db_postgres_shared_client_max_active_total="$2"
      shift 2
      ;;
    --lasm-db-postgres-shared-client-max-active-total=*)
      lasm_db_postgres_shared_client_max_active_total="${1#--lasm-db-postgres-shared-client-max-active-total=}"
      shift
      ;;
    --lasm-instances)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_instances="$2"
      shift 2
      ;;
    --lasm-instances=*)
      lasm_instances="${1#--lasm-instances=}"
      shift
      ;;
    --lasm-autoscale-max-instances)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_autoscale_max_instances="$2"
      shift 2
      ;;
    --lasm-autoscale-max-instances=*)
      lasm_autoscale_max_instances="${1#--lasm-autoscale-max-instances=}"
      shift
      ;;
    --lasm-autoscale-target-connections)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_autoscale_target_connections="$2"
      shift 2
      ;;
    --lasm-autoscale-target-connections=*)
      lasm_autoscale_target_connections="${1#--lasm-autoscale-target-connections=}"
      shift
      ;;
    --lasm-autoscale-check-ms)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_autoscale_check_ms="$2"
      shift 2
      ;;
    --lasm-autoscale-check-ms=*)
      lasm_autoscale_check_ms="${1#--lasm-autoscale-check-ms=}"
      shift
      ;;
    --lasm-cluster-relay-workers)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_cluster_relay_workers="$2"
      shift 2
      ;;
    --lasm-cluster-relay-workers=*)
      lasm_cluster_relay_workers="${1#--lasm-cluster-relay-workers=}"
      shift
      ;;
    --lasm-cluster-relay-queue)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_cluster_relay_queue="$2"
      shift 2
      ;;
    --lasm-cluster-relay-queue=*)
      lasm_cluster_relay_queue="${1#--lasm-cluster-relay-queue=}"
      shift
      ;;
    --lasm-cluster-accept-workers)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_cluster_accept_workers="$2"
      shift 2
      ;;
    --lasm-cluster-accept-workers=*)
      lasm_cluster_accept_workers="${1#--lasm-cluster-accept-workers=}"
      shift
      ;;
    --lasm-cluster-relay-accept-batch-max)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_cluster_relay_accept_batch_max="$2"
      shift 2
      ;;
    --lasm-cluster-relay-accept-batch-max=*)
      lasm_cluster_relay_accept_batch_max="${1#--lasm-cluster-relay-accept-batch-max=}"
      shift
      ;;
    --lasm-cluster-relay-pump-batch-max)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_cluster_relay_pump_batch_max="$2"
      shift 2
      ;;
    --lasm-cluster-relay-pump-batch-max=*)
      lasm_cluster_relay_pump_batch_max="${1#--lasm-cluster-relay-pump-batch-max=}"
      shift
      ;;
    --out-step-matrix)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_step_matrix="$2"
      shift 2
      ;;
    --out-step-matrix=*)
      out_step_matrix="${1#--out-step-matrix=}"
      shift
      ;;
    --out-runs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_runs="$2"
      shift 2
      ;;
    --out-runs=*)
      out_runs="${1#--out-runs=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

suite_dir="$(cd "$(dirname "$0")/.." && pwd)"
repo_root="$(cd "$suite_dir/.." && pwd)"

normalize_repo_path() {
  local path="$1"
  if [ -z "$path" ]; then
    printf '%s' "$path"
    return
  fi
  case "$path" in
    "$repo_root"/*)
      printf '%s' "${path#${repo_root}/}"
      ;;
    *)
      printf '%s' "$path"
      ;;
  esac
}

if [ -z "$matrix_path" ]; then
  matrix_path="${suite_dir}/workbench/matrix.backends.json"
fi
if [ -z "$out_step_matrix" ]; then
  out_step_matrix="${suite_dir}/results/summaries/workbench-step-matrix.json"
fi
if [ -z "$out_runs" ]; then
  out_runs="${suite_dir}/results/summaries/workbench-step-runs.json"
fi

if [ ! -f "$matrix_path" ]; then
  echo "workbench matrix missing: $matrix_path" >&2
  exit 2
fi

mkdir -p "${suite_dir}/results/raw" "${suite_dir}/results/summaries"

if [ -n "$impls_csv" ]; then
  selected_impls_json="$(
    IFS=',' read -r -a impls <<<"$impls_csv"
    json='[]'
    for raw_impl in "${impls[@]}"; do
      impl="$(echo "$raw_impl" | tr -d '[:space:]')"
      [ -z "$impl" ] && continue
      json="$(jq -c --arg impl "$impl" '. + [$impl]' <<<"$json")"
    done
    printf '%s' "$json"
  )"
else
  selected_impls_json='[]'
fi

is_selected_impl() {
  local impl="$1"
  if [ "$selected_impls_json" = "[]" ]; then
    return 0
  fi
  jq -e --arg impl "$impl" 'index($impl) != null' <<<"$selected_impls_json" >/dev/null
}

supported_endpoint() {
  case "$1" in
    wb-tasks-post|wb-tasks-with-comment|wb-tasks-with-comment-tx|wb-task-comment-post|wb-task-get|wb-tasks-list)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

supported_lasm_db_adapter() {
  case "$1" in
    sqlite|postgres)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

IFS=',' read -r -a endpoints <<<"$endpoints_csv"
if [ "${#endpoints[@]}" -eq 0 ]; then
  echo "no workbench endpoints provided" >&2
  exit 2
fi
for raw_endpoint in "${endpoints[@]}"; do
  endpoint="$(echo "$raw_endpoint" | tr -d '[:space:]')"
  [ -z "$endpoint" ] && continue
  if ! supported_endpoint "$endpoint"; then
    echo "unsupported workbench endpoint: $endpoint" >&2
    exit 2
  fi
done

if ! supported_lasm_db_adapter "$lasm_db_adapter"; then
  echo "unsupported LASM workbench DB adapter: $lasm_db_adapter" >&2
  exit 2
fi

if [ -z "$lasm_mode_compare_repeats_file" ]; then
  lasm_mode_compare_repeats_file="${suite_dir}/results/summaries/workbench-lasm-mode-compare-repeats.json"
fi

if [ -n "$lasm_mode" ]; then
  if [ "$lasm_mode" = "auto" ]; then
    if [ ! -f "$lasm_mode_compare_repeats_file" ]; then
      echo "LASM auto mode requires mode-compare artifact: $lasm_mode_compare_repeats_file" >&2
      exit 2
    fi
    lasm_mode="$(jq -r '.recommendation.mode // empty' "$lasm_mode_compare_repeats_file")"
    if [ -z "$lasm_mode" ] || [ "$lasm_mode" = "null" ]; then
      echo "LASM auto mode could not resolve recommendation from: $lasm_mode_compare_repeats_file" >&2
      exit 2
    fi
  fi
  case "$lasm_mode" in
    single)
      lasm_instances="1"
      lasm_autoscale_max_instances="1"
      ;;
    fixed)
      if [ "$lasm_instances" -lt 2 ]; then
        lasm_instances="2"
      fi
      lasm_autoscale_max_instances="$lasm_instances"
      ;;
    proxy)
      if [ "$lasm_instances" -lt 2 ]; then
        lasm_instances="2"
      fi
      if [ -z "$lasm_autoscale_max_instances" ] || [ "$lasm_autoscale_max_instances" -le "$lasm_instances" ]; then
        lasm_autoscale_max_instances="$((lasm_instances + 2))"
      fi
      ;;
    *)
      echo "unsupported LASM mode: $lasm_mode" >&2
      exit 2
      ;;
  esac
fi

if [ -z "$lasm_autoscale_max_instances" ]; then
  lasm_autoscale_max_instances="$lasm_instances"
fi

lasm_cluster_mode="single"
if [ "$lasm_instances" != "1" ]; then
  if [ "$lasm_autoscale_max_instances" = "$lasm_instances" ]; then
    lasm_cluster_mode="cluster-fixed"
  else
    lasm_cluster_mode="cluster-proxy"
  fi
fi

if [ "$lasm_cluster_mode" = "single" ] && [ "$lasm_autoscale_max_instances" != "1" ]; then
  echo "LASM workbench step benchmark cluster flags invalid: --lasm-autoscale-max-instances requires --lasm-instances > 1" >&2
  exit 2
fi

if [ "$lasm_cluster_mode" != "cluster-proxy" ] && {
  [ -n "$lasm_cluster_relay_workers" ] ||
  [ -n "$lasm_cluster_relay_queue" ] ||
  [ -n "$lasm_cluster_accept_workers" ] ||
  [ -n "$lasm_cluster_relay_accept_batch_max" ] ||
  [ -n "$lasm_cluster_relay_pump_batch_max" ];
}; then
  echo "LASM workbench step benchmark relay tuning flags require proxy cluster mode (--lasm-instances > 1 with --lasm-autoscale-max-instances > --lasm-instances)" >&2
  exit 2
fi

lasm_cluster_run_args=()
if [ "$lasm_cluster_mode" != "single" ]; then
  lasm_cluster_run_args+=(--instances "$lasm_instances")
  lasm_cluster_run_args+=(--autoscale-max-instances "$lasm_autoscale_max_instances")
  lasm_cluster_run_args+=(--autoscale-target-connections "$lasm_autoscale_target_connections")
  lasm_cluster_run_args+=(--autoscale-check-ms "$lasm_autoscale_check_ms")
  if [ -n "$lasm_cluster_relay_workers" ]; then
    lasm_cluster_run_args+=(--cluster-relay-workers "$lasm_cluster_relay_workers")
  fi
  if [ -n "$lasm_cluster_relay_queue" ]; then
    lasm_cluster_run_args+=(--cluster-relay-queue "$lasm_cluster_relay_queue")
  fi
  if [ -n "$lasm_cluster_accept_workers" ]; then
    lasm_cluster_run_args+=(--cluster-accept-workers "$lasm_cluster_accept_workers")
  fi
  if [ -n "$lasm_cluster_relay_accept_batch_max" ]; then
    lasm_cluster_run_args+=(--cluster-relay-accept-batch-max "$lasm_cluster_relay_accept_batch_max")
  fi
  if [ -n "$lasm_cluster_relay_pump_batch_max" ]; then
    lasm_cluster_run_args+=(--cluster-relay-pump-batch-max "$lasm_cluster_relay_pump_batch_max")
  fi
fi
if [ -n "$lasm_db_postgres_shared_client_max_active_per_key" ]; then
  lasm_cluster_run_args+=(--db-postgres-shared-client-max-active-per-key "$lasm_db_postgres_shared_client_max_active_per_key")
fi
if [ -n "$lasm_db_postgres_shared_client_max_active_total" ]; then
  lasm_cluster_run_args+=(--db-postgres-shared-client-max-active-total "$lasm_db_postgres_shared_client_max_active_total")
fi

runnable_rows_json='[]'
while IFS= read -r impl_row; do
  impl="$(jq -r '.impl' <<<"$impl_row")"
  status="$(jq -r '.status' <<<"$impl_row")"
  if [ "$status" != "implemented-alpha" ] && [ "$status" != "implemented" ]; then
    continue
  fi
  if ! is_selected_impl "$impl"; then
    continue
  fi
  runnable_rows_json="$(jq -c --argjson row "$impl_row" '. + [$row]' <<<"$runnable_rows_json")"
done < <(jq -c '.implementations[]' "$matrix_path")

if [ "$(jq 'length' <<<"$runnable_rows_json")" -eq 0 ]; then
  echo "no runnable workbench implementations selected from matrix" >&2
  exit 2
fi

runnable_impls_csv="$(jq -r '.[].impl' <<<"$runnable_rows_json" | paste -sd, -)"

if [ "$dry_run" = "true" ]; then
  BENCH_REQUIRE_WRK2="$require_wrk2" "${suite_dir}/scripts/preflight.sh" --impls "$runnable_impls_csv" --dry-run-only
else
  BENCH_REQUIRE_WRK2="$require_wrk2" "${suite_dir}/scripts/preflight.sh" --impls "$runnable_impls_csv"
fi

bench_port_base="$bench_port"
current_bench_port="$bench_port_base"
current_base_url="http://127.0.0.1:${current_bench_port}"
pg_dsn="${BENCH_WORKBENCH_PG_DSN:-${SEC4_DB_ALPHA_DB_POSTGRES_DSN:-${SEC4_RT_LASM_DB_POSTGRES_DSN:-postgresql://127.0.0.1:5432/postgres?sslmode=disable}}}"
lasm_postgres_dsn=""
if [ "$lasm_db_adapter" = "postgres" ]; then
  if [ -n "$lasm_postgres_dsn_file" ]; then
    if [ ! -f "$lasm_postgres_dsn_file" ]; then
      echo "lasm postgres dsn file not found: $lasm_postgres_dsn_file" >&2
      exit 2
    fi
    lasm_postgres_dsn="$(<"$lasm_postgres_dsn_file")"
    lasm_postgres_dsn="${lasm_postgres_dsn//$'\r'/}"
    lasm_postgres_dsn="${lasm_postgres_dsn//$'\n'/}"
  else
    lasm_postgres_dsn="$pg_dsn"
  fi
fi

service_pid=""
service_temp_dir=""
cleanup_temp_dir="false"

kill_process_tree_recursive() {
  local root_pid="$1"
  local child_pid=""
  if [ -z "$root_pid" ] || ! [[ "$root_pid" =~ ^[0-9]+$ ]]; then
    return
  fi
  while IFS= read -r child_pid; do
    [ -z "$child_pid" ] && continue
    kill_process_tree_recursive "$child_pid"
  done < <(pgrep -P "$root_pid" 2>/dev/null || true)
  kill "$root_pid" >/dev/null 2>&1 || true
}

wait_for_listener_port_release() {
  local wait_port="$1"
  local attempts=0
  local listener_pid=""
  while [ "$attempts" -lt 50 ]; do
    listener_pid="$(resolve_listener_pid_by_port "$wait_port" || true)"
    if ! [[ "$listener_pid" =~ ^[0-9]+$ ]]; then
      return 0
    fi
    kill_process_tree_recursive "$listener_pid"
    sleep 0.1
    attempts=$((attempts + 1))
  done
  listener_pid="$(resolve_listener_pid_by_port "$wait_port" || true)"
  if [[ "$listener_pid" =~ ^[0-9]+$ ]]; then
    return 1
  fi
  return 0
}

cleanup_impl() {
  if [ -n "$service_pid" ] && kill -0 "$service_pid" >/dev/null 2>&1; then
    kill_process_tree_recursive "$service_pid"
    wait "$service_pid" >/dev/null 2>&1 || true
  fi
  wait_for_listener_port_release "$current_bench_port" >/dev/null 2>&1 || true
  service_pid=""
  if [ "$cleanup_temp_dir" = "true" ] && [ -n "$service_temp_dir" ] && [ -d "$service_temp_dir" ]; then
    rm -rf "$service_temp_dir"
  fi
  service_temp_dir=""
  cleanup_temp_dir="false"
}

start_impl_service() {
  local impl="$1"
  local service_abs="$2"
  local log_file="$3"

  service_temp_dir=""
  cleanup_temp_dir="false"

  case "$impl" in
    sec4)
      service_temp_dir="$(mktemp -d "/tmp/sec4-workbench-step-db.XXXXXX")"
      cleanup_temp_dir="true"
      (
        cd "$repo_root"
        SEC4_RT_DB_BASE="$service_temp_dir" cargo run -q -p sec4 -- run \
          --path "$service_abs" \
          --backend c \
          --port "$current_bench_port" \
          --serve-timeout-ms 20000
      ) >"$log_file" 2>&1 &
      ;;
    sec4-lasm)
      if [ "$lasm_db_adapter" = "sqlite" ]; then
        if [ -n "$lasm_db_base" ]; then
          service_temp_dir="$lasm_db_base"
          cleanup_temp_dir="false"
          mkdir -p "$service_temp_dir"
        else
          service_temp_dir="$(mktemp -d "/tmp/sec4-lasm-workbench-step-db.XXXXXX")"
          cleanup_temp_dir="true"
        fi
        (
          cd "$repo_root"
          cargo run -q -p sec4 -- run \
            --path "$service_abs" \
            --backend lasm \
            --db-adapter sqlite \
            --db-base "$service_temp_dir" \
            --port "$current_bench_port" \
            --serve-timeout-ms 20000 \
            "${lasm_cluster_run_args[@]}"
        ) >"$log_file" 2>&1 &
      else
        (
          cd "$repo_root"
          SEC4_RT_LASM_DB_POSTGRES_DSN="$lasm_postgres_dsn" \
            cargo run -q -p sec4 -- run \
              --path "$service_abs" \
              --backend lasm \
              --db-adapter postgres \
              --port "$current_bench_port" \
              --serve-timeout-ms 20000 \
              "${lasm_cluster_run_args[@]}"
        ) >"$log_file" 2>&1 &
      fi
      ;;
    node)
      (
        cd "$service_abs"
        BENCH_WORKBENCH_PG_DSN="$pg_dsn" PORT="$current_bench_port" node server.mjs
      ) >"$log_file" 2>&1 &
      ;;
    go)
      (
        cd "$service_abs"
        BENCH_WORKBENCH_PG_DSN="$pg_dsn" PORT="$current_bench_port" go run .
      ) >"$log_file" 2>&1 &
      ;;
    rust)
      (
        cd "$service_abs"
        BENCH_WORKBENCH_PG_DSN="$pg_dsn" PORT="$current_bench_port" cargo run --quiet
      ) >"$log_file" 2>&1 &
      ;;
    *)
      echo "unsupported workbench impl runtime: $impl" >&2
      return 2
      ;;
  esac

  service_pid="$!"
}

wait_ready() {
  local pid="$1"
  local health_file="/tmp/workbench-step-health-${current_bench_port}.txt"
  local ready_timeout_seconds="${BENCH_WORKBENCH_READY_TIMEOUT_SECONDS:-90}"
  local ready_probe_interval_seconds="${BENCH_WORKBENCH_READY_PROBE_INTERVAL_SECONDS:-0.1}"
  local health_body=""
  local deadline=$((SECONDS + ready_timeout_seconds))

  while [ "$SECONDS" -lt "$deadline" ]; do
    if ! kill -0 "$pid" >/dev/null 2>&1; then
      return 2
    fi
    if curl -fsS "${current_base_url}/health" >"$health_file" 2>/dev/null; then
      health_body="$(tr -d '\r\n[:space:]' <"$health_file" 2>/dev/null || true)"
      if [ "$health_body" = "ok" ] || [ "$health_body" = "\"ok\"" ]; then
        return 0
      fi
    fi
    sleep "$ready_probe_interval_seconds"
  done
  return 1
}

resolve_listener_pid_by_port() {
  local port="$1"
  local pid=""
  if command -v lsof >/dev/null 2>&1; then
    pid="$(lsof -nP -t -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk 'NF { print; exit }')"
    if [[ "$pid" =~ ^[0-9]+$ ]]; then
      printf '%s\n' "$pid"
      return 0
    fi
  fi
  if command -v ss >/dev/null 2>&1; then
    pid="$(ss -ltnp "sport = :$port" 2>/dev/null | awk -F'pid=|,' '/pid=/{print $2; exit}')"
    if [[ "$pid" =~ ^[0-9]+$ ]]; then
      printf '%s\n' "$pid"
      return 0
    fi
  fi
  return 1
}

seed_impl_state() {
  local impl="$1"
  local auth_header='Authorization: Bearer token123'
  local run_id
  run_id="$(date +%s%N)"

  local setup_status
  setup_status="$(curl -sS -o "/tmp/workbench-step-${impl}-setup.json" -w '%{http_code}' \
    -X POST -H "$auth_header" "${current_base_url}/wb/setup")"
  if [ "$setup_status" != "200" ]; then
    echo "workbench step setup failed impl=${impl} status=${setup_status}" >&2
    return 1
  fi

  local seed_task_id="${impl}-wb-step-seed-task-${run_id}"
  local seed_comment_id="${impl}-wb-step-seed-comment-${run_id}"
  local seed_task_params
  local seed_task_params_uri
  local seed_task_status

  seed_task_params="$(jq -nc \
    --arg id "$seed_task_id" \
    --arg title "SeedTask" \
    --arg description "workbench step benchmark seed" \
    --arg status "open" \
    --argjson priority 3 \
    --argjson created 1700000000000 \
    '[ $id, $title, $description, $status, $priority, $created ]')"
  seed_task_params_uri="$(printf '%s' "$seed_task_params" | jq -sRr @uri)"
  seed_task_status="$(curl -sS -o "/tmp/workbench-step-${impl}-seed-task.json" -w '%{http_code}' \
    -X POST -H "$auth_header" \
    "${current_base_url}/wb/tasks?params=${seed_task_params_uri}")"
  case "$seed_task_status" in
    200|201) ;;
    *)
      echo "workbench step seed task failed impl=${impl} status=${seed_task_status}" >&2
      return 1
      ;;
  esac

  local seed_comment_params
  local seed_comment_params_uri
  local seed_comment_status

  seed_comment_params="$(jq -nc \
    --arg id "$seed_comment_id" \
    --arg task_id "$seed_task_id" \
    --arg body "seed comment" \
    --argjson created 1700000000001 \
    '[ $id, $task_id, $body, $created ]')"
  seed_comment_params_uri="$(printf '%s' "$seed_comment_params" | jq -sRr @uri)"
  seed_comment_status="$(curl -sS -o "/tmp/workbench-step-${impl}-seed-comment.json" -w '%{http_code}' \
    -X POST -H "$auth_header" \
    "${current_base_url}/wb/tasks/${seed_task_id}/comments?params=${seed_comment_params_uri}")"
  case "$seed_comment_status" in
    200|201) ;;
    *)
      echo "workbench step seed comment failed impl=${impl} status=${seed_comment_status}" >&2
      return 1
      ;;
  esac

  printf '%s' "$seed_task_id"
}

probe_endpoint_once() {
  local impl="$1"
  local endpoint="$2"
  local seed_task_id="$3"
  local run_tag="$4"
  bash "${suite_dir}/scripts/workbench_preflight_probe.sh" \
    --impl "$impl" \
    --task-id "$seed_task_id" \
    --run-tag "$run_tag" \
    "$endpoint" "$current_base_url"
}

trap cleanup_impl EXIT

started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
runs_json='[]'
total_passed=0
total_failed=0
total_skipped=0
passed_impls_csv=""
impl_index=0

while IFS= read -r impl_row; do
  impl="$(jq -r '.impl' <<<"$impl_row")"
  status="$(jq -r '.status' <<<"$impl_row")"
  service_rel="$(jq -r '.servicePath' <<<"$impl_row")"
  service_abs="${repo_root}/${service_rel}"
  current_bench_port=$((bench_port_base + impl_index))
  current_base_url="http://127.0.0.1:${current_bench_port}"
  log_file="${suite_dir}/results/raw/${impl}-workbench-step-service.log"

  if [ "$dry_run" = "true" ]; then
    if [ "$impl" = "sec4-lasm" ]; then
      if [ "$lasm_db_adapter" = "postgres" ]; then
        echo "start: impl=${impl} servicePath=${service_rel} port=${current_bench_port} lasmDbAdapter=${lasm_db_adapter} lasmMode=${lasm_cluster_mode} lasmInstances=${lasm_instances} lasmAutoscaleMaxInstances=${lasm_autoscale_max_instances} lasmPostgresDsn=${lasm_postgres_dsn_file:-ENV/default}"
      else
        echo "start: impl=${impl} servicePath=${service_rel} port=${current_bench_port} lasmDbAdapter=${lasm_db_adapter} lasmMode=${lasm_cluster_mode} lasmInstances=${lasm_instances} lasmAutoscaleMaxInstances=${lasm_autoscale_max_instances} lasmDbBase=${lasm_db_base:-mktemp}"
      fi
    else
      echo "start: impl=${impl} servicePath=${service_rel} port=${current_bench_port}"
    fi
    for raw_endpoint in "${endpoints[@]}"; do
      endpoint="$(echo "$raw_endpoint" | tr -d '[:space:]')"
      [ -z "$endpoint" ] && continue
      echo "run: ${suite_dir}/scripts/run_workbench_step_profile.sh --dry-run ${impl} ${endpoint} ${current_base_url}"
      echo "run: ${suite_dir}/scripts/analyze_step_profile.sh ${suite_dir}/results/summaries/${impl}-${endpoint}-step.json ${suite_dir}/results/summaries/${impl}-${endpoint}-step-analysis.json"
    done
    continue
  fi

  result="passed"
  reason=""
  exit_code=0
  seed_task_id=""
  profile_service_pid=""

  start_impl_service "$impl" "$service_abs" "$log_file" || {
    result="failed"
    reason="failed to start service process"
    exit_code=1
  }

  if [ "$result" = "passed" ]; then
    if ! wait_ready "$service_pid"; then
      result="failed"
      reason="service failed readiness on ${current_base_url}/health"
      exit_code=1
    else
      profile_service_pid="$service_pid"
      if [ "$impl" = "sec4-lasm" ] && [ "$lasm_cluster_mode" = "proxy" ]; then
        resolved_listener_pid="$(resolve_listener_pid_by_port "$current_bench_port" || true)"
        if [[ "$resolved_listener_pid" =~ ^[0-9]+$ ]]; then
          profile_service_pid="$resolved_listener_pid"
        fi
      fi
    fi
  fi

  if [ "$result" = "passed" ]; then
    seed_task_id="$(seed_impl_state "$impl" || true)"
    if [ -z "$seed_task_id" ]; then
      result="failed"
      reason="failed to setup/seed workbench state"
      exit_code=1
    fi
  fi

  if [ "$result" = "passed" ]; then
    for raw_endpoint in "${endpoints[@]}"; do
      endpoint="$(echo "$raw_endpoint" | tr -d '[:space:]')"
      [ -z "$endpoint" ] && continue
      run_tag="${seed_task_id}-${endpoint}-step"
      probe_run_tag="${seed_task_id}-${endpoint}-probe"
      probe_json=""
      if ! probe_json="$(probe_endpoint_once "$impl" "$endpoint" "$seed_task_id" "$probe_run_tag")"; then
        probe_reason="$(jq -r '.reason // empty' <<<"$probe_json" 2>/dev/null || true)"
        result="failed"
        if [ -n "$probe_reason" ]; then
          reason="preflight failed ${probe_reason}"
        else
          reason="preflight failed endpoint=${endpoint}"
        fi
        exit_code=1
        break
      fi
      if ! BENCH_REQUIRE_WRK2="$require_wrk2" BENCH_SERVER_PID="$profile_service_pid" BENCH_SERVER_PORT="$current_bench_port" BENCH_WB_TASK_ID="$seed_task_id" BENCH_WB_RUN_TAG="$run_tag" \
        "${suite_dir}/scripts/run_workbench_step_profile.sh" "$impl" "$endpoint" "$current_base_url"; then
        result="failed"
        reason="step profile failed endpoint=${endpoint}"
        exit_code=1
        break
      fi
      if ! "${suite_dir}/scripts/analyze_step_profile.sh" \
        "${suite_dir}/results/summaries/${impl}-${endpoint}-step.json" \
        "${suite_dir}/results/summaries/${impl}-${endpoint}-step-analysis.json" \
        "${STEP_KNEE_THRESHOLD:-0.9}"; then
        result="failed"
        reason="step analysis failed endpoint=${endpoint}"
        exit_code=1
        break
      fi
    done
  fi

  if [ "$result" = "passed" ]; then
    total_passed=$((total_passed + 1))
    if [ -z "$passed_impls_csv" ]; then
      passed_impls_csv="$impl"
    else
      passed_impls_csv="${passed_impls_csv},${impl}"
    fi
  else
    total_failed=$((total_failed + 1))
    if [ -f "$log_file" ]; then
      echo "--- ${impl} workbench step log tail ---" >&2
      tail -n 30 "$log_file" >&2 || true
      echo "--- end log tail ---" >&2
    fi
  fi

  run_row="$(jq -nc \
    --arg impl "$impl" \
    --arg status "$status" \
    --arg servicePath "$service_rel" \
    --arg result "$result" \
    --arg reason "$reason" \
    --argjson exitCode "$exit_code" \
    --arg endpoints "$endpoints_csv" \
    --arg lasmDbAdapter "$lasm_db_adapter" \
    --arg lasmClusterMode "$lasm_cluster_mode" \
    --argjson lasmInstances "$lasm_instances" \
    --argjson lasmAutoscaleMaxInstances "$lasm_autoscale_max_instances" \
    --argjson lasmAutoscaleTargetConnections "$lasm_autoscale_target_connections" \
    --argjson lasmAutoscaleCheckMs "$lasm_autoscale_check_ms" \
    --argjson lasmClusterRelayWorkers "$(if [ -n "$lasm_cluster_relay_workers" ]; then printf '%s' "$lasm_cluster_relay_workers"; else printf 'null'; fi)" \
    --argjson lasmClusterRelayQueue "$(if [ -n "$lasm_cluster_relay_queue" ]; then printf '%s' "$lasm_cluster_relay_queue"; else printf 'null'; fi)" \
    --argjson lasmClusterAcceptWorkers "$(if [ -n "$lasm_cluster_accept_workers" ]; then printf '%s' "$lasm_cluster_accept_workers"; else printf 'null'; fi)" \
    --argjson lasmClusterRelayAcceptBatchMax "$(if [ -n "$lasm_cluster_relay_accept_batch_max" ]; then printf '%s' "$lasm_cluster_relay_accept_batch_max"; else printf 'null'; fi)" \
    --argjson lasmClusterRelayPumpBatchMax "$(if [ -n "$lasm_cluster_relay_pump_batch_max" ]; then printf '%s' "$lasm_cluster_relay_pump_batch_max"; else printf 'null'; fi)" \
    --argjson lasmDbPostgresSharedClientMaxActivePerKey "$(if [ -n "$lasm_db_postgres_shared_client_max_active_per_key" ]; then printf '%s' "$lasm_db_postgres_shared_client_max_active_per_key"; else printf 'null'; fi)" \
    --argjson lasmDbPostgresSharedClientMaxActiveTotal "$(if [ -n "$lasm_db_postgres_shared_client_max_active_total" ]; then printf '%s' "$lasm_db_postgres_shared_client_max_active_total"; else printf 'null'; fi)" \
    '{
      impl: $impl,
      status: $status,
      servicePath: $servicePath,
      stepResult: $result,
      reason: (if $reason == "" then null else $reason end),
      exitCode: $exitCode,
      endpoints: ($endpoints | split(",")),
      lasm: (if $impl == "sec4-lasm" then {
        dbAdapter: $lasmDbAdapter,
        mode: $lasmClusterMode,
        instances: $lasmInstances,
        autoscaleMaxInstances: $lasmAutoscaleMaxInstances,
        autoscaleTargetConnections: $lasmAutoscaleTargetConnections,
        autoscaleCheckMs: $lasmAutoscaleCheckMs,
        clusterRelayWorkers: $lasmClusterRelayWorkers,
        clusterRelayQueue: $lasmClusterRelayQueue,
        clusterAcceptWorkers: $lasmClusterAcceptWorkers,
        clusterRelayAcceptBatchMax: $lasmClusterRelayAcceptBatchMax,
        clusterRelayPumpBatchMax: $lasmClusterRelayPumpBatchMax,
        dbPostgresSharedClientMaxActivePerKey: $lasmDbPostgresSharedClientMaxActivePerKey,
        dbPostgresSharedClientMaxActiveTotal: $lasmDbPostgresSharedClientMaxActiveTotal
      } else null end),
      lasmRuntime: (if $impl == "sec4-lasm" then {
        dbAdapter: $lasmDbAdapter,
        mode: $lasmClusterMode,
        instances: $lasmInstances,
        autoscaleMaxInstances: $lasmAutoscaleMaxInstances,
        autoscaleTargetConnections: $lasmAutoscaleTargetConnections,
        autoscaleCheckMs: $lasmAutoscaleCheckMs,
        clusterRelayWorkers: $lasmClusterRelayWorkers,
        clusterRelayQueue: $lasmClusterRelayQueue,
        clusterAcceptWorkers: $lasmClusterAcceptWorkers,
        clusterRelayAcceptBatchMax: $lasmClusterRelayAcceptBatchMax,
        clusterRelayPumpBatchMax: $lasmClusterRelayPumpBatchMax,
        dbPostgresSharedClientMaxActivePerKey: $lasmDbPostgresSharedClientMaxActivePerKey,
        dbPostgresSharedClientMaxActiveTotal: $lasmDbPostgresSharedClientMaxActiveTotal
      } else null end)
    }')"
  runs_json="$(jq -c --argjson row "$run_row" '. + [$row]' <<<"$runs_json")"

  cleanup_impl
  impl_index=$((impl_index + 1))
done < <(jq -c '.[]' <<<"$runnable_rows_json")

if [ "$dry_run" = "true" ]; then
  echo "run: ${suite_dir}/scripts/compare_step_matrix.sh ${suite_dir}/results/summaries ${out_step_matrix} ${runnable_impls_csv} ${endpoints_csv}"
  exit 0
fi

finished_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
mkdir -p "$(dirname "$out_runs")"
matrix_path_rel="$(normalize_repo_path "$matrix_path")"
out_step_matrix_rel="$(normalize_repo_path "$out_step_matrix")"
jq -n \
  --arg version "0.1" \
  --arg startedAt "$started_at" \
  --arg finishedAt "$finished_at" \
  --arg matrixPath "$matrix_path_rel" \
  --arg endpoints "$endpoints_csv" \
  --arg stepMatrixPath "$out_step_matrix_rel" \
  --argjson totals "$(jq -nc --argjson passed "$total_passed" --argjson failed "$total_failed" --argjson skipped "$total_skipped" '{passed:$passed,failed:$failed,skipped:$skipped}')" \
  --argjson runs "$runs_json" \
  '{
    version: $version,
    startedAt: $startedAt,
    finishedAt: $finishedAt,
    matrixPath: $matrixPath,
    endpoints: ($endpoints | split(",")),
    stepMatrixPath: $stepMatrixPath,
    totals: $totals,
    runs: $runs
  }' >"$out_runs"

if [ -n "$passed_impls_csv" ]; then
  "${suite_dir}/scripts/compare_step_matrix.sh" "${suite_dir}/results/summaries" "$out_step_matrix" "$passed_impls_csv" "$endpoints_csv"
fi

echo "wrote workbench step run summary: $out_runs"
if [ -n "$passed_impls_csv" ]; then
  echo "wrote workbench step matrix: $out_step_matrix"
fi
echo "totals: passed=${total_passed} failed=${total_failed} skipped=${total_skipped}"

if [ "$total_failed" -gt 0 ]; then
  exit 1
fi
