#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--matrix path] [--impls sec4-lasm,node,go,rust]
          [--endpoints wb-tasks-post,wb-tasks-with-comment,wb-tasks-with-comment-tx,wb-task-comment-post,wb-task-get,wb-tasks-list]
          [--lasm-db-adapter sqlite|postgres] [--lasm-db-base path] [--lasm-postgres-dsn-file path]
          [--lasm-db-records-capture-enabled 0|1]
          [--lasm-mode single|fixed|proxy|auto] [--lasm-mode-compare-repeats-file path]
          [--lasm-mode-compare-repeats <n>] [--out-mode-compare-repeats path]
          [--lasm-db-postgres-shared-client-max-active-per-key <n>]
          [--lasm-db-postgres-shared-client-max-active-total <n>]
          [--lasm-instances <n>] [--lasm-autoscale-max-instances <n>]
          [--lasm-autoscale-target-connections <n>] [--lasm-autoscale-check-ms <n>]
          [--lasm-cluster-relay-workers <n>] [--lasm-cluster-relay-queue <n>]
          [--lasm-cluster-accept-workers <n>] [--lasm-cluster-relay-accept-batch-max <n>]
          [--lasm-cluster-relay-pump-batch-max <n>]
          [--port <n>] [--out-runs path] [--out-fixed-runs path] [--out-step-runs path]
          [--out-compare path] [--out-analysis path] [--out-step-matrix path]
          [--out-report path] [--out-report-html path]

Runs workbench fixed-target matrix + workbench step-load matrix and republishes one
combined markdown report containing step-load signals.
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
lasm_db_records_capture_enabled="${BENCH_WORKBENCH_LASM_DB_RECORDS_CAPTURE_ENABLED:-}"
lasm_mode="${BENCH_WORKBENCH_LASM_MODE:-}"
lasm_mode_compare_repeats_file=""
lasm_mode_compare_repeats="${BENCH_WORKBENCH_LASM_MODE_COMPARE_REPEATS:-}"
lasm_db_postgres_shared_client_max_active_per_key="${BENCH_WORKBENCH_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_ACTIVE_PER_KEY:-}"
lasm_db_postgres_shared_client_max_active_total="${BENCH_WORKBENCH_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_ACTIVE_TOTAL:-}"
lasm_instances="${BENCH_WORKBENCH_LASM_INSTANCES:-}"
lasm_autoscale_max_instances="${BENCH_WORKBENCH_LASM_AUTOSCALE_MAX_INSTANCES:-}"
lasm_autoscale_target_connections="${BENCH_WORKBENCH_LASM_AUTOSCALE_TARGET_CONNECTIONS:-}"
lasm_autoscale_check_ms="${BENCH_WORKBENCH_LASM_AUTOSCALE_CHECK_MS:-}"
lasm_cluster_relay_workers="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_WORKERS:-}"
lasm_cluster_relay_queue="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_QUEUE:-}"
lasm_cluster_accept_workers="${BENCH_WORKBENCH_LASM_CLUSTER_ACCEPT_WORKERS:-}"
lasm_cluster_relay_accept_batch_max="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX:-}"
lasm_cluster_relay_pump_batch_max="${BENCH_WORKBENCH_LASM_CLUSTER_RELAY_PUMP_BATCH_MAX:-}"
out_runs=""
out_fixed_runs=""
out_step_runs=""
out_compare=""
out_analysis=""
out_step_matrix=""
out_mode_compare_repeats=""
out_report=""
out_report_html=""

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
    --lasm-db-records-capture-enabled)
      lasm_db_records_capture_enabled="${2:-}"
      shift 2
      ;;
    --lasm-db-records-capture-enabled=*)
      lasm_db_records_capture_enabled="${1#--lasm-db-records-capture-enabled=}"
      shift
      ;;
    --lasm-mode)
      lasm_mode="${2:-}"
      shift 2
      ;;
    --lasm-mode=*)
      lasm_mode="${1#--lasm-mode=}"
      shift
      ;;
    --lasm-mode-compare-repeats-file)
      lasm_mode_compare_repeats_file="${2:-}"
      shift 2
      ;;
    --lasm-mode-compare-repeats-file=*)
      lasm_mode_compare_repeats_file="${1#--lasm-mode-compare-repeats-file=}"
      shift
      ;;
    --lasm-mode-compare-repeats)
      lasm_mode_compare_repeats="${2:-}"
      shift 2
      ;;
    --lasm-mode-compare-repeats=*)
      lasm_mode_compare_repeats="${1#--lasm-mode-compare-repeats=}"
      shift
      ;;
    --lasm-db-postgres-shared-client-max-active-per-key)
      lasm_db_postgres_shared_client_max_active_per_key="${2:-}"
      shift 2
      ;;
    --lasm-db-postgres-shared-client-max-active-per-key=*)
      lasm_db_postgres_shared_client_max_active_per_key="${1#--lasm-db-postgres-shared-client-max-active-per-key=}"
      shift
      ;;
    --lasm-db-postgres-shared-client-max-active-total)
      lasm_db_postgres_shared_client_max_active_total="${2:-}"
      shift 2
      ;;
    --lasm-db-postgres-shared-client-max-active-total=*)
      lasm_db_postgres_shared_client_max_active_total="${1#--lasm-db-postgres-shared-client-max-active-total=}"
      shift
      ;;
    --lasm-instances)
      lasm_instances="${2:-}"
      shift 2
      ;;
    --lasm-instances=*)
      lasm_instances="${1#--lasm-instances=}"
      shift
      ;;
    --lasm-autoscale-max-instances)
      lasm_autoscale_max_instances="${2:-}"
      shift 2
      ;;
    --lasm-autoscale-max-instances=*)
      lasm_autoscale_max_instances="${1#--lasm-autoscale-max-instances=}"
      shift
      ;;
    --lasm-autoscale-target-connections)
      lasm_autoscale_target_connections="${2:-}"
      shift 2
      ;;
    --lasm-autoscale-target-connections=*)
      lasm_autoscale_target_connections="${1#--lasm-autoscale-target-connections=}"
      shift
      ;;
    --lasm-autoscale-check-ms)
      lasm_autoscale_check_ms="${2:-}"
      shift 2
      ;;
    --lasm-autoscale-check-ms=*)
      lasm_autoscale_check_ms="${1#--lasm-autoscale-check-ms=}"
      shift
      ;;
    --lasm-cluster-relay-workers)
      lasm_cluster_relay_workers="${2:-}"
      shift 2
      ;;
    --lasm-cluster-relay-workers=*)
      lasm_cluster_relay_workers="${1#--lasm-cluster-relay-workers=}"
      shift
      ;;
    --lasm-cluster-relay-queue)
      lasm_cluster_relay_queue="${2:-}"
      shift 2
      ;;
    --lasm-cluster-relay-queue=*)
      lasm_cluster_relay_queue="${1#--lasm-cluster-relay-queue=}"
      shift
      ;;
    --lasm-cluster-accept-workers)
      lasm_cluster_accept_workers="${2:-}"
      shift 2
      ;;
    --lasm-cluster-accept-workers=*)
      lasm_cluster_accept_workers="${1#--lasm-cluster-accept-workers=}"
      shift
      ;;
    --lasm-cluster-relay-accept-batch-max)
      lasm_cluster_relay_accept_batch_max="${2:-}"
      shift 2
      ;;
    --lasm-cluster-relay-accept-batch-max=*)
      lasm_cluster_relay_accept_batch_max="${1#--lasm-cluster-relay-accept-batch-max=}"
      shift
      ;;
    --lasm-cluster-relay-pump-batch-max)
      lasm_cluster_relay_pump_batch_max="${2:-}"
      shift 2
      ;;
    --lasm-cluster-relay-pump-batch-max=*)
      lasm_cluster_relay_pump_batch_max="${1#--lasm-cluster-relay-pump-batch-max=}"
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
    --out-fixed-runs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_fixed_runs="$2"
      shift 2
      ;;
    --out-fixed-runs=*)
      out_fixed_runs="${1#--out-fixed-runs=}"
      shift
      ;;
    --out-step-runs)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_step_runs="$2"
      shift 2
      ;;
    --out-step-runs=*)
      out_step_runs="${1#--out-step-runs=}"
      shift
      ;;
    --out-compare)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_compare="$2"
      shift 2
      ;;
    --out-compare=*)
      out_compare="${1#--out-compare=}"
      shift
      ;;
    --out-analysis)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_analysis="$2"
      shift 2
      ;;
    --out-analysis=*)
      out_analysis="${1#--out-analysis=}"
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
    --out-mode-compare-repeats)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_mode_compare_repeats="$2"
      shift 2
      ;;
    --out-mode-compare-repeats=*)
      out_mode_compare_repeats="${1#--out-mode-compare-repeats=}"
      shift
      ;;
    --out-report)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_report="$2"
      shift 2
      ;;
    --out-report=*)
      out_report="${1#--out-report=}"
      shift
      ;;
    --out-report-html)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      out_report_html="$2"
      shift 2
      ;;
    --out-report-html=*)
      out_report_html="${1#--out-report-html=}"
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
if [ -z "$out_runs" ]; then
  out_runs="${suite_dir}/results/summaries/workbench-full-runs.json"
fi
if [ -z "$out_fixed_runs" ]; then
  out_fixed_runs="${suite_dir}/results/summaries/workbench-benchmark-runs.json"
fi
if [ -z "$out_step_runs" ]; then
  out_step_runs="${suite_dir}/results/summaries/workbench-step-runs.json"
fi
if [ -z "$out_compare" ]; then
  out_compare="${suite_dir}/results/summaries/workbench-benchmark-compare-matrix.json"
fi
if [ -z "$out_analysis" ]; then
  out_analysis="${suite_dir}/results/summaries/workbench-benchmark-analysis.json"
fi
if [ -z "$out_step_matrix" ]; then
  out_step_matrix="${suite_dir}/results/summaries/workbench-step-matrix.json"
fi
if [ -z "$out_mode_compare_repeats" ] && [ -n "$lasm_mode_compare_repeats" ]; then
  out_mode_compare_repeats="${suite_dir}/results/summaries/workbench-lasm-mode-compare-repeats.json"
fi
if [ -z "$lasm_mode_compare_repeats_file" ]; then
  lasm_mode_compare_repeats_file="${suite_dir}/results/summaries/workbench-lasm-mode-compare-repeats.json"
fi
if [ -z "$out_report" ]; then
  out_report="${suite_dir}/results/workbench-full-benchmark-report.md"
fi
if [ -z "$out_report_html" ]; then
  out_report_html="${suite_dir}/results/workbench-full-benchmark-report.html"
fi

mkdir -p "$(dirname "$out_runs")" "$(dirname "$out_report")" "$(dirname "$out_report_html")"

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
  echo "unsupported LASM DB adapter: $lasm_db_adapter" >&2
  exit 2
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
      if [ -z "$lasm_instances" ] || [ "$lasm_instances" -lt 2 ]; then
        lasm_instances="2"
      fi
      lasm_autoscale_max_instances="$lasm_instances"
      ;;
    proxy)
      if [ -z "$lasm_instances" ] || [ "$lasm_instances" -lt 2 ]; then
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

bench_cmd=(
  "${suite_dir}/scripts/run_workbench_benchmark_matrix.sh"
  --matrix "$matrix_path"
  --impls "$impls_csv"
  --endpoints "$endpoints_csv"
  --port "$bench_port"
  --out-runs "$out_fixed_runs"
  --out-compare "$out_compare"
  --out-analysis "$out_analysis"
  --out-report "$out_report"
  --out-report-html "$out_report_html"
)

step_cmd=(
  "${suite_dir}/scripts/run_workbench_step_matrix.sh"
  --matrix "$matrix_path"
  --impls "$impls_csv"
  --endpoints "$endpoints_csv"
  --port "$bench_port"
  --out-runs "$out_step_runs"
  --out-step-matrix "$out_step_matrix"
)

mode_compare_cmd=()

if [ -n "$lasm_db_adapter" ]; then
  bench_cmd+=(--lasm-db-adapter "$lasm_db_adapter")
  step_cmd+=(--lasm-db-adapter "$lasm_db_adapter")
fi
if [ -n "$lasm_db_base" ]; then
  bench_cmd+=(--lasm-db-base "$lasm_db_base")
  step_cmd+=(--lasm-db-base "$lasm_db_base")
fi
if [ -n "$lasm_postgres_dsn_file" ]; then
  bench_cmd+=(--lasm-postgres-dsn-file "$lasm_postgres_dsn_file")
  step_cmd+=(--lasm-postgres-dsn-file "$lasm_postgres_dsn_file")
fi
if [ -n "$lasm_db_records_capture_enabled" ]; then
  case "$lasm_db_records_capture_enabled" in
    0|1) ;;
    *)
      echo "--lasm-db-records-capture-enabled must be 0 or 1" >&2
      exit 2
      ;;
  esac
  bench_cmd+=(--lasm-db-records-capture-enabled "$lasm_db_records_capture_enabled")
  step_cmd+=(--lasm-db-records-capture-enabled "$lasm_db_records_capture_enabled")
fi
if [ -n "$lasm_db_postgres_shared_client_max_active_per_key" ]; then
  bench_cmd+=(--lasm-db-postgres-shared-client-max-active-per-key "$lasm_db_postgres_shared_client_max_active_per_key")
  step_cmd+=(--lasm-db-postgres-shared-client-max-active-per-key "$lasm_db_postgres_shared_client_max_active_per_key")
fi
if [ -n "$lasm_db_postgres_shared_client_max_active_total" ]; then
  bench_cmd+=(--lasm-db-postgres-shared-client-max-active-total "$lasm_db_postgres_shared_client_max_active_total")
  step_cmd+=(--lasm-db-postgres-shared-client-max-active-total "$lasm_db_postgres_shared_client_max_active_total")
fi
if [ -n "$lasm_instances" ]; then
  bench_cmd+=(--lasm-instances "$lasm_instances")
  step_cmd+=(--lasm-instances "$lasm_instances")
fi
if [ -n "$lasm_autoscale_max_instances" ]; then
  bench_cmd+=(--lasm-autoscale-max-instances "$lasm_autoscale_max_instances")
  step_cmd+=(--lasm-autoscale-max-instances "$lasm_autoscale_max_instances")
fi
if [ -n "$lasm_autoscale_target_connections" ]; then
  bench_cmd+=(--lasm-autoscale-target-connections "$lasm_autoscale_target_connections")
  step_cmd+=(--lasm-autoscale-target-connections "$lasm_autoscale_target_connections")
fi
if [ -n "$lasm_autoscale_check_ms" ]; then
  bench_cmd+=(--lasm-autoscale-check-ms "$lasm_autoscale_check_ms")
  step_cmd+=(--lasm-autoscale-check-ms "$lasm_autoscale_check_ms")
fi
if [ -n "$lasm_cluster_relay_workers" ]; then
  bench_cmd+=(--lasm-cluster-relay-workers "$lasm_cluster_relay_workers")
  step_cmd+=(--lasm-cluster-relay-workers "$lasm_cluster_relay_workers")
fi
if [ -n "$lasm_cluster_relay_queue" ]; then
  bench_cmd+=(--lasm-cluster-relay-queue "$lasm_cluster_relay_queue")
  step_cmd+=(--lasm-cluster-relay-queue "$lasm_cluster_relay_queue")
fi
if [ -n "$lasm_cluster_accept_workers" ]; then
  bench_cmd+=(--lasm-cluster-accept-workers "$lasm_cluster_accept_workers")
  step_cmd+=(--lasm-cluster-accept-workers "$lasm_cluster_accept_workers")
fi
if [ -n "$lasm_cluster_relay_accept_batch_max" ]; then
  bench_cmd+=(--lasm-cluster-relay-accept-batch-max "$lasm_cluster_relay_accept_batch_max")
  step_cmd+=(--lasm-cluster-relay-accept-batch-max "$lasm_cluster_relay_accept_batch_max")
fi
if [ -n "$lasm_cluster_relay_pump_batch_max" ]; then
  bench_cmd+=(--lasm-cluster-relay-pump-batch-max "$lasm_cluster_relay_pump_batch_max")
  step_cmd+=(--lasm-cluster-relay-pump-batch-max "$lasm_cluster_relay_pump_batch_max")
fi
if [ "$dry_run" = "true" ]; then
  bench_cmd=("${bench_cmd[@]:0:1}" --dry-run "${bench_cmd[@]:1}")
  step_cmd=("${step_cmd[@]:0:1}" --dry-run "${step_cmd[@]:1}")
fi

if [ -n "$lasm_mode_compare_repeats" ]; then
  case "$lasm_mode_compare_repeats" in
    ''|*[!0-9]*)
      echo "--lasm-mode-compare-repeats must be an integer >= 1" >&2
      exit 2
      ;;
  esac
  if [ "$lasm_mode_compare_repeats" -lt 1 ]; then
    echo "--lasm-mode-compare-repeats must be >= 1" >&2
    exit 2
  fi
  case ",${impls_csv}," in
    *,sec4-lasm,*)
      ;;
    *)
      echo "--lasm-mode-compare-repeats requires impls to include sec4-lasm" >&2
      exit 2
      ;;
  esac
  if [ -z "$out_mode_compare_repeats" ]; then
    echo "mode compare repeats output path is required when repeats are enabled" >&2
    exit 2
  fi
  mode_compare_cmd=(
    "${suite_dir}/scripts/run_workbench_lasm_mode_compare_repeats.sh"
    --repeats "$lasm_mode_compare_repeats"
    --out "$out_mode_compare_repeats"
    --endpoints "$endpoints_csv"
    --port "$bench_port"
  )
  if [ -n "$lasm_db_adapter" ]; then
    mode_compare_cmd+=(--lasm-db-adapter "$lasm_db_adapter")
  fi
  if [ -n "$lasm_db_base" ]; then
    mode_compare_cmd+=(--lasm-db-base "$lasm_db_base")
  fi
  if [ -n "$lasm_postgres_dsn_file" ]; then
    mode_compare_cmd+=(--lasm-postgres-dsn-file "$lasm_postgres_dsn_file")
  fi
  if [ -n "$lasm_db_postgres_shared_client_max_active_per_key" ]; then
    mode_compare_cmd+=(--lasm-db-postgres-shared-client-max-active-per-key "$lasm_db_postgres_shared_client_max_active_per_key")
  fi
  if [ -n "$lasm_db_postgres_shared_client_max_active_total" ]; then
    mode_compare_cmd+=(--lasm-db-postgres-shared-client-max-active-total "$lasm_db_postgres_shared_client_max_active_total")
  fi
  if [ -n "$lasm_instances" ]; then
    mode_compare_cmd+=(--instances "$lasm_instances")
  fi
  if [ -n "$lasm_autoscale_max_instances" ]; then
    mode_compare_cmd+=(--autoscale-max-instances "$lasm_autoscale_max_instances")
  fi
  if [ -n "$lasm_autoscale_target_connections" ]; then
    mode_compare_cmd+=(--autoscale-target-connections "$lasm_autoscale_target_connections")
  fi
  if [ -n "$lasm_autoscale_check_ms" ]; then
    mode_compare_cmd+=(--autoscale-check-ms "$lasm_autoscale_check_ms")
  fi
  if [ -n "$lasm_cluster_relay_workers" ]; then
    mode_compare_cmd+=(--cluster-relay-workers "$lasm_cluster_relay_workers")
  fi
  if [ -n "$lasm_cluster_relay_queue" ]; then
    mode_compare_cmd+=(--cluster-relay-queue "$lasm_cluster_relay_queue")
  fi
  if [ -n "$lasm_cluster_accept_workers" ]; then
    mode_compare_cmd+=(--cluster-accept-workers "$lasm_cluster_accept_workers")
  fi
  if [ -n "$lasm_cluster_relay_accept_batch_max" ]; then
    mode_compare_cmd+=(--cluster-relay-accept-batch-max "$lasm_cluster_relay_accept_batch_max")
  fi
  if [ -n "$lasm_cluster_relay_pump_batch_max" ]; then
    mode_compare_cmd+=(--cluster-relay-pump-batch-max "$lasm_cluster_relay_pump_batch_max")
  fi
  if [ "$dry_run" = "true" ]; then
    mode_compare_cmd=("${mode_compare_cmd[@]:0:1}" --dry-run "${mode_compare_cmd[@]:1}")
  fi
fi

started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
bench_exit_code=0
step_exit_code=0
mode_compare_exit_code=0
publish_exit_code=0
suite_result="passed"

set +e
"${bench_cmd[@]}"
bench_exit_code=$?
if [ "$bench_exit_code" -ne 0 ]; then
  suite_result="failed"
fi
"${step_cmd[@]}"
step_exit_code=$?
if [ "$step_exit_code" -ne 0 ]; then
  suite_result="failed"
fi
if [ "${#mode_compare_cmd[@]}" -gt 0 ]; then
  "${mode_compare_cmd[@]}"
  mode_compare_exit_code=$?
  if [ "$mode_compare_exit_code" -ne 0 ]; then
    suite_result="failed"
  fi
fi
set -e

if [ "$dry_run" = "true" ]; then
  if [ "${#mode_compare_cmd[@]}" -gt 0 ]; then
    echo "run: ${suite_dir}/scripts/publish_report.sh ${out_compare} ${out_report} '' ${out_analysis} ${out_step_matrix} '' ${out_mode_compare_repeats}"
  else
    echo "run: ${suite_dir}/scripts/publish_report.sh ${out_compare} ${out_report} '' ${out_analysis} ${out_step_matrix}"
  fi
  exit 0
fi

if [ -f "$out_compare" ] && [ -f "$out_analysis" ] && [ -f "$out_step_matrix" ]; then
  if [ "${#mode_compare_cmd[@]}" -gt 0 ]; then
    set +e
    "${suite_dir}/scripts/publish_report.sh" "$out_compare" "$out_report" "" "$out_analysis" "$out_step_matrix" "" "$out_mode_compare_repeats"
    publish_exit_code=$?
    set -e
    if [ "$publish_exit_code" -ne 0 ]; then
      suite_result="failed"
    fi
  else
    set +e
    "${suite_dir}/scripts/publish_report.sh" "$out_compare" "$out_report" "" "$out_analysis" "$out_step_matrix"
    publish_exit_code=$?
    set -e
    if [ "$publish_exit_code" -ne 0 ]; then
      suite_result="failed"
    fi
  fi
else
  publish_exit_code=1
  suite_result="failed"
fi

finished_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
matrix_path_rel="$(normalize_repo_path "$matrix_path")"
out_fixed_runs_rel="$(normalize_repo_path "$out_fixed_runs")"
out_step_runs_rel="$(normalize_repo_path "$out_step_runs")"
out_compare_rel="$(normalize_repo_path "$out_compare")"
out_analysis_rel="$(normalize_repo_path "$out_analysis")"
out_step_matrix_rel="$(normalize_repo_path "$out_step_matrix")"
out_mode_compare_repeats_rel="$(normalize_repo_path "$out_mode_compare_repeats")"
out_report_rel="$(normalize_repo_path "$out_report")"
out_report_html_rel="$(normalize_repo_path "$out_report_html")"

jq -n \
  --arg version "0.1" \
  --arg startedAt "$started_at" \
  --arg finishedAt "$finished_at" \
  --arg matrixPath "$matrix_path_rel" \
  --arg impls "$impls_csv" \
  --arg endpoints "$endpoints_csv" \
  --argjson port "$bench_port" \
  --arg fixedRunsPath "$out_fixed_runs_rel" \
  --arg stepRunsPath "$out_step_runs_rel" \
  --arg comparePath "$out_compare_rel" \
  --arg analysisPath "$out_analysis_rel" \
  --arg stepMatrixPath "$out_step_matrix_rel" \
  --arg modeCompareRepeatsPath "$out_mode_compare_repeats_rel" \
  --arg reportPath "$out_report_rel" \
  --arg reportHtmlPath "$out_report_html_rel" \
  --arg suiteResult "$suite_result" \
  --argjson benchExitCode "$bench_exit_code" \
  --argjson stepExitCode "$step_exit_code" \
  --argjson modeCompareExitCode "$mode_compare_exit_code" \
  --argjson publishExitCode "$publish_exit_code" \
  '{
    version: $version,
    startedAt: $startedAt,
    finishedAt: $finishedAt,
    matrixPath: $matrixPath,
    impls: ($impls | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
    endpoints: ($endpoints | split(",") | map(gsub(" "; "")) | map(select(length > 0))),
    port: $port,
    artifacts: {
      fixedRuns: $fixedRunsPath,
      stepRuns: $stepRunsPath,
      compareMatrix: $comparePath,
      analysis: $analysisPath,
      stepMatrix: $stepMatrixPath,
      modeCompareRepeats: (if $modeCompareRepeatsPath == "" then null else $modeCompareRepeatsPath end),
      report: $reportPath,
      reportHtml: $reportHtmlPath
    },
    suiteResult: $suiteResult,
    phaseExitCodes: {
      benchmarkMatrix: $benchExitCode,
      stepMatrix: $stepExitCode,
      modeCompareRepeats: $modeCompareExitCode,
      publishReport: $publishExitCode
    }
  }' >"$out_runs"

echo "wrote workbench full run summary: $out_runs"
echo "wrote workbench full report: $out_report"
echo "wrote workbench full html report: $out_report_html"

if [ "$suite_result" != "passed" ]; then
  exit 1
fi
