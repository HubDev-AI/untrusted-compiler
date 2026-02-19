#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<USAGE
usage: $0 [--dry-run] [--impls sec4,sec4-lasm,node,go,rust,c] [--endpoints ping,decode,users-post,users-get] [--sec-audit path]

Runs benchmark profiles for each implementation, builds per-impl reports,
then emits compare-matrix and markdown report artifacts.
USAGE
}

dry_run="false"
impls_csv="sec4,sec4-lasm,node,go,rust"
endpoints_csv="ping,decode,users-post,users-get"
sec_audit_path=""

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
    --sec-audit)
      if [ "$#" -lt 2 ]; then
        usage
        exit 2
      fi
      sec_audit_path="$2"
      shift 2
      ;;
    --sec-audit=*)
      sec_audit_path="${1#--sec-audit=}"
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
results_dir="${root_dir}/results"
summaries_dir="${results_dir}/summaries"
raw_dir="${results_dir}/raw"
mkdir -p "$summaries_dir" "$raw_dir"

if [ -z "$sec_audit_path" ]; then
  candidate="${root_dir}/../baselines/sec-audit/default-secure-prod.hello.json"
  if [ -f "$candidate" ]; then
    sec_audit_path="$candidate"
  fi
fi

IFS=',' read -r -a impls <<< "$impls_csv"
if [ "${#impls[@]}" -eq 0 ]; then
  echo "no implementations provided" >&2
  exit 2
fi

IFS=',' read -r -a endpoints <<< "$endpoints_csv"
if [ "${#endpoints[@]}" -eq 0 ]; then
  echo "no endpoints provided" >&2
  exit 2
fi

is_supported_impl() {
  case "$1" in
    sec4|sec4-lasm|node|go|rust|c)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

is_supported_endpoint() {
  case "$1" in
    ping|decode|users-post|users-get)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

for raw_impl in "${impls[@]}"; do
  normalized="${raw_impl// /}"
  [ -z "$normalized" ] && continue
  if ! is_supported_impl "$normalized"; then
    echo "unsupported implementation for orchestrator: ${normalized}" >&2
    exit 2
  fi
done

for raw_endpoint in "${endpoints[@]}"; do
  normalized="${raw_endpoint// /}"
  [ -z "$normalized" ] && continue
  if ! is_supported_endpoint "$normalized"; then
    echo "unsupported endpoint for orchestrator: ${normalized}" >&2
    exit 2
  fi
done

if [ "$dry_run" = "true" ]; then
  "${root_dir}/scripts/preflight.sh" --impls "$impls_csv" --dry-run-only
else
  "${root_dir}/scripts/preflight.sh" --impls "$impls_csv"
fi

start_service() {
  local impl="$1"
  local service_dir="${root_dir}/services/${impl}"
  local log_file="${raw_dir}/${impl}-service.log"
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
        cargo run -q -p sec4 -- run --path "$service_dir" --backend lasm --port "$bench_port" --serve-timeout-ms 20000
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

for impl in "${impls[@]}"; do
  impl="${impl// /}"
  [ -z "$impl" ] && continue

  echo "=== impl=${impl} ==="

  if [ "$dry_run" = "true" ]; then
    echo "start: ${impl} service on :${bench_port}"
    for raw_endpoint in "${endpoints[@]}"; do
      endpoint="${raw_endpoint// /}"
      [ -z "$endpoint" ] && continue
      echo "run: ${root_dir}/scripts/run_profile.sh --dry-run ${impl} ${endpoint} ${base_url}"
    done
    if { [ "$impl" = "sec4" ] || [ "$impl" = "sec4-lasm" ]; } && [ -n "$sec_audit_path" ]; then
      echo "run: ${root_dir}/scripts/build_report.sh ${impl} ${results_dir} ${summaries_dir}/${impl}-report.json ${sec_audit_path} ${endpoints_csv}"
    else
      echo "run: ${root_dir}/scripts/build_report.sh ${impl} ${results_dir} ${summaries_dir}/${impl}-report.json \"\" ${endpoints_csv}"
    fi
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

  echo "waiting for readiness impl=${impl} url=${base_url}/ping"
  if ! wait_for_ready "$pid"; then
    echo "service failed readiness check for impl=${impl} on port ${bench_port}" >&2
    if [ -n "$log_file" ] && [ -f "$log_file" ]; then
      echo "--- ${impl} service log tail ---" >&2
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
    BENCH_SERVER_PID="$pid" "${root_dir}/scripts/run_profile.sh" "$impl" "$endpoint" "$base_url"
  done
  if { [ "$impl" = "sec4" ] || [ "$impl" = "sec4-lasm" ]; } && [ -n "$sec_audit_path" ]; then
    "${root_dir}/scripts/build_report.sh" "$impl" "$results_dir" "${summaries_dir}/${impl}-report.json" "$sec_audit_path" "$endpoints_csv"
  else
    "${root_dir}/scripts/build_report.sh" "$impl" "$results_dir" "${summaries_dir}/${impl}-report.json" "" "$endpoints_csv"
  fi

  cleanup_impl
  trap - EXIT
done

matrix_path="${summaries_dir}/compare-matrix.json"
analysis_path="${summaries_dir}/analysis.json"
report_md_path="${results_dir}/benchmark-report.md"

if [ "$dry_run" = "true" ]; then
  echo "run: ${root_dir}/scripts/compare_matrix.sh ${summaries_dir} ${matrix_path} ${impls_csv}"
  echo "run: ${root_dir}/scripts/analyze_matrix.sh ${matrix_path} ${analysis_path}"
  if [ -n "$sec_audit_path" ]; then
    echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} ${sec_audit_path} ${analysis_path}"
  else
    echo "run: ${root_dir}/scripts/publish_report.sh ${matrix_path} ${report_md_path} \"\" ${analysis_path}"
  fi
  exit 0
fi

"${root_dir}/scripts/compare_matrix.sh" "$summaries_dir" "$matrix_path" "$impls_csv"
"${root_dir}/scripts/analyze_matrix.sh" "$matrix_path" "$analysis_path"

if [ -n "$sec_audit_path" ]; then
  "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "$sec_audit_path" "$analysis_path"
else
  "${root_dir}/scripts/publish_report.sh" "$matrix_path" "$report_md_path" "" "$analysis_path"
fi

echo "comparison matrix written to ${matrix_path}"
echo "analysis written to ${analysis_path}"
echo "benchmark report written to ${report_md_path}"
