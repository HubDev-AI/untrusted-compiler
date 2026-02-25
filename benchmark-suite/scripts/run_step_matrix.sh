#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--impls sec4,sec4-lasm,node,go,rust,c] [--endpoints ping,decode,users-post,users-get,db-hot-write,db-hot-write-tx,db-hot-query-one,db-records]
          [--lasm-db-adapter records-log|sqlite|postgres] [--lasm-db-postgres-dsn-file path]

Runs step-load benchmark profiles for each implementation/endpoint and emits
step analysis artifacts plus a cross-implementation step matrix.
USAGE
}

dry_run="false"
impls_csv="sec4,sec4-lasm,node,go,rust"
endpoints_csv="ping,decode,users-post,users-get"
lasm_db_adapter="${BENCH_LASM_DB_ADAPTER:-}"
lasm_db_postgres_dsn_file="${BENCH_LASM_DB_POSTGRES_DSN_FILE:-}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run="true"
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
    --lasm-db-postgres-dsn-file)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      lasm_db_postgres_dsn_file="$2"
      shift 2
      ;;
    --lasm-db-postgres-dsn-file=*)
      lasm_db_postgres_dsn_file="${1#--lasm-db-postgres-dsn-file=}"
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

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
bench_port="${BENCH_PORT:-18085}"
base_url="http://127.0.0.1:${bench_port}"
summaries_dir="${root_dir}/results/summaries"
mkdir -p "$summaries_dir"

IFS=',' read -r -a impls <<< "$impls_csv"
IFS=',' read -r -a endpoints <<< "$endpoints_csv"

is_supported_impl() {
  case "$1" in
    sec4|sec4-lasm|node|go|rust|c) return 0 ;;
    *) return 1 ;;
  esac
}

is_supported_endpoint() {
  case "$1" in
    ping|decode|users-post|users-get|db-hot-write|db-hot-write-tx|db-hot-query-one|db-records) return 0 ;;
    *) return 1 ;;
  esac
}

is_supported_lasm_db_adapter() {
  case "$1" in
    records-log|sqlite|postgres) return 0 ;;
    *) return 1 ;;
  esac
}

for raw_impl in "${impls[@]}"; do
  impl="${raw_impl// /}"
  [ -z "$impl" ] && continue
  if ! is_supported_impl "$impl"; then
    echo "unsupported implementation for step matrix: ${impl}" >&2
    exit 2
  fi
done

for raw_endpoint in "${endpoints[@]}"; do
  endpoint="${raw_endpoint// /}"
  [ -z "$endpoint" ] && continue
  if ! is_supported_endpoint "$endpoint"; then
    echo "unsupported endpoint for step matrix: ${endpoint}" >&2
    exit 2
  fi
done

if [ -n "$lasm_db_adapter" ] && ! is_supported_lasm_db_adapter "$lasm_db_adapter"; then
  echo "unsupported LASM DB adapter: ${lasm_db_adapter}" >&2
  exit 2
fi

lasm_postgres_dsn=""
if [ "$lasm_db_adapter" = "postgres" ]; then
  if [ -n "$lasm_db_postgres_dsn_file" ]; then
    if [ ! -f "$lasm_db_postgres_dsn_file" ]; then
      echo "lasm postgres dsn file not found: ${lasm_db_postgres_dsn_file}" >&2
      exit 2
    fi
    lasm_postgres_dsn="$(<"$lasm_db_postgres_dsn_file")"
    lasm_postgres_dsn="${lasm_postgres_dsn//$'\r'/}"
    lasm_postgres_dsn="${lasm_postgres_dsn//$'\n'/}"
  elif [ -n "${SEC4_RT_LASM_DB_POSTGRES_DSN:-}" ]; then
    lasm_postgres_dsn="${SEC4_RT_LASM_DB_POSTGRES_DSN}"
  else
    echo "postgres adapter requires --lasm-db-postgres-dsn-file or SEC4_RT_LASM_DB_POSTGRES_DSN" >&2
    exit 2
  fi
fi

if [ "$dry_run" = "true" ]; then
  "${root_dir}/scripts/preflight.sh" --impls "$impls_csv" --dry-run-only
else
  "${root_dir}/scripts/preflight.sh" --impls "$impls_csv"
fi

start_service() {
  local impl="$1"
  local service_dir="${root_dir}/services/${impl}"
  local log_file="${root_dir}/results/raw/${impl}-step-service.log"
  if [ ! -d "$service_dir" ]; then
    echo "service directory not found for impl=${impl}: ${service_dir}" >&2
    return 2
  fi

  case "$impl" in
    sec4)
      (
        cd "$service_dir"
        ./build.sh >/dev/null
        PORT="$bench_port" ./sec4-bench-server
      ) >"$log_file" 2>&1 &
      ;;
    sec4-lasm)
      (
        cd "$service_dir"
        sec4_lasm_cmd=(cargo run -q -p sec4 -- run --path "$service_dir" --backend lasm --port "$bench_port" --serve-timeout-ms 20000)
        if [ -n "$lasm_db_adapter" ]; then
          sec4_lasm_cmd+=(--db-adapter "$lasm_db_adapter")
        fi
        if [ -n "$lasm_postgres_dsn" ]; then
          SEC4_RT_LASM_DB_POSTGRES_DSN="$lasm_postgres_dsn" "${sec4_lasm_cmd[@]}"
        else
          "${sec4_lasm_cmd[@]}"
        fi
      ) >"$log_file" 2>&1 &
      ;;
    node)
      (
        cd "$service_dir"
        PORT="$bench_port" npm run start
      ) >"$log_file" 2>&1 &
      ;;
    go)
      (
        cd "$service_dir"
        PORT="$bench_port" go run .
      ) >"$log_file" 2>&1 &
      ;;
    rust)
      (
        cd "$service_dir"
        PORT="$bench_port" cargo run --quiet
      ) >"$log_file" 2>&1 &
      ;;
    c)
      (
        cd "$service_dir"
        cc -O2 -std=c11 server.c -o c-bench-server
        PORT="$bench_port" ./c-bench-server
      ) >"$log_file" 2>&1 &
      ;;
  esac

  service_pid="$!"
  service_log_file="$log_file"
}

wait_for_ready() {
  local pid="$1"
  local tries=120
  while [ "$tries" -gt 0 ]; do
    if ! kill -0 "$pid" >/dev/null 2>&1; then
      return 2
    fi
    ping_body="$(curl -fsS "${base_url}/ping" 2>/dev/null || true)"
    if [ "$ping_body" = "ok" ]; then
      return 0
    fi
    tries=$((tries - 1))
    sleep 0.25
  done
  return 1
}

for raw_impl in "${impls[@]}"; do
  impl="${raw_impl// /}"
  [ -z "$impl" ] && continue
  echo "=== step impl=${impl} ==="

  if [ "$dry_run" = "true" ]; then
    if [ "$impl" = "sec4-lasm" ] && [ -n "$lasm_db_adapter" ]; then
      if [ "$lasm_db_adapter" = "postgres" ]; then
        echo "start: ${impl} service on :${bench_port} (db-adapter=${lasm_db_adapter} dsn-file=${lasm_db_postgres_dsn_file:-ENV})"
      else
        echo "start: ${impl} service on :${bench_port} (db-adapter=${lasm_db_adapter})"
      fi
    else
      echo "start: ${impl} service on :${bench_port}"
    fi
    for raw_endpoint in "${endpoints[@]}"; do
      endpoint="${raw_endpoint// /}"
      [ -z "$endpoint" ] && continue
      echo "run: ${root_dir}/scripts/run_step_profile.sh --dry-run ${impl} ${endpoint} ${base_url}"
      echo "run: ${root_dir}/scripts/analyze_step_profile.sh ${summaries_dir}/${impl}-${endpoint}-step.json ${summaries_dir}/${impl}-${endpoint}-step-analysis.json"
    done
    continue
  fi

  service_pid=""
  service_log_file=""
  start_service "$impl"
  pid="$service_pid"
  log_file="$service_log_file"

  cleanup_impl() {
    if kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
  }
  trap cleanup_impl EXIT

  echo "waiting for step readiness impl=${impl} url=${base_url}/ping"
  if ! wait_for_ready "$pid"; then
    echo "step service failed readiness check for impl=${impl}" >&2
    if [ -n "$log_file" ] && [ -f "$log_file" ]; then
      echo "--- ${impl} step service log tail ---" >&2
      tail -n 40 "$log_file" >&2 || true
      echo "--- end log tail ---" >&2
    fi
    cleanup_impl
    trap - EXIT
    exit 1
  fi

  for raw_endpoint in "${endpoints[@]}"; do
    endpoint="${raw_endpoint// /}"
    [ -z "$endpoint" ] && continue
    BENCH_SERVER_PID="$pid" "${root_dir}/scripts/run_step_profile.sh" "$impl" "$endpoint" "$base_url"
    "${root_dir}/scripts/analyze_step_profile.sh" \
      "${summaries_dir}/${impl}-${endpoint}-step.json" \
      "${summaries_dir}/${impl}-${endpoint}-step-analysis.json" \
      "${STEP_KNEE_THRESHOLD:-0.9}"
  done

  cleanup_impl
  trap - EXIT
done

step_matrix_path="${summaries_dir}/step-matrix.json"
if [ "$dry_run" = "true" ]; then
  echo "run: ${root_dir}/scripts/compare_step_matrix.sh ${summaries_dir} ${step_matrix_path} ${impls_csv} ${endpoints_csv}"
  exit 0
fi

"${root_dir}/scripts/compare_step_matrix.sh" "$summaries_dir" "$step_matrix_path" "$impls_csv" "$endpoints_csv"
echo "step matrix written to ${step_matrix_path}"
